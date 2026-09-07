//! Project manifests, unit placement, and CLI-facing output helpers.

use super::CompilationResult;
use super::diagnostic::{DiagCode, Diagnostic};
use super::source::SourceSpan;
use super::z80::GeneratedProgram;
use crate::asm::AssembledProgram;
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectTarget {
    GenericZ80,
    Tvc,
    Zx82,
}

impl ProjectTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GenericZ80 => "generic-z80",
            Self::Tvc => "tvc",
            Self::Zx82 => "zx82",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "generic-z80" => Self::GenericZ80,
            "tvc" => Self::Tvc,
            "zx82" => Self::Zx82,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    C80,
    Asm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub version: u32,
    pub target: ProjectTarget,
    pub entry: Option<String>,
    pub stack: Option<StackSpec>,
    pub reserves: Vec<ReserveSpec>,
    pub units: Vec<UnitSpec>,
    pub basic: Option<BasicSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackSpec {
    pub base: u16,
    pub size: u16,
    pub interrupt_allowance: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReserveSpec {
    pub name: String,
    pub base: u16,
    pub size: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitSpec {
    pub name: String,
    pub path: PathBuf,
    pub kind: UnitKind,
    pub origin: Option<u16>,
    pub exports: Vec<String>,
    pub stack_extra: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicSpec {
    pub path: PathBuf,
    pub origin: u16,
    pub size: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledBasic {
    pub origin: u16,
    pub region_size: u16,
    pub payload: Vec<u8>,
    pub substituted_source: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestDto {
    #[serde(default)]
    version: Option<u32>,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    entry: Option<String>,
    #[serde(default)]
    stack: Option<StackDto>,
    #[serde(default)]
    reserve: Vec<ReserveDto>,
    #[serde(default)]
    unit: Vec<UnitDto>,
    #[serde(default)]
    basic: Option<BasicDto>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StackDto {
    base: u64,
    size: u64,
    #[serde(default)]
    interrupt_allowance: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReserveDto {
    name: String,
    base: u64,
    size: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitDto {
    name: String,
    path: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    origin: Option<u64>,
    #[serde(default)]
    exports: Vec<String>,
    #[serde(default)]
    stack_extra: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BasicDto {
    path: String,
    origin: u64,
    size: u64,
}

pub fn parse_manifest(text: &str) -> Result<Manifest, String> {
    let dto: ManifestDto = toml::from_str(text).map_err(|err| err.to_string())?;
    let version = dto.version.unwrap_or(1);
    if version != 1 {
        return Err(format!("unsupported manifest version {version}"));
    }
    let target = match dto.target.as_deref() {
        None => ProjectTarget::GenericZ80,
        Some(value) => {
            ProjectTarget::parse(value).ok_or_else(|| format!("unsupported target '{value}'"))?
        }
    };
    let stack = match dto.stack {
        Some(s) => Some(StackSpec {
            base: fit_u16("stack.base", s.base)?,
            size: fit_u16("stack.size", s.size)?,
            interrupt_allowance: s
                .interrupt_allowance
                .map(|v| fit_u16("stack.interrupt_allowance", v))
                .transpose()?,
        }),
        None => None,
    };
    let mut names = BTreeSet::new();
    let mut reserves = Vec::new();
    for r in dto.reserve {
        if !names.insert(r.name.clone()) {
            return Err(format!("duplicate reserve name '{}'", r.name));
        }
        reserves.push(ReserveSpec {
            name: r.name,
            base: fit_u16("reserve.base", r.base)?,
            size: fit_range_size("reserve.size", r.size)?,
        });
    }
    names.clear();
    let mut units = Vec::new();
    for u in dto.unit {
        if u.name == "project" {
            return Err("unit name 'project' is reserved".to_string());
        }
        if !is_ascii_ident(&u.name) {
            return Err(format!("unit name '{}' is not an ASCII identifier", u.name));
        }
        if !names.insert(u.name.clone()) {
            return Err(format!("duplicate unit name '{}'", u.name));
        }
        let kind = match u.kind.as_deref() {
            None | Some("c80") => UnitKind::C80,
            Some("asm") => UnitKind::Asm,
            Some(other) => return Err(format!("unsupported unit kind '{other}'")),
        };
        if kind == UnitKind::C80 && (!u.exports.is_empty() || u.stack_extra.is_some()) {
            return Err(format!(
                "c80 unit '{}' cannot use exports or stack_extra",
                u.name
            ));
        }
        units.push(UnitSpec {
            name: u.name,
            path: PathBuf::from(u.path),
            kind,
            origin: u.origin.map(|v| fit_u16("unit.origin", v)).transpose()?,
            exports: u.exports,
            stack_extra: u
                .stack_extra
                .map(|v| fit_u16("unit.stack_extra", v))
                .transpose()?,
        });
    }
    if units.is_empty() {
        return Err("manifest has no [[unit]] entries".to_string());
    }
    let basic = match dto.basic {
        None => None,
        Some(b) => {
            if target != ProjectTarget::Tvc {
                return Err("[basic] is only supported for target = \"tvc\"".to_string());
            }
            let origin = fit_u16("basic.origin", b.origin)?;
            let size = fit_range_size("basic.size", b.size)?;
            if u32::from(origin) + u32::from(size) > 65536 {
                return Err(
                    "basic.origin+basic.size overflows the 16-bit address space".to_string()
                );
            }
            Some(BasicSpec {
                path: PathBuf::from(b.path),
                origin,
                size,
            })
        }
    };
    Ok(Manifest {
        version,
        target,
        entry: dto.entry,
        stack,
        reserves,
        units,
        basic,
    })
}

fn fit_u16(field: &str, value: u64) -> Result<u16, String> {
    u16::try_from(value).map_err(|_| format!("{field} is out of range"))
}

fn fit_range_size(field: &str, value: u64) -> Result<u16, String> {
    if value == 0 || value > 65535 {
        return Err(format!("{field} must be 1..=65535"));
    }
    Ok(value as u16)
}

pub fn is_ascii_ident(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// In-memory project unit. Paths are already resolved by the CLI adapter.
pub struct ProjectUnitInput<'a> {
    pub name: &'a str,
    pub kind: UnitKind,
    pub origin: Option<u16>,
    pub text: &'a str,
    pub source_name: &'a str,
    pub exports: &'a [String],
    pub stack_extra: Option<u16>,
}

pub fn compile_project(
    manifest: &Manifest,
    units: &[ProjectUnitInput<'_>],
    basic_source: Option<&str>,
) -> CompilationResult {
    use super::ast::CallConv;
    use super::ir::{FuncId, TypedProgram};
    use super::lower::{
        chunks_emit_bytes, lower_to_chunks, map_chunks_from_lines, max_stack_bound, render_chunks,
    };
    use super::parser::parse_file;
    use super::semantics::{
        BasicLayout, ProjectBuiltins, ReserveLayout, StackLayout, UnitExports,
        analyze_unit_in_project, apply_const_bits, check_call_graph, collect_exports,
    };
    use super::source::{FileId, IdGen, SourceMap};
    use super::z80::{GeneratedFunction, asm_global_label, asm_label};
    use super::{CompilationResult, diagnostic::DiagCode};
    use crate::asm::assemble_program;

    let mut sources = SourceMap::new();
    let mut diagnostics = Vec::new();
    let mut ids = IdGen::new();
    let mut parsed_units = Vec::new();
    let mut ast_units = Vec::new();

    for unit in units {
        let file = sources.add(unit.source_name, unit.text);
        parsed_units.push((file, unit));
        if unit.kind == UnitKind::C80 {
            let ast = parse_file(sources.get(file), &mut ids, &mut diagnostics);
            ast_units.push((unit.name.to_string(), file, ast));
        }
    }

    let stack_layout = manifest.stack.as_ref().map(|s| StackLayout {
        base: s.base,
        size: s.size,
    });
    let basic_layout = manifest.basic.as_ref().map(|b| BasicLayout {
        origin: b.origin,
        size: b.size,
    });
    let reserve_layouts: Vec<ReserveLayout<'_>> = manifest
        .reserves
        .iter()
        .map(|r| ReserveLayout {
            name: r.name.as_str(),
            base: r.base,
            size: r.size,
        })
        .collect();
    let builtins = ProjectBuiltins {
        stack: stack_layout,
        basic: basic_layout,
        reserves: &reserve_layouts,
    };

    let mut exports: HashMap<String, UnitExports> = HashMap::new();
    for (name, _, ast) in &ast_units {
        exports.insert(name.clone(), collect_exports(ast, &mut diagnostics));
    }

    let mut program = TypedProgram {
        structs: Vec::new(),
        globals: Vec::new(),
        functions: Vec::new(),
    };
    let mut all_calls = Vec::new();
    let mut parsed_asts = Vec::new();
    for (name, file, ast) in ast_units {
        let (part, calls) = analyze_unit_in_project(
            &ast,
            &name,
            Some(&exports),
            Some(builtins),
            &mut diagnostics,
        );
        if let Some(slot) = exports.get_mut(&name) {
            apply_const_bits(slot, &part);
        }
        program.structs.extend(part.structs);
        program.globals.extend(part.globals);
        program.functions.extend(part.functions);
        all_calls.extend(calls);
        parsed_asts.push(ast);
        let _ = file;
        let _ = name;
    }
    check_call_graph(&all_calls, &mut diagnostics);

    let ast_list = parsed_asts;
    if diagnostics.iter().any(Diagnostic::is_error) {
        return CompilationResult {
            sources,
            diagnostics,
            units: ast_list,
            program: None,
            code: None,
            basic: None,
        };
    }

    let chunks = match lower_to_chunks(&program, &mut ids, &mut diagnostics) {
        Some(chunks) => chunks,
        None => {
            return CompilationResult {
                sources,
                diagnostics,
                units: ast_list,
                program: Some(program),
                code: None,
                basic: None,
            };
        }
    };

    let c80_stack = max_stack_bound(&chunks);
    let mut chunks_by_file: HashMap<FileId, Vec<super::lower::EmitChunk>> = HashMap::new();
    for chunk in chunks {
        chunks_by_file
            .entry(chunk.file_id())
            .or_default()
            .push(chunk);
    }

    let mut subst = HashMap::new();
    for (unit_name, unit_ex) in &exports {
        for (sym_name, sym) in &unit_ex.symbols {
            if !match sym {
                super::semantics::ExportedSymbol::Function { is_pub, .. }
                | super::semantics::ExportedSymbol::Value { is_pub, .. } => *is_pub,
            } {
                continue;
            }
            let value = match sym {
                super::semantics::ExportedSymbol::Function { id, .. } => asm_label(*id, sym_name),
                super::semantics::ExportedSymbol::Value {
                    id, is_const, bits, ..
                } => {
                    if *is_const {
                        bits.unwrap_or(0).to_string()
                    } else {
                        asm_global_label(*id, sym_name)
                    }
                }
            };
            subst.insert((unit_name.clone(), sym_name.clone()), value);
        }
    }
    insert_project_symbols(&mut subst, manifest);

    let mut asm_export_labels: HashMap<(String, String), String> = HashMap::new();
    for (file, unit) in &parsed_units {
        if unit.kind != UnitKind::Asm {
            continue;
        }
        match collect_asm_labels(unit.text, file.0, unit.exports, dummy_span(*file)) {
            Ok(map) => {
                for (export, label) in &map {
                    subst.insert((unit.name.to_string(), export.clone()), label.clone());
                    asm_export_labels
                        .insert((unit.name.to_string(), export.clone()), label.clone());
                }
            }
            Err(diag) => diagnostics.push(diag),
        }
    }

    if diagnostics.iter().any(Diagnostic::is_error) {
        return CompilationResult {
            sources,
            diagnostics,
            units: ast_list,
            program: Some(program),
            code: None,
            basic: None,
        };
    }

    let mut assembly = String::new();
    let mut c80_ranges: Vec<(FileId, usize, usize, u16)> = Vec::new();
    let mut first_origin = None;

    for (file, unit) in &parsed_units {
        let dummy = dummy_span(*file);
        match unit.kind {
            UnitKind::C80 => {
                let Some(unit_chunks) = chunks_by_file.get(file) else {
                    continue;
                };
                if !chunks_emit_bytes(unit_chunks) {
                    continue;
                }
                let Some(origin) = unit.origin else {
                    diagnostics.push(Diagnostic::error(
                        DiagCode::LnMissingOrigin,
                        dummy,
                        format!("unit '{}' emits bytes and requires origin", unit.name),
                    ));
                    continue;
                };
                let (_, text) = render_chunks(unit_chunks);
                let block = format!("ORG 0x{origin:04X}\n{text}");
                let (start, end) = append_assembly(&mut assembly, &block);
                c80_ranges.push((*file, start, end, origin));
                first_origin.get_or_insert(origin);
            }
            UnitKind::Asm => {
                let rewritten =
                    match prepare_asm_source(unit.text, file.0, unit.exports, dummy, &subst) {
                        Ok(text) => text,
                        Err(diag) => {
                            diagnostics.push(diag);
                            continue;
                        }
                    };
                if !asm_emits_bytes(&rewritten) {
                    continue;
                }
                let Some(origin) = unit.origin else {
                    diagnostics.push(Diagnostic::error(
                        DiagCode::LnMissingOrigin,
                        dummy,
                        format!("unit '{}' emits bytes and requires origin", unit.name),
                    ));
                    continue;
                };
                first_origin.get_or_insert(origin);
                let block = format!("ORG 0x{origin:04X}\n{rewritten}");
                append_assembly(&mut assembly, &block);
            }
        }
    }

    if diagnostics.iter().any(Diagnostic::is_error) {
        return CompilationResult {
            sources,
            diagnostics,
            units: ast_list,
            program: Some(program),
            code: None,
            basic: None,
        };
    }

    let origin = first_origin.unwrap_or(0);
    let assembled = match assemble_program(&assembly, origin) {
        Ok(assembled) => assembled,
        Err(err) => {
            diagnostics.push(Diagnostic::error(
                DiagCode::CgInternal,
                dummy_span(FileId(0)),
                format!("assembler rejected generated code: {err}"),
            ));
            return CompilationResult {
                sources,
                diagnostics,
                units: ast_list,
                program: Some(program),
                code: None,
                basic: None,
            };
        }
    };

    check_layout_ranges(
        manifest,
        &assembled,
        dummy_span(FileId(0)),
        &mut diagnostics,
    );
    check_stack_budget(
        manifest,
        c80_stack,
        units,
        dummy_span(FileId(0)),
        &mut diagnostics,
    );
    check_symbol_collisions(&assembled, dummy_span(FileId(0)), &mut diagnostics);

    if diagnostics.iter().any(Diagnostic::is_error) {
        return CompilationResult {
            sources,
            diagnostics,
            units: ast_list,
            program: Some(program),
            code: None,
            basic: None,
        };
    }

    let mut functions = Vec::new();
    let mut globals = Vec::new();
    let mut items = Vec::new();
    for (file, start, end, unit_origin) in &c80_ranges {
        if let Some(unit_chunks) = chunks_by_file.get(file) {
            let (mut funcs, mut globs) =
                map_chunks_from_lines(unit_chunks, &assembled, *start, *end, *unit_origin);
            functions.append(&mut funcs);
            globals.append(&mut globs);
            for chunk in unit_chunks {
                items.extend(chunk.items().iter().cloned());
            }
        }
    }

    for ((unit_name, export), label) in &asm_export_labels {
        let Some(&addr) = assembled.symbols.get(label) else {
            continue;
        };
        let extra = units
            .iter()
            .find(|u| u.name == unit_name.as_str())
            .and_then(|u| u.stack_extra)
            .unwrap_or(0);
        functions.push(GeneratedFunction {
            name: export.clone(),
            label: label.clone(),
            id: FuncId(super::source::NodeId(0)),
            span: dummy_span(FileId(0)),
            addr,
            size: 0,
            conv: CallConv::Register,
            param_homes: Vec::new(),
            param_types: Vec::new(),
            ret: None,
            stack_bound: extra,
            stack_provenance: super::z80::StackProvenance::Declared,
            frame_bytes: 0,
            instruction_ids: Vec::new(),
            mapped: Vec::new(),
        });
    }

    let code = super::z80::GeneratedProgram {
        origin: assembled.origin,
        items,
        assembly,
        assembled,
        functions,
        globals,
    };

    let mut basic_out = None;
    if let Some(spec) = &manifest.basic {
        match compile_basic_unit(
            spec,
            basic_source,
            &subst,
            &code.assembled,
            dummy_span(FileId(0)),
        ) {
            Ok(compiled) => basic_out = Some(compiled),
            Err(diag) => diagnostics.push(diag),
        }
    } else if basic_source.is_some() {
        diagnostics.push(Diagnostic::error(
            DiagCode::CgUnsupported,
            dummy_span(FileId(0)),
            "BASIC source was supplied without a [basic] manifest region",
        ));
    }

    if diagnostics.iter().any(Diagnostic::is_error) {
        return CompilationResult {
            sources,
            diagnostics,
            units: ast_list,
            program: Some(program),
            code: None,
            basic: None,
        };
    }

    CompilationResult {
        sources,
        diagnostics,
        units: ast_list,
        program: Some(program),
        code: Some(code),
        basic: basic_out,
    }
}

fn dummy_span(file: super::source::FileId) -> SourceSpan {
    SourceSpan::point(file, 0)
}

fn append_assembly(assembly: &mut String, block: &str) -> (usize, usize) {
    if !assembly.is_empty() && !assembly.ends_with('\n') {
        assembly.push('\n');
    }
    let start = assembly.lines().count() + 1;
    assembly.push_str(block);
    if !block.ends_with('\n') {
        assembly.push('\n');
    }
    let end = assembly.lines().count() + 1;
    (start, end)
}

fn insert_project_symbols(subst: &mut HashMap<(String, String), String>, manifest: &Manifest) {
    if let Some(stack) = &manifest.stack {
        let end = u32::from(stack.base) + u32::from(stack.size);
        subst.insert(
            ("project".into(), "stack_base".into()),
            stack.base.to_string(),
        );
        subst.insert(
            ("project".into(), "stack_size".into()),
            stack.size.to_string(),
        );
        subst.insert(
            ("project".into(), "stack_top".into()),
            ((end % 65536) as u16).to_string(),
        );
        subst.insert(("project".into(), "stack_end".into()), end.to_string());
    }
    if let Some(basic) = &manifest.basic {
        let end = u32::from(basic.origin) + u32::from(basic.size);
        subst.insert(
            ("project".into(), "basic_base".into()),
            basic.origin.to_string(),
        );
        subst.insert(
            ("project".into(), "basic_size".into()),
            basic.size.to_string(),
        );
        subst.insert(("project".into(), "basic_end".into()), end.to_string());
        subst.insert(
            ("project".into(), "basic_himem".into()),
            (end - 1).to_string(),
        );
    }
    for r in &manifest.reserves {
        let end = u32::from(r.base) + u32::from(r.size);
        subst.insert(
            ("project".into(), format!("reserve_{}_base", r.name)),
            r.base.to_string(),
        );
        subst.insert(
            ("project".into(), format!("reserve_{}_size", r.name)),
            r.size.to_string(),
        );
        subst.insert(
            ("project".into(), format!("reserve_{}_end", r.name)),
            end.to_string(),
        );
    }
}

fn collect_asm_labels(
    source: &str,
    file_id: u32,
    exports: &[String],
    span: SourceSpan,
) -> Result<HashMap<String, String>, Diagnostic> {
    let mut originals: HashMap<String, String> = HashMap::new();
    for (idx, raw) in source.lines().enumerate() {
        let line = strip_asm_comment(raw);
        let trimmed = line.trim();
        if mnemonic_is(trimmed, "ORG") || mnemonic_is(trimmed, "BASIC_START") {
            return Err(Diagnostic::error(
                DiagCode::CgUnsupported,
                span,
                format!("line {}: ASM units cannot use ORG or BASIC_START", idx + 1),
            ));
        }
        let mut rest = trimmed;
        while let Some(colon) = rest.find(':') {
            let before = rest[..colon].trim();
            if before.is_empty() || before.contains(char::is_whitespace) {
                break;
            }
            if !is_asm_label(before) {
                break;
            }
            let upper = before.to_ascii_uppercase();
            if let Some(prev) = originals.get(&upper) {
                if prev != before {
                    return Err(Diagnostic::error(
                        DiagCode::TyDuplicateName,
                        span,
                        format!("ASM label case collision between '{prev}' and '{before}'"),
                    ));
                }
            }
            originals.insert(upper, before.to_string());
            rest = rest[colon + 1..].trim_start();
        }
    }
    let mut export_map = HashMap::new();
    for export in exports {
        let upper = export.to_ascii_uppercase();
        let Some(original) = originals.get(&upper) else {
            return Err(Diagnostic::error(
                DiagCode::TyUnresolvedName,
                span,
                format!("ASM export '{export}' is not defined"),
            ));
        };
        if original != export {
            return Err(Diagnostic::error(
                DiagCode::TyUnresolvedName,
                span,
                format!("ASM export '{export}' must match label spelling '{original}'"),
            ));
        }
        export_map.insert(export.clone(), format!("U{file_id}_{upper}"));
    }
    Ok(export_map)
}

fn prepare_asm_source(
    source: &str,
    file_id: u32,
    exports: &[String],
    span: SourceSpan,
    subst: &HashMap<(String, String), String>,
) -> Result<String, Diagnostic> {
    let _ = collect_asm_labels(source, file_id, exports, span)?;
    let substituted = substitute_markers(source, subst)
        .map_err(|msg| Diagnostic::error(DiagCode::TyUnresolvedName, span, msg))?;
    Ok(namespace_asm_labels(&substituted, file_id))
}

fn substitute_markers(
    source: &str,
    subst: &HashMap<(String, String), String>,
) -> Result<String, String> {
    let mut out = String::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut in_string = false;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '"' {
            in_string = !in_string;
            out.push(ch);
            i += 1;
            continue;
        }
        if !in_string && ch == ';' {
            while i < bytes.len() && bytes[i] as char != '\n' {
                out.push(bytes[i] as char);
                i += 1;
            }
            continue;
        }
        if !in_string && ch == '@' && bytes.get(i + 1) == Some(&b'{') {
            let rest = &source[i + 2..];
            let Some(end) = rest.find('}') else {
                return Err("unterminated @{...} marker".to_string());
            };
            let inner = &rest[..end];
            let Some((unit, name)) = inner.split_once("::") else {
                return Err(format!("invalid marker '@{{{inner}}}'"));
            };
            let Some(value) = subst.get(&(unit.to_string(), name.to_string())) else {
                return Err(format!("unresolved marker '@{{{inner}}}'"));
            };
            out.push_str(value);
            i += 2 + end + 1;
            continue;
        }
        out.push(ch);
        i += 1;
    }
    Ok(out)
}

fn compile_basic_unit(
    spec: &BasicSpec,
    source: Option<&str>,
    subst: &HashMap<(String, String), String>,
    assembled: &AssembledProgram,
    span: SourceSpan,
) -> Result<CompiledBasic, Diagnostic> {
    let Some(source) = source else {
        return Err(Diagnostic::error(
            DiagCode::LnMissingOrigin,
            span,
            "manifest [basic] is present but no BASIC source was supplied",
        ));
    };
    let numeric = numeric_subst(subst, assembled);
    let substituted = substitute_basic(source, &numeric)
        .map_err(|msg| Diagnostic::error(DiagCode::TyUnresolvedName, span, msg))?;
    let payload = crate::basic::tokenize_program(&substituted).map_err(|err| {
        Diagnostic::error(
            DiagCode::ParseExpected,
            span,
            format!("BASIC tokenize failed: {err}"),
        )
    })?;
    let payload_len = payload.len() as u32;
    if payload_len + 0x100 > u32::from(spec.size) {
        return Err(Diagnostic::error(
            DiagCode::LnOverlap,
            span,
            format!(
                "tokenized BASIC is {} bytes; payload+0x100 must fit in basic.size {}",
                payload.len(),
                spec.size
            ),
        ));
    }
    Ok(CompiledBasic {
        origin: spec.origin,
        region_size: spec.size,
        payload,
        substituted_source: substituted,
    })
}

fn numeric_subst(
    subst: &HashMap<(String, String), String>,
    assembled: &AssembledProgram,
) -> HashMap<(String, String), String> {
    let mut out = HashMap::new();
    for ((unit, name), value) in subst {
        let rendered = if value.bytes().all(|b| b.is_ascii_digit()) {
            value.clone()
        } else if let Some(&addr) = assembled.symbols.get(value) {
            u32::from(addr).to_string()
        } else {
            assembled
                .symbols
                .iter()
                .find(|(label, _)| label.eq_ignore_ascii_case(value))
                .map(|(_, addr)| u32::from(*addr).to_string())
                .unwrap_or_else(|| value.clone())
        };
        out.insert((unit.clone(), name.clone()), rendered);
    }
    out
}

fn substitute_basic(
    source: &str,
    subst: &HashMap<(String, String), String>,
) -> Result<String, String> {
    let modes = crate::basic::basic_copy_modes(source);
    let bytes = source.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let active = modes.get(i).copied() == Some(crate::basic::BasicCopyMode::Normal);
        if active && bytes[i] == b'@' && bytes.get(i + 1) == Some(&b'{') {
            let rest = &source[i + 2..];
            let Some(end) = rest.find('}') else {
                return Err("unterminated @{...} marker".to_string());
            };
            let inner = &rest[..end];
            let Some((unit, name)) = inner.split_once("::") else {
                return Err(format!("invalid marker '@{{{inner}}}'"));
            };
            let Some(value) = subst.get(&(unit.to_string(), name.to_string())) else {
                return Err(format!("unresolved marker '@{{{inner}}}'"));
            };
            if name == "basic_program_size" {
                return Err(
                    "project::basic_program_size is not available during BASIC substitution"
                        .to_string(),
                );
            }
            out.push_str(value);
            i += 2 + end + 1;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    Ok(out)
}

fn namespace_asm_labels(source: &str, file_id: u32) -> String {
    let mut labels = Vec::new();
    for raw in source.lines() {
        let line = strip_asm_comment(raw);
        let mut rest = line.trim();
        while let Some(colon) = rest.find(':') {
            let before = rest[..colon].trim();
            if before.is_empty() || before.contains(char::is_whitespace) || !is_asm_label(before) {
                break;
            }
            labels.push(before.to_string());
            rest = rest[colon + 1..].trim_start();
        }
    }
    let mut out_lines = Vec::new();
    for raw in source.split_inclusive('\n') {
        let (line, nl) = raw
            .strip_suffix('\n')
            .map(|l| (l, "\n"))
            .unwrap_or((raw, ""));
        out_lines.push(format!(
            "{}{nl}",
            rewrite_line_labels(line, file_id, &labels)
        ));
    }
    out_lines.concat()
}

fn rewrite_line_labels(line: &str, file_id: u32, labels: &[String]) -> String {
    let (code, comment) = match line.find(';') {
        Some(idx) if !in_string_at(line, idx) => (&line[..idx], &line[idx..]),
        _ => (line, ""),
    };
    let mut out = String::new();
    let mut i = 0;
    let chars: Vec<char> = code.chars().collect();
    let mut in_string = false;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' {
            in_string = !in_string;
            out.push(ch);
            i += 1;
            continue;
        }
        if !in_string && (ch.is_ascii_alphabetic() || ch == '_' || ch == '.') {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.')
            {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if let Some(label) = labels.iter().find(|l| l.eq_ignore_ascii_case(&word)) {
                out.push_str(&format!("U{file_id}_{}", label.to_ascii_uppercase()));
            } else {
                out.push_str(&word);
            }
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out.push_str(comment);
    out
}

fn in_string_at(line: &str, idx: usize) -> bool {
    let mut in_string = false;
    for (i, ch) in line.char_indices() {
        if i >= idx {
            break;
        }
        if ch == '"' {
            in_string = !in_string;
        }
    }
    in_string
}

fn strip_asm_comment(line: &str) -> String {
    match line.find(';') {
        Some(idx) if !in_string_at(line, idx) => line[..idx].to_string(),
        _ => line.to_string(),
    }
}

fn mnemonic_is(line: &str, mnem: &str) -> bool {
    let rest = line.split(':').last().unwrap_or(line).trim();
    rest.split_whitespace()
        .next()
        .is_some_and(|w| w.eq_ignore_ascii_case(mnem))
}

fn is_asm_label(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '.')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

fn asm_emits_bytes(source: &str) -> bool {
    for raw in source.lines() {
        let line = strip_asm_comment(raw);
        let mut rest = line.trim();
        while let Some(colon) = rest.find(':') {
            let before = rest[..colon].trim();
            if before.is_empty() || before.contains(char::is_whitespace) || !is_asm_label(before) {
                break;
            }
            rest = rest[colon + 1..].trim_start();
        }
        if rest.is_empty() {
            continue;
        }
        let mnem = rest.split_whitespace().next().unwrap_or("");
        if mnem.eq_ignore_ascii_case("EQU") || mnem.eq_ignore_ascii_case("ORG") {
            continue;
        }
        return true;
    }
    false
}

struct LayoutRange {
    name: String,
    start: u32,
    end: u32,
}

fn check_layout_ranges(
    manifest: &Manifest,
    assembled: &AssembledProgram,
    span: SourceSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut ranges = Vec::new();
    for (i, seg) in assembled.segments.iter().enumerate() {
        let start = u32::from(seg.addr);
        let end = start + seg.bytes.len() as u32;
        push_range(
            &mut ranges,
            LayoutRange {
                name: format!("segment{i}"),
                start,
                end,
            },
            span,
            diagnostics,
        );
    }
    if let Some(stack) = &manifest.stack {
        push_range(
            &mut ranges,
            LayoutRange {
                name: "stack".into(),
                start: u32::from(stack.base),
                end: u32::from(stack.base) + u32::from(stack.size),
            },
            span,
            diagnostics,
        );
    }
    for r in &manifest.reserves {
        push_range(
            &mut ranges,
            LayoutRange {
                name: format!("reserve {}", r.name),
                start: u32::from(r.base),
                end: u32::from(r.base) + u32::from(r.size),
            },
            span,
            diagnostics,
        );
    }
    if let Some(basic) = &manifest.basic {
        push_range(
            &mut ranges,
            LayoutRange {
                name: "basic".into(),
                start: u32::from(basic.origin),
                end: u32::from(basic.origin) + u32::from(basic.size),
            },
            span,
            diagnostics,
        );
    }
}

fn push_range(
    ranges: &mut Vec<LayoutRange>,
    new: LayoutRange,
    span: SourceSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if new.end > 65536 {
        diagnostics.push(Diagnostic::error(
            DiagCode::LnOverlap,
            span,
            format!("{} overflows the 16-bit address space", new.name),
        ));
        return;
    }
    for existing in ranges.iter() {
        if new.start < existing.end && existing.start < new.end {
            diagnostics.push(Diagnostic::error(
                DiagCode::LnOverlap,
                span,
                format!("{} overlaps {}", new.name, existing.name),
            ));
            return;
        }
    }
    ranges.push(new);
}

fn check_stack_budget(
    manifest: &Manifest,
    c80_stack: u16,
    units: &[ProjectUnitInput<'_>],
    span: SourceSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(stack) = &manifest.stack else {
        return;
    };
    let mut extra = 0u32;
    let mut unknown = false;
    for unit in units {
        if unit.kind != UnitKind::Asm {
            continue;
        }
        match unit.stack_extra {
            Some(v) => extra = extra.max(u32::from(v)),
            None if asm_emits_bytes(unit.text) => unknown = true,
            None => {}
        }
    }
    let interrupt = stack.interrupt_allowance.map(u32::from).unwrap_or(0);
    let needed = u32::from(c80_stack) + extra + interrupt;
    if unknown {
        diagnostics.push(Diagnostic::warning(
            DiagCode::LnStack,
            span,
            "stack usage includes unknown assembly bounds",
        ));
    }
    if needed > u32::from(stack.size) {
        diagnostics.push(Diagnostic::error(
            DiagCode::LnStack,
            span,
            format!(
                "known stack use {needed} exceeds reserved stack {}",
                stack.size
            ),
        ));
    }
}

fn check_symbol_collisions(
    assembled: &AssembledProgram,
    span: SourceSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut folded: HashMap<String, String> = HashMap::new();
    for name in assembled.symbols.keys() {
        let key = name.to_ascii_uppercase();
        if let Some(prev) = folded.insert(key, name.clone()) {
            if prev != *name {
                diagnostics.push(Diagnostic::error(
                    DiagCode::TyDuplicateName,
                    span,
                    format!("assembled label collision between '{prev}' and '{name}'"),
                ));
            }
        }
    }
}

pub fn resolve_manifest_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

pub fn render_rtvc_asm_v1(
    source: &str,
    requested_origin: u16,
    assembled: &AssembledProgram,
) -> String {
    let mut out = Vec::new();
    out.push(format!("format = \"rtvc-asm-v1\""));
    out.push(format!("source = {}", toml_string(source)));
    out.push(format!("requested_origin = {}", hex16(requested_origin)));
    out.push(format!("origin = {}", hex16(assembled.origin)));
    out.push(format!("next_addr = {}", hex16(assembled.next_addr)));
    if !assembled.symbols.is_empty() {
        out.push(String::new());
        out.push("[symbols]".to_string());
        for (name, value) in &assembled.symbols {
            out.push(format!("{name} = {}", hex16(*value)));
        }
    }
    for segment in &assembled.segments {
        out.push(String::new());
        out.push("[[segments]]".to_string());
        out.push(format!("addr = {}", hex16(segment.addr)));
        out.push(format!("len = {}", segment.bytes.len()));
        out.push("bytes = [".to_string());
        for row in segment.bytes.chunks(16) {
            let items = row
                .iter()
                .map(|b| format!("0x{b:02X}"))
                .collect::<Vec<_>>()
                .join(", ");
            out.push(format!("  {items},"));
        }
        out.push("]".to_string());
    }
    out.join("\n") + "\n"
}

pub fn contiguous_bytes(code: &GeneratedProgram) -> Result<Vec<u8>, String> {
    let mut segs = code.assembled.segments.clone();
    segs.sort_by_key(|s| s.addr);
    if segs.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = segs[0].bytes.clone();
    let mut next = segs[0].addr.wrapping_add(segs[0].bytes.len() as u16);
    for seg in segs.iter().skip(1) {
        if seg.addr != next {
            return Err("raw binary requires a contiguous union of emitted ranges".to_string());
        }
        out.extend_from_slice(&seg.bytes);
        next = next.wrapping_add(seg.bytes.len() as u16);
    }
    Ok(out)
}

pub fn write_atomic(files: &[(PathBuf, Vec<u8>)]) -> Result<(), String> {
    let mut tmps = Vec::new();
    let mut replaced = Vec::new();
    for (path, bytes) in files {
        let tmp = sibling_tmp(path);
        if let Err(err) = fs::write(&tmp, bytes) {
            for existing in &tmps {
                let _ = fs::remove_file(existing);
            }
            return Err(format!("cannot write {}: {err}", tmp.display()));
        }
        tmps.push(tmp);
    }
    for ((path, _), tmp) in files.iter().zip(tmps.iter()) {
        if let Err(err) = fs::rename(tmp, path) {
            let _ = fs::remove_file(tmp);
            let already = if replaced.is_empty() {
                String::new()
            } else {
                format!("; already replaced {}", replaced.join(", "))
            };
            return Err(format!("cannot replace {}: {err}{already}", path.display()));
        }
        replaced.push(path.display().to_string());
    }
    Ok(())
}

fn sibling_tmp(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_else(|| "out".into());
    name.push(".tmp");
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(name),
        _ => PathBuf::from(name),
    }
}

fn toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn hex16(value: u16) -> String {
    format!("0x{value:04X}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::CpuBus;
    use crate::compiler::harness::execute_function;

    fn compile_units(
        units: &[ProjectUnitInput<'_>],
        stack: Option<StackSpec>,
        reserves: Vec<ReserveSpec>,
    ) -> crate::compiler::CompilationResult {
        let specs = units
            .iter()
            .map(|u| UnitSpec {
                name: u.name.to_string(),
                path: PathBuf::from(u.source_name),
                kind: u.kind,
                origin: u.origin,
                exports: u.exports.to_vec(),
                stack_extra: u.stack_extra,
            })
            .collect();
        compile_project(
            &Manifest {
                version: 1,
                target: ProjectTarget::GenericZ80,
                entry: None,
                stack,
                reserves,
                units: specs,
                basic: None,
            },
            units,
            None,
        )
    }

    fn compile_tvc(
        units: &[ProjectUnitInput<'_>],
        basic: BasicSpec,
        basic_src: &str,
    ) -> crate::compiler::CompilationResult {
        let specs = units
            .iter()
            .map(|u| UnitSpec {
                name: u.name.to_string(),
                path: PathBuf::from(u.source_name),
                kind: u.kind,
                origin: u.origin,
                exports: u.exports.to_vec(),
                stack_extra: u.stack_extra,
            })
            .collect();
        compile_project(
            &Manifest {
                version: 1,
                target: ProjectTarget::Tvc,
                entry: None,
                stack: None,
                reserves: Vec::new(),
                units: specs,
                basic: Some(basic),
            },
            units,
            Some(basic_src),
        )
    }

    fn codes(result: &crate::compiler::CompilationResult) -> Vec<&'static str> {
        result
            .diagnostics
            .iter()
            .filter(|d| d.is_error())
            .map(|d| d.code.as_str())
            .collect()
    }

    #[test]
    fn parse_rejects_unknown_fields_and_duplicate_units() {
        assert!(
            parse_manifest("version = 1\nnope = 1\n[[unit]]\nname=\"a\"\npath=\"a.c80\"\n")
                .is_err()
        );
        assert!(
            parse_manifest(
                "[[unit]]\nname=\"a\"\npath=\"a.c80\"\n[[unit]]\nname=\"a\"\npath=\"b.c80\"\n"
            )
            .is_err()
        );
        assert!(parse_manifest("[[unit]]\nname=\"project\"\npath=\"a.c80\"\n").is_err());
        assert!(parse_manifest("version = 2\n[[unit]]\nname=\"a\"\npath=\"a.c80\"\n").is_err());
        assert!(
            parse_manifest(
                "target=\"generic-z80\"\n[basic]\npath=\"a.bas\"\norigin=0x4000\nsize=0x8000\n[[unit]]\nname=\"a\"\npath=\"a.c80\"\n"
            )
            .is_err()
        );
        let ok = parse_manifest(
            "target=\"tvc\"\n[basic]\npath=\"src/main.bas\"\norigin=0x4000\nsize=0x8000\n[[unit]]\nname=\"math\"\npath=\"echo.c80\"\norigin=0x3000\n",
        )
        .unwrap();
        let basic = ok.basic.unwrap();
        assert_eq!(basic.origin, 0x4000);
        assert_eq!(basic.size, 0x8000);
    }

    #[test]
    fn private_symbol_is_not_visible_across_units() {
        let lib = "u8 hidden = 3;\npub u8 visible = 4;\n";
        let main = "import lib;\nu8 f() { return lib::hidden; }\n";
        let result = compile_units(
            &[
                ProjectUnitInput {
                    name: "lib",
                    kind: UnitKind::C80,
                    origin: Some(0x8100),
                    text: lib,
                    source_name: "lib.c80",
                    exports: &[],
                    stack_extra: None,
                },
                ProjectUnitInput {
                    name: "main",
                    kind: UnitKind::C80,
                    origin: Some(0x8000),
                    text: main,
                    source_name: "main.c80",
                    exports: &[],
                    stack_extra: None,
                },
            ],
            None,
            Vec::new(),
        );
        assert!(
            codes(&result).contains(&"ty-unresolved-name"),
            "{:?}",
            result.diagnostics
        );
    }

    #[test]
    fn public_lookup_and_constants_only_unit_emit_no_bytes() {
        let consts = "pub const u16 WIDTH = 4;\n";
        let main = "import data;\nu16 f() { return data::WIDTH; }\n";
        let result = compile_units(
            &[
                ProjectUnitInput {
                    name: "data",
                    kind: UnitKind::C80,
                    origin: None,
                    text: consts,
                    source_name: "data.c80",
                    exports: &[],
                    stack_extra: None,
                },
                ProjectUnitInput {
                    name: "main",
                    kind: UnitKind::C80,
                    origin: Some(0x8000),
                    text: main,
                    source_name: "main.c80",
                    exports: &[],
                    stack_extra: None,
                },
            ],
            None,
            Vec::new(),
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let code = result.code.as_ref().unwrap();
        assert!(code.global("WIDTH").is_none());
        let exec = execute_function(code, "f", &[]).unwrap();
        assert_eq!(exec.return_word(), 4);
        assert_eq!(code.assembled.segments.len(), 1);
        assert_eq!(code.assembled.segments[0].addr, 0x8000);
    }

    #[test]
    fn missing_import_unit_is_an_error() {
        let main = "import nope;\nvoid f() { nope::g(); }\n";
        let result = compile_units(
            &[ProjectUnitInput {
                name: "main",
                kind: UnitKind::C80,
                origin: Some(0x8000),
                text: main,
                source_name: "main.c80",
                exports: &[],
                stack_extra: None,
            }],
            None,
            Vec::new(),
        );
        assert!(codes(&result).contains(&"ty-unresolved-name"));
    }

    #[test]
    fn cross_unit_call_and_callee_growth_updates_caller() {
        let caller = "import lib;\nu16 f() { return lib::g(); }\n";
        let small = "pub u16 g() { return 1; }\n";
        let large = "u8 pad[8];\npub u16 g() { return 1; }\n";
        let small_res = compile_units(
            &[
                ProjectUnitInput {
                    name: "main",
                    kind: UnitKind::C80,
                    origin: Some(0x8000),
                    text: caller,
                    source_name: "main.c80",
                    exports: &[],
                    stack_extra: None,
                },
                ProjectUnitInput {
                    name: "lib",
                    kind: UnitKind::C80,
                    origin: Some(0x8100),
                    text: small,
                    source_name: "lib.c80",
                    exports: &[],
                    stack_extra: None,
                },
            ],
            None,
            Vec::new(),
        );
        let large_res = compile_units(
            &[
                ProjectUnitInput {
                    name: "main",
                    kind: UnitKind::C80,
                    origin: Some(0x8000),
                    text: caller,
                    source_name: "main.c80",
                    exports: &[],
                    stack_extra: None,
                },
                ProjectUnitInput {
                    name: "lib",
                    kind: UnitKind::C80,
                    origin: Some(0x8100),
                    text: large,
                    source_name: "lib.c80",
                    exports: &[],
                    stack_extra: None,
                },
            ],
            None,
            Vec::new(),
        );
        assert!(!small_res.has_errors(), "{:?}", small_res.diagnostics);
        assert!(!large_res.has_errors(), "{:?}", large_res.diagnostics);
        let small_g = small_res.code.as_ref().unwrap().function("g").unwrap().addr;
        let large_g = large_res.code.as_ref().unwrap().function("g").unwrap().addr;
        assert_ne!(small_g, large_g);
        let small_bytes = small_res
            .code
            .as_ref()
            .unwrap()
            .function_bytes("f")
            .unwrap();
        let large_bytes = large_res
            .code
            .as_ref()
            .unwrap()
            .function_bytes("f")
            .unwrap();
        assert_eq!(small_bytes[0], 0xCD);
        assert_eq!(large_bytes[0], 0xCD);
        let small_target = u16::from(small_bytes[1]) | (u16::from(small_bytes[2]) << 8);
        let large_target = u16::from(large_bytes[1]) | (u16::from(large_bytes[2]) << 8);
        assert_eq!(small_target, small_g);
        assert_eq!(large_target, large_g);
        assert_eq!(
            execute_function(small_res.code.as_ref().unwrap(), "f", &[])
                .unwrap()
                .return_word(),
            1
        );
    }

    #[test]
    fn cross_unit_recursion_is_rejected() {
        let a = "import b;\npub void fa() { b::fb(); }\n";
        let b = "import a;\npub void fb() { a::fa(); }\n";
        let result = compile_units(
            &[
                ProjectUnitInput {
                    name: "a",
                    kind: UnitKind::C80,
                    origin: Some(0x8000),
                    text: a,
                    source_name: "a.c80",
                    exports: &[],
                    stack_extra: None,
                },
                ProjectUnitInput {
                    name: "b",
                    kind: UnitKind::C80,
                    origin: Some(0x8100),
                    text: b,
                    source_name: "b.c80",
                    exports: &[],
                    stack_extra: None,
                },
            ],
            None,
            Vec::new(),
        );
        assert!(
            codes(&result).contains(&"ty-recursion"),
            "{:?}",
            result.diagnostics
        );
    }

    #[test]
    fn adjacent_ranges_are_legal_and_overlap_is_not() {
        let src = "void f() { return; }\n";
        let ok = compile_units(
            &[
                ProjectUnitInput {
                    name: "a",
                    kind: UnitKind::C80,
                    origin: Some(0x8000),
                    text: src,
                    source_name: "a.c80",
                    exports: &[],
                    stack_extra: None,
                },
                ProjectUnitInput {
                    name: "b",
                    kind: UnitKind::C80,
                    origin: Some(0x8001),
                    text: src,
                    source_name: "b.c80",
                    exports: &[],
                    stack_extra: None,
                },
            ],
            None,
            vec![ReserveSpec {
                name: "tail".into(),
                base: 0x8002,
                size: 0x0002,
            }],
        );
        assert!(!ok.has_errors(), "{:?}", ok.diagnostics);
        let overlap = compile_units(
            &[ProjectUnitInput {
                name: "a",
                kind: UnitKind::C80,
                origin: Some(0x8000),
                text: src,
                source_name: "a.c80",
                exports: &[],
                stack_extra: None,
            }],
            None,
            vec![ReserveSpec {
                name: "hit".into(),
                base: 0x8000,
                size: 1,
            }],
        );
        assert!(
            codes(&overlap).contains(&"ln-overlap"),
            "{:?}",
            overlap.diagnostics
        );
    }

    #[test]
    fn reservation_emits_no_bytes() {
        let src = "void f() { return; }\n";
        let result = compile_units(
            &[ProjectUnitInput {
                name: "main",
                kind: UnitKind::C80,
                origin: Some(0x2200),
                text: src,
                source_name: "main.c80",
                exports: &[],
                stack_extra: None,
            }],
            None,
            vec![ReserveSpec {
                name: "screen".into(),
                base: 0x8000,
                size: 0x3800,
            }],
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let code = result.code.as_ref().unwrap();
        assert!(!code.assembled.segments.iter().any(|s| s.addr == 0x8000));
    }

    #[test]
    fn stack_end_at_10000h_is_not_a_u16_and_top_wraps() {
        let src =
            "u16 top() { return project::stack_top; }\nu16 end() { return project::stack_end; }\n";
        let result = compile_units(
            &[ProjectUnitInput {
                name: "main",
                kind: UnitKind::C80,
                origin: Some(0x8000),
                text: src,
                source_name: "main.c80",
                exports: &[],
                stack_extra: None,
            }],
            Some(StackSpec {
                base: 0xF800,
                size: 0x0800,
                interrupt_allowance: Some(0),
            }),
            Vec::new(),
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("stack_end")),
            "{:?}",
            result.diagnostics
        );
    }

    #[test]
    fn asm_entry_calls_c80_and_returns_to_continuation() {
        let c80 = "pub u8 inc(u8 x) { return x + 1; }\n";
        let asm = "start:\n    LD A, 7\n    CALL @{main::inc}\ncont:\n    RET\n";
        let exports = ["start".to_string()];
        let result = compile_units(
            &[
                ProjectUnitInput {
                    name: "boot",
                    kind: UnitKind::Asm,
                    origin: Some(0x2000),
                    text: asm,
                    source_name: "boot.asm",
                    exports: &exports,
                    stack_extra: Some(8),
                },
                ProjectUnitInput {
                    name: "main",
                    kind: UnitKind::C80,
                    origin: Some(0x2200),
                    text: c80,
                    source_name: "main.c80",
                    exports: &[],
                    stack_extra: None,
                },
            ],
            None,
            Vec::new(),
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let code = result.code.as_ref().unwrap();
        assert!(code.assembly.contains("CALL "));
        assert!(!code.assembly.contains("@{"));
        let exec = execute_function(code, "start", &[]).unwrap();
        assert!(!exec.timed_out);
        assert_eq!(exec.return_byte(), 8);
    }

    #[test]
    fn stack_top_sp_encoding_uses_wrapped_zero() {
        let asm = "start:\n    LD SP, @{project::stack_top}\n    RET\n";
        let exports = ["start".to_string()];
        let result = compile_units(
            &[ProjectUnitInput {
                name: "boot",
                kind: UnitKind::Asm,
                origin: Some(0x2000),
                text: asm,
                source_name: "boot.asm",
                exports: &exports,
                stack_extra: Some(0),
            }],
            Some(StackSpec {
                base: 0xF800,
                size: 0x0800,
                interrupt_allowance: Some(0),
            }),
            Vec::new(),
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let bytes = &result.code.as_ref().unwrap().assembled.segments[0].bytes;
        assert_eq!(bytes, &[0x31, 0x00, 0x00, 0xC9]);
    }

    #[test]
    fn asm_case_collision_is_rejected() {
        let asm = "foo:\n    RET\nFOO:\n    RET\n";
        let result = compile_units(
            &[ProjectUnitInput {
                name: "boot",
                kind: UnitKind::Asm,
                origin: Some(0x2000),
                text: asm,
                source_name: "boot.asm",
                exports: &[],
                stack_extra: None,
            }],
            None,
            Vec::new(),
        );
        assert!(
            codes(&result).contains(&"ty-duplicate-name"),
            "{:?}",
            result.diagnostics
        );
    }

    #[test]
    fn missing_origin_is_rejected_when_bytes_are_emitted() {
        let src = "void f() { return; }\n";
        let result = compile_units(
            &[ProjectUnitInput {
                name: "main",
                kind: UnitKind::C80,
                origin: None,
                text: src,
                source_name: "main.c80",
                exports: &[],
                stack_extra: None,
            }],
            None,
            Vec::new(),
        );
        assert!(codes(&result).contains(&"ln-missing-origin"));
    }

    #[test]
    fn separated_ranges_reject_raw_binary() {
        let src = "void f() { return; }\n";
        let result = compile_units(
            &[
                ProjectUnitInput {
                    name: "a",
                    kind: UnitKind::C80,
                    origin: Some(0x2000),
                    text: src,
                    source_name: "a.c80",
                    exports: &[],
                    stack_extra: None,
                },
                ProjectUnitInput {
                    name: "b",
                    kind: UnitKind::C80,
                    origin: Some(0x8000),
                    text: src,
                    source_name: "b.c80",
                    exports: &[],
                    stack_extra: None,
                },
            ],
            None,
            Vec::new(),
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(contiguous_bytes(result.code.as_ref().unwrap()).is_err());
    }

    #[test]
    fn toml_segments_round_trip() {
        let src = "void f() { return; }\n";
        let result = compile_units(
            &[ProjectUnitInput {
                name: "main",
                kind: UnitKind::C80,
                origin: Some(0x8000),
                text: src,
                source_name: "main.c80",
                exports: &[],
                stack_extra: None,
            }],
            None,
            Vec::new(),
        );
        let text = render_rtvc_asm_v1("main.c80", 0x8000, &result.code.as_ref().unwrap().assembled);
        let value: toml::Value = toml::from_str(&text).unwrap();
        assert_eq!(
            value.get("format").and_then(|v| v.as_str()),
            Some("rtvc-asm-v1")
        );
        let segs = value.get("segments").and_then(|v| v.as_array()).unwrap();
        assert_eq!(
            segs[0].get("addr").and_then(|v| v.as_integer()),
            Some(0x8000)
        );
        let bytes = segs[0].get("bytes").and_then(|v| v.as_array()).unwrap();
        assert_eq!(bytes[0].as_integer(), Some(0xC9));
    }

    #[test]
    fn relative_manifest_paths_are_from_the_manifest_directory() {
        let resolved = resolve_manifest_path(Path::new("/proj/src"), Path::new("units/a.c80"));
        assert_eq!(resolved, PathBuf::from("/proj/src/units/a.c80"));
    }

    #[test]
    fn write_atomic_leaves_existing_file_on_failure() {
        let dir = std::env::temp_dir().join(format!("rtvc-c80-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let good = dir.join("out.bin");
        fs::write(&good, b"keep").unwrap();
        let missing = dir.join("nope").join("out.bin");
        let err = write_atomic(&[(missing, b"x".to_vec()), (good.clone(), b"new".to_vec())]);
        assert!(err.is_err());
        assert_eq!(fs::read(&good).unwrap(), b"keep");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn stack_top_is_zero_when_the_range_ends_at_10000h() {
        let src = "u16 top() { return project::stack_top; }\n";
        let result = compile_units(
            &[ProjectUnitInput {
                name: "main",
                kind: UnitKind::C80,
                origin: Some(0x8000),
                text: src,
                source_name: "main.c80",
                exports: &[],
                stack_extra: None,
            }],
            Some(StackSpec {
                base: 0xF800,
                size: 0x0800,
                interrupt_allowance: Some(0),
            }),
            Vec::new(),
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert_eq!(
            execute_function(result.code.as_ref().unwrap(), "top", &[])
                .unwrap()
                .return_word(),
            0
        );
    }

    fn math_unit(src: &str) -> ProjectUnitInput<'_> {
        ProjectUnitInput {
            name: "math",
            kind: UnitKind::C80,
            origin: Some(0x3000),
            text: src,
            source_name: "echo.c80",
            exports: &[],
            stack_extra: None,
        }
    }

    fn fixture_basic() -> BasicSpec {
        BasicSpec {
            path: PathBuf::from("main.bas"),
            origin: 0x4000,
            size: 0x8000,
        }
    }

    #[test]
    fn basic_markers_resolve_after_code_growth_and_respect_boundaries() {
        let small = "@fastcall pub i16 echo(i16 value) { return value; }\n";
        let large = "u8 pad[16];\n@fastcall pub i16 echo(i16 value) { return value; }\n";
        let bas = r#"
10 PRINT "@{math::echo}"
20 REM @{math::echo}
30 DATA @{math::echo}:LET R=USR(@{math::echo},42)
"#;
        let small_res = compile_tvc(&[math_unit(small)], fixture_basic(), bas);
        let large_res = compile_tvc(&[math_unit(large)], fixture_basic(), bas);
        assert!(!small_res.has_errors(), "{:?}", small_res.diagnostics);
        assert!(!large_res.has_errors(), "{:?}", large_res.diagnostics);
        let small_addr = small_res
            .code
            .as_ref()
            .unwrap()
            .function("echo")
            .unwrap()
            .addr;
        let large_addr = large_res
            .code
            .as_ref()
            .unwrap()
            .function("echo")
            .unwrap()
            .addr;
        assert_ne!(small_addr, large_addr);
        let small_src = &small_res.basic.as_ref().unwrap().substituted_source;
        let large_src = &large_res.basic.as_ref().unwrap().substituted_source;
        assert!(
            small_src.contains(&format!("PRINT \"@{{math::echo}}\"")),
            "{small_src}"
        );
        assert!(small_src.contains("REM @{math::echo}"), "{small_src}");
        assert!(small_src.contains("DATA @{math::echo}:"), "{small_src}");
        assert!(
            small_src.contains(&format!("USR({small_addr},42)")),
            "{small_src}"
        );
        assert!(
            large_src.contains(&format!("USR({large_addr},42)")),
            "{large_src}"
        );
        assert!(!small_src.contains("USR(@{"), "{small_src}");
        let payload = &small_res.basic.as_ref().unwrap().payload;
        assert_eq!(*payload.last().unwrap(), 0x00);
        assert!(payload.len() as u32 + 0x100 <= 0x8000);
    }

    #[test]
    fn missing_basic_symbol_and_c80_program_size_are_errors() {
        let src = "@fastcall pub i16 echo(i16 value) { return value; }\n";
        let missing = compile_tvc(
            &[math_unit(src)],
            fixture_basic(),
            "10 LET R=USR(@{math::nope},1)\n",
        );
        assert!(
            codes(&missing).contains(&"ty-unresolved-name"),
            "{:?}",
            missing.diagnostics
        );
        let c80 = "u16 n() { return project::basic_program_size; }\n";
        let result = compile_tvc(&[math_unit(c80)], fixture_basic(), "10 PRINT 1\n");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("basic_program_size")),
            "{:?}",
            result.diagnostics
        );
    }

    #[test]
    fn basic_region_overlap_and_payload_gap_are_rejected() {
        let src = "@fastcall pub i16 echo(i16 value) { return value; }\n";
        let overlap = compile_tvc(
            &[ProjectUnitInput {
                name: "math",
                kind: UnitKind::C80,
                origin: Some(0x4100),
                text: src,
                source_name: "echo.c80",
                exports: &[],
                stack_extra: None,
            }],
            fixture_basic(),
            "10 PRINT 1\n",
        );
        assert!(
            codes(&overlap).contains(&"ln-overlap"),
            "{:?}",
            overlap.diagnostics
        );
        let tiny = compile_tvc(
            &[math_unit(src)],
            BasicSpec {
                path: PathBuf::from("main.bas"),
                origin: 0x4000,
                size: 0x20,
            },
            "10 PRINT 1\n",
        );
        assert!(
            codes(&tiny).contains(&"ln-overlap"),
            "{:?}",
            tiny.diagnostics
        );
    }

    #[test]
    fn basic_usr_echo_runs_on_tvc_12_after_lomem() {
        let src = "@fastcall pub i16 echo(i16 value) { return value; }\n";
        let bas = "\
10 LET R=USR(@{math::echo},42)\n\
20 IF R=42 THEN POKE 16383,1\n\
30 LET R=USR(@{math::echo},-1)\n\
40 IF R=-1 THEN POKE 16383,PEEK(16383)+1\n\
50 LET R=USR(@{math::echo},32767)\n\
60 IF R=32767 THEN POKE 16383,PEEK(16383)+1\n\
70 LET R=USR(@{math::echo},-32767)\n\
80 IF R=-32767 THEN POKE 16383,PEEK(16383)+1\n\
90 POKE 16382,1\n";
        let result = compile_tvc(&[math_unit(src)], fixture_basic(), bas);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let code = result.code.as_ref().unwrap();
        let basic = result.basic.as_ref().unwrap();
        let echo_addr = code.function("echo").unwrap().addr;
        assert_eq!(echo_addr, 0x3000);
        assert!(
            basic.substituted_source.contains("USR(12288,42)"),
            "{}",
            basic.substituted_source
        );

        let mut tvc = boot_tvc12();
        wait_until(&mut tvc, 400, |tvc| peek16(tvc, 0x1722) == 0x19EF);
        for _ in 0..120 {
            tvc.run_for_a_frame();
        }
        assert_eq!(peek16(&mut tvc, 0x0B19), 0xBFFF, "HIMEM");
        type_line(&mut tvc, "LOMEM 16384");
        wait_until(&mut tvc, 600, |tvc| peek16(tvc, 0x1722) == 0x4000);
        assert_eq!(peek16(&mut tvc, 0x1722), 0x4000, "TEXT after LOMEM");
        assert_eq!(peek16(&mut tvc, 0x0B19), 0xBFFF, "HIMEM after LOMEM");

        let text = peek16(&mut tvc, 0x1722);
        let old_top = peek16(&mut tvc, 0x1726);
        let old_chain = peek16(&mut tvc, 0x1724);
        for (i, byte) in basic.payload.iter().enumerate() {
            tvc.bus.w8(text.wrapping_add(i as u16), *byte);
        }
        let extra = (basic.payload.len() as u16).saturating_sub(1);
        poke16(&mut tvc, 0x1726, old_top.wrapping_add(extra));
        poke16(&mut tvc, 0x1724, old_chain.wrapping_add(extra));

        let c80_bytes: Vec<(u16, u8)> = code
            .assembled
            .segments
            .iter()
            .flat_map(|seg| {
                seg.bytes
                    .iter()
                    .enumerate()
                    .map(|(i, b)| (seg.addr.wrapping_add(i as u16), *b))
            })
            .collect();
        let before: Vec<_> = c80_bytes
            .iter()
            .map(|(addr, _)| tvc.bus.r8(*addr))
            .collect();
        for (addr, byte) in &c80_bytes {
            tvc.bus.w8(*addr, *byte);
        }
        let after: Vec<_> = c80_bytes
            .iter()
            .map(|(addr, _)| tvc.bus.r8(*addr))
            .collect();
        assert_eq!(
            after,
            c80_bytes.iter().map(|(_, b)| *b).collect::<Vec<_>>(),
            "C80 bytes must land in RAM; map={:02X} before={before:02X?}",
            tvc.bus.mmu.get_map_val()
        );

        type_line(&mut tvc, "RUN");
        wait_until(&mut tvc, 1200, |tvc| {
            tvc.bus.r8(0x3FFF) == 4 && tvc.bus.r8(0x3FFE) == 1
        });
        assert_eq!(tvc.bus.r8(0x3FFF), 4, "USR results");
        assert_eq!(tvc.bus.r8(0x3FFE), 1, "interpreter continued");
        assert_eq!(peek16(&mut tvc, 0x1722), 0x4000, "TEXT after USR");
        let still: Vec<_> = c80_bytes
            .iter()
            .map(|(addr, _)| tvc.bus.r8(*addr))
            .collect();
        assert_eq!(still, after, "C80 bytes after USR");
        let _ = before;
    }

    fn boot_tvc12() -> crate::tvc::Tvc {
        let mut tvc = crate::tvc::Tvc::new_with_vid_model(false, crate::vid::VidModel::Interleaved);
        tvc.add_rom("TVC12_D4.64K", include_bytes!("../../roms/TVC12_D4.64K"));
        tvc.add_rom("TVC12_D3.64K", include_bytes!("../../roms/TVC12_D3.64K"));
        tvc.add_rom("TVC12_D7.64K", include_bytes!("../../roms/TVC12_D7.64K"));
        tvc.set_fast_boot(true);
        tvc.reset();
        tvc
    }

    fn poke16(tvc: &mut crate::tvc::Tvc, addr: u16, value: u16) {
        let bytes = value.to_le_bytes();
        tvc.bus.w8(addr, bytes[0]);
        tvc.bus.w8(addr.wrapping_add(1), bytes[1]);
    }

    fn peek16(tvc: &mut crate::tvc::Tvc, addr: u16) -> u16 {
        u16::from_le_bytes([tvc.bus.r8(addr), tvc.bus.r8(addr.wrapping_add(1))])
    }

    fn wait_until(
        tvc: &mut crate::tvc::Tvc,
        frames: u32,
        mut pred: impl FnMut(&mut crate::tvc::Tvc) -> bool,
    ) {
        for _ in 0..frames {
            tvc.run_for_a_frame();
            if pred(tvc) {
                return;
            }
        }
        let pc = tvc.z80.state.pc;
        let map = tvc.bus.mmu.get_map_val();
        let marker = tvc.bus.r8(0x3FFF);
        panic!(
            "timeout pc={pc:04X} TEXT={:04X} HIMEM={:04X} map={map:02X} m3fff={marker:02X}",
            peek16(tvc, 0x1722),
            peek16(tvc, 0x0B19),
        );
    }

    fn type_line(tvc: &mut crate::tvc::Tvc, text: &str) {
        for ch in text.chars().chain(std::iter::once('\r')) {
            let code = ch as u32;
            tvc.key_down(code);
            if ch != '\r' {
                tvc.key_press(ch);
            }
            for _ in 0..8 {
                tvc.run_for_a_frame();
            }
            tvc.key_up(code);
            for _ in 0..3 {
                tvc.run_for_a_frame();
            }
        }
        for _ in 0..40 {
            tvc.run_for_a_frame();
        }
    }
}
