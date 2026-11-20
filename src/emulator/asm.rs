use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    rc::Rc,
};

const TVC_BASIC_LOAD_ADDR: u16 = 0x19EF;
const TVC_BASIC_USR_ENTRY: u16 = 0x1A30;
/// Upper bound on the source lines `REPT` blocks may expand to.
const MAX_REPT_LINES: usize = 1 << 20;
/// Upper bound on nested `INCLUDE` files.
const MAX_INCLUDE_DEPTH: usize = 32;

/// What a file is read for, so a reader can treat source text differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsmFileKind {
    /// `INCLUDE`: assembly source text.
    Source,
    /// `INCBIN`: raw bytes.
    Binary,
    /// `IMPORT`: `rtvc-asm-v1` TOML.
    Symbols,
}

/// File access for `INCLUDE`, `INCBIN` and `IMPORT`.
///
/// The assembler itself has no file system access, so it also runs in the
/// debugger and in WebAssembly builds; callers that have files provide this.
pub trait AsmFiles {
    /// Read `path` as written in the source file named `from`. Returns the
    /// name to report for the file and its bytes.
    fn read(&self, path: &str, from: &str, kind: AsmFileKind) -> Result<(String, Vec<u8>), String>;
}

/// An assembler error. `line` is the 1-based source line when the error
/// belongs to one; `Display` renders it as `line N: message`, and
/// [`AsmError::display_in`] as `file:N: message` for editors and the CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmError {
    message: String,
    line: Option<usize>,
    /// The included file the line belongs to; `None` for the main source.
    file: Option<String>,
}

impl AsmError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            file: None,
        }
    }

    /// The error text without any line prefix.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The 1-based source line, when the error belongs to one.
    pub fn line(&self) -> Option<usize> {
        self.line
    }

    /// The included file the error belongs to, or `None` for the main source.
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }

    /// Render as `file:line: message`, or `file: message` without a line.
    /// `file` names the main source; an error in an included file names that
    /// file instead.
    pub fn display_in(&self, file: &str) -> String {
        let file = self.file.as_deref().unwrap_or(file);
        match self.line {
            Some(line) => format!("{file}:{line}: {}", self.message),
            None => format!("{file}: {}", self.message),
        }
    }
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.file, self.line) {
            (Some(file), Some(line)) => write!(f, "{file}:{line}: {}", self.message),
            (_, Some(line)) => write!(f, "line {line}: {}", self.message),
            (_, None) => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for AsmError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembledProgram {
    pub origin: u16,
    pub bytes: Vec<u8>,
    pub segments: Vec<AssembledSegment>,
    pub symbols: BTreeMap<String, u16>,
    pub mappings: Vec<AsmMapping>,
    pub lines: Vec<AssembledLine>,
    pub next_addr: u16,
    /// Source file names: index 0 is the main source, then each included
    /// file in the order it was first included. `AssembledLine::file`
    /// indexes this list.
    pub files: Vec<String>,
    /// Addresses marked with `BREAK`, in source order.
    pub breakpoints: Vec<AsmBreakpoint>,
}

/// A `BREAK` mark: a debugger breakpoint at the run address of the next
/// instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmBreakpoint {
    pub addr: u16,
    /// Index into `AssembledProgram::files`; 0 is the main source.
    pub file: usize,
    pub line: usize,
}

/// A named address transformation captured at an `ORG` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmMapping {
    pub name: String,
    pub source_base: u16,
    pub mapped_base: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembledSegment {
    pub addr: u16,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembledLine {
    /// Index into `AssembledProgram::files`; 0 is the main source.
    pub file: usize,
    pub line: usize,
    /// Address the bytes are loaded at.
    pub addr: u16,
    /// Address the bytes run at: `addr` except inside `PHASE`.
    pub run_addr: u16,
    pub len: usize,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedLine {
    line_no: usize,
    /// Index into the program's file list; 0 is the main source.
    file: usize,
    /// Name of an included file, for errors; `None` for the main source.
    file_name: Option<Rc<str>>,
    source: String,
    labels: Vec<String>,
    statement: Statement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Statement {
    Empty,
    Org {
        address: String,
        mappings: Vec<(String, String)>,
    },
    BasicStart,
    Equ {
        label: String,
        expr: String,
    },
    Bytes(Vec<ByteValue>),
    Words(Vec<String>),
    Space {
        count: String,
        fill: Option<String>,
    },
    Instruction(String),
    Rept {
        count: String,
        counter: Option<String>,
    },
    Endr,
    Phase(String),
    Dephase,
    Break,
    /// `INCLUDE`, replaced by the included lines when the source is loaded.
    Include(String),
    /// `INCBIN` before its file is read.
    IncbinFile {
        path: String,
        offset: Option<String>,
        length: Option<String>,
    },
    Incbin {
        data: Rc<[u8]>,
        offset: Option<String>,
        length: Option<String>,
    },
    /// `IMPORT` before its file is read.
    ImportFile {
        path: String,
        prefix: Option<String>,
    },
    Import(Rc<[(String, u16)]>),
    Assert {
        condition: String,
        message: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ByteValue {
    Expr(String),
    String(Vec<u8>),
}

/// Assemble one Z80 instruction at `pc`.
///
/// The program counter is needed for the relative displacement used by JR and
/// DJNZ. Labels, expressions, macros, and multi-line source are intentionally
/// outside this small assembler's scope.
pub fn assemble_line(source: &str, pc: u16) -> Result<Vec<u8>, AsmError> {
    let source = source.split(';').next().unwrap_or("").trim();
    if source.is_empty() {
        return Err(AsmError::new("empty assembly line"));
    }

    let split = source.find(char::is_whitespace).unwrap_or(source.len());
    let mnemonic = source[..split].to_ascii_uppercase();
    let operands = split_operands(source[split..].trim())?;
    let ops: Vec<String> = operands.iter().map(|op| normalize(op)).collect();
    let op: Vec<&str> = ops.iter().map(String::as_str).collect();

    let bytes = match mnemonic.as_str() {
        "DB" | "DEFB" => assemble_db(&op)?,
        "LD" => assemble_ld(&op)?,
        "INC" => assemble_inc_dec(&op, false)?,
        "DEC" => assemble_inc_dec(&op, true)?,
        "ADD" => assemble_add(&op)?,
        "ADC" => assemble_adc_sbc(&op, false)?,
        "SBC" => assemble_adc_sbc(&op, true)?,
        "SUB" => assemble_alu(&op, 2)?,
        "AND" => assemble_alu(&op, 4)?,
        "XOR" => assemble_alu(&op, 5)?,
        "OR" => assemble_alu(&op, 6)?,
        "CP" => assemble_alu(&op, 7)?,
        "JP" => assemble_jp(&op)?,
        "JR" => assemble_jr(&op, pc)?,
        "DJNZ" => {
            expect_count(&op, 1)?;
            vec![0x10, relative(op[0], pc)?]
        }
        "CALL" => assemble_call(&op)?,
        "RET" => assemble_ret(&op)?,
        "RST" => assemble_rst(&op)?,
        "PUSH" => assemble_push_pop(&op, true)?,
        "POP" => assemble_push_pop(&op, false)?,
        "EX" => assemble_ex(&op)?,
        "IN" => assemble_in(&op)?,
        "OUT" => assemble_out(&op)?,
        "IM" => assemble_im(&op)?,
        "BIT" => assemble_bit(&op, 1)?,
        "RES" => assemble_bit(&op, 2)?,
        "SET" => assemble_bit(&op, 3)?,
        "RLC" => assemble_rotate(&op, 0)?,
        "RRC" => assemble_rotate(&op, 1)?,
        "RL" => assemble_rotate(&op, 2)?,
        "RR" => assemble_rotate(&op, 3)?,
        "SLA" => assemble_rotate(&op, 4)?,
        "SRA" => assemble_rotate(&op, 5)?,
        "SLL" => assemble_rotate(&op, 6)?,
        "SRL" => assemble_rotate(&op, 7)?,
        _ => assemble_fixed(&mnemonic, &op)?,
    };

    Ok(bytes)
}

/// Assemble a Z80 source block at `origin`, without file access.
///
/// `INCLUDE`, `INCBIN` and `IMPORT` are errors; use
/// [`assemble_program_with`] to provide files. See `info/assembler.md` for
/// the source language. Instruction encoding is delegated to
/// [`assemble_line`].
pub fn assemble_program(source: &str, origin: u16) -> Result<AssembledProgram, AsmError> {
    assemble_program_with(source, origin, "", None)
}

/// Assemble the source named `name` at `origin`, reading `INCLUDE`,
/// `INCBIN` and `IMPORT` files through `files`.
pub fn assemble_program_with(
    source: &str,
    origin: u16,
    name: &str,
    files: Option<&dyn AsmFiles>,
) -> Result<AssembledProgram, AsmError> {
    let mut loader = SourceLoader {
        files,
        names: vec![name.to_string()],
    };
    let mut stack = vec![name.to_string()];
    let mut parsed = loader.load(source, 0, &mut stack)?;
    let mut imported = BTreeSet::new();

    let mut symbols = BTreeMap::new();
    let mut mappings = Vec::new();
    let mut mappings_by_name = BTreeMap::new();
    // `pc` is the address code runs at, which labels and `$` see; `load` is
    // where its bytes go. They differ only between PHASE and DEPHASE.
    let mut pc = origin;
    let mut load = origin;
    let mut phase_line: Option<ParsedLine> = None;
    let mut index = 0;
    let mut repeated_lines = 0usize;

    while index < parsed.len() {
        // A REPT block is expanded when pass 1 reaches it, so its count may use
        // any symbol defined above it. The copies are then assembled in turn.
        if let Statement::Rept { count, counter } = &parsed[index].statement {
            for label in &parsed[index].labels {
                insert_symbol(&mut symbols, label, pc, &parsed[index])?;
            }
            let count = eval_nonnegative(count, pc, &symbols, &mappings_by_name, false, 0)
                .map_err(|err| parsed[index].error(err))?;
            let counter = counter.clone();
            let end = matching_endr(&parsed, index)?;
            let body = &parsed[index + 1..end];
            repeated_lines = repeated_lines.saturating_add(count.saturating_mul(body.len()));
            if repeated_lines > MAX_REPT_LINES {
                return Err(parsed[index].error(AsmError::new(format!(
                    "REPT blocks expand to more than {MAX_REPT_LINES} lines"
                ))));
            }
            let mut expansion = Vec::with_capacity(count * body.len() + 1);
            for copy in 0..count {
                for line in body {
                    expansion.push(match &counter {
                        Some(name) => {
                            let mut copy = parse_program_line(
                                line.line_no,
                                &substitute_symbol(&line.source, name, copy),
                                line.file,
                                line.file_name.clone(),
                            )?;
                            keep_loaded_files(&line.statement, &mut copy.statement);
                            copy
                        }
                        None => line.clone(),
                    });
                }
            }
            expansion.push(labels_only(&parsed[end]));
            parsed.splice(index..=end, expansion);
            continue;
        }

        let line = &parsed[index];
        index += 1;
        if !matches!(line.statement, Statement::Equ { .. }) {
            for label in &line.labels {
                insert_symbol(&mut symbols, label, pc, line)?;
            }
        }

        match &line.statement {
            Statement::Empty => {}
            Statement::Org {
                address,
                mappings: declarations,
            } => {
                if phase_line.is_some() {
                    return Err(line.error(AsmError::new(
                        "ORG inside PHASE; end the block with DEPHASE first",
                    )));
                }
                pc = eval_word(address, pc, &symbols, &mappings_by_name, false, 0)
                    .map_err(|err| line.error(err))?;
                load = pc;
                for (name, mapped_address) in declarations {
                    if mappings_by_name.contains_key(name) {
                        return Err(
                            line.error(AsmError::new(format!("duplicate mapping '{name}'")))
                        );
                    }
                    let mapped_base =
                        eval_word(mapped_address, pc, &symbols, &mappings_by_name, false, 0)
                            .map_err(|err| line.error(err))?;
                    let mapping = AsmMapping {
                        name: name.clone(),
                        source_base: pc,
                        mapped_base,
                    };
                    mappings_by_name.insert(name.clone(), mapping.clone());
                    mappings.push(mapping);
                }
            }
            Statement::BasicStart => {
                if phase_line.is_some() {
                    return Err(line.error(AsmError::new("BASIC_START inside PHASE")));
                }
                load = TVC_BASIC_USR_ENTRY;
                insert_symbol(&mut symbols, "BASIC_START", TVC_BASIC_USR_ENTRY, line)?;
                pc = TVC_BASIC_USR_ENTRY;
            }
            Statement::Equ { label, expr } => {
                let value = eval_word(expr, pc, &symbols, &mappings_by_name, false, 0)
                    .map_err(|err| line.error(err))?;
                insert_symbol(&mut symbols, label, value, line)?;
            }
            Statement::Bytes(values) => {
                let len = byte_values_len(values).map_err(|err| line.error(err))? as u16;
                pc = pc.wrapping_add(len);
                load = load.wrapping_add(len);
            }
            Statement::Words(values) => {
                let len = values.len().wrapping_mul(2) as u16;
                pc = pc.wrapping_add(len);
                load = load.wrapping_add(len);
            }
            Statement::Space { count, .. } => {
                let count = eval_nonnegative(count, pc, &symbols, &mappings_by_name, false, 0)
                    .map_err(|err| line.error(err))?;
                pc = pc.wrapping_add(count as u16);
                load = load.wrapping_add(count as u16);
            }
            Statement::Instruction(statement) => {
                let rendered = render_instruction(statement, pc, &symbols, &mappings_by_name, true)
                    .map_err(|err| line.error(err))?;
                let bytes = assemble_line(&rendered, pc).map_err(|err| line.error(err))?;
                pc = pc.wrapping_add(bytes.len() as u16);
                load = load.wrapping_add(bytes.len() as u16);
            }
            Statement::Phase(address) => {
                if let Some(start) = &phase_line {
                    return Err(line.error(AsmError::new(format!(
                        "PHASE inside PHASE; the block from line {} has no DEPHASE",
                        start.line_no
                    ))));
                }
                pc = eval_word(address, pc, &symbols, &mappings_by_name, false, 0)
                    .map_err(|err| line.error(err))?;
                phase_line = Some(labels_only(line));
            }
            Statement::Dephase => {
                if phase_line.take().is_none() {
                    return Err(line.error(AsmError::new("DEPHASE without PHASE")));
                }
                pc = load;
            }
            Statement::Incbin {
                data,
                offset,
                length,
            } => {
                let (start, end) =
                    incbin_range(data, offset, length, pc, &symbols, &mappings_by_name)
                        .map_err(|err| line.error(err))?;
                let len = (end - start) as u16;
                pc = pc.wrapping_add(len);
                load = load.wrapping_add(len);
            }
            Statement::Import(entries) => {
                for (name, value) in entries.iter() {
                    insert_symbol(&mut symbols, name, *value, line)?;
                    imported.insert(name.clone());
                }
            }
            Statement::Include(_) | Statement::IncbinFile { .. } | Statement::ImportFile { .. } => {
                unreachable!("files are read when the source is loaded")
            }
            Statement::Assert { .. } | Statement::Break => {}
            Statement::Endr => {
                return Err(line.error(AsmError::new("ENDR without REPT")));
            }
            Statement::Rept { .. } => unreachable!("REPT blocks are expanded above"),
        }
    }
    if let Some(start) = phase_line {
        return Err(start.error(AsmError::new("PHASE without DEPHASE")));
    }

    let mut bytes = Vec::new();
    let mut segments = Vec::new();
    let mut lines = Vec::new();
    let mut breakpoints = Vec::new();
    pc = origin;
    load = origin;

    for line in &parsed {
        let (addr, run_addr) = (load, pc);
        let emitted = match &line.statement {
            Statement::Empty => Vec::new(),
            Statement::Org { address, .. } => {
                pc = eval_word(address, pc, &symbols, &mappings_by_name, false, 0)
                    .map_err(|err| line.error(err))?;
                load = pc;
                Vec::new()
            }
            Statement::BasicStart => {
                pc = TVC_BASIC_LOAD_ADDR;
                load = TVC_BASIC_LOAD_ADDR;
                tvc_basic_start_bytes()
            }
            Statement::Phase(address) => {
                pc = eval_word(address, pc, &symbols, &mappings_by_name, false, 0)
                    .map_err(|err| line.error(err))?;
                Vec::new()
            }
            Statement::Dephase => {
                pc = load;
                Vec::new()
            }
            Statement::Incbin {
                data,
                offset,
                length,
            } => {
                let (start, end) =
                    incbin_range(data, offset, length, pc, &symbols, &mappings_by_name)
                        .map_err(|err| line.error(err))?;
                data[start..end].to_vec()
            }
            Statement::Import(_) => Vec::new(),
            Statement::Break => {
                breakpoints.push(AsmBreakpoint {
                    addr: pc,
                    file: line.file,
                    line: line.line_no,
                });
                Vec::new()
            }
            Statement::Include(_) | Statement::IncbinFile { .. } | Statement::ImportFile { .. } => {
                unreachable!("files are read when the source is loaded")
            }
            Statement::Equ { .. } => Vec::new(),
            Statement::Bytes(values) => emit_byte_values(values, pc, &symbols, &mappings_by_name)
                .map_err(|err| line.error(err))?,
            Statement::Words(values) => {
                let mut out = Vec::with_capacity(values.len() * 2);
                for value in values {
                    let value = eval_word(value, pc, &symbols, &mappings_by_name, false, 0)
                        .map_err(|err| line.error(err))?;
                    push_word(&mut out, value);
                }
                out
            }
            Statement::Space { count, fill } => {
                let count = eval_nonnegative(count, pc, &symbols, &mappings_by_name, false, 0)
                    .map_err(|err| line.error(err))?;
                let fill = match fill {
                    Some(fill) => eval_byte(fill, pc, &symbols, &mappings_by_name, false, 0)
                        .map_err(|err| line.error(err))?,
                    None => 0,
                };
                vec![fill; count]
            }
            Statement::Instruction(statement) => {
                let rendered =
                    render_instruction(statement, pc, &symbols, &mappings_by_name, false)
                        .map_err(|err| line.error(err))?;
                assemble_line(&rendered, pc).map_err(|err| line.error(err))?
            }
            Statement::Assert { condition, message } => {
                check_assert(
                    condition,
                    message.as_deref(),
                    pc,
                    &symbols,
                    &mappings_by_name,
                )
                .map_err(|err| line.error(err))?;
                Vec::new()
            }
            Statement::Rept { .. } | Statement::Endr => {
                unreachable!("REPT blocks are expanded in pass 1")
            }
        };

        if !emitted.is_empty() {
            emit_segment(&mut segments, load, &emitted);
            bytes.extend_from_slice(&emitted);
            let (line_addr, line_run_addr) = if matches!(line.statement, Statement::BasicStart) {
                (TVC_BASIC_LOAD_ADDR, TVC_BASIC_LOAD_ADDR)
            } else {
                (addr, run_addr)
            };
            lines.push(AssembledLine {
                file: line.file,
                line: line.line_no,
                addr: line_addr,
                run_addr: line_run_addr,
                len: emitted.len(),
                source: line.source.clone(),
            });
            pc = pc.wrapping_add(emitted.len() as u16);
            load = load.wrapping_add(emitted.len() as u16);
        }
    }

    // Imported symbols belong to the module they came from.
    symbols.retain(|name, _| !imported.contains(name));

    Ok(AssembledProgram {
        origin: segments
            .first()
            .map(|segment| segment.addr)
            .unwrap_or(origin),
        bytes,
        segments,
        symbols,
        mappings,
        lines,
        next_addr: pc,
        files: loader.names,
        breakpoints,
    })
}

fn parse_program_line(
    line_no: usize,
    source: &str,
    file: usize,
    file_name: Option<Rc<str>>,
) -> Result<ParsedLine, AsmError> {
    let mut line = ParsedLine {
        line_no,
        file,
        file_name,
        source: source.to_string(),
        labels: Vec::new(),
        statement: Statement::Empty,
    };
    let clean = strip_comment(source).map_err(|err| line.error(err))?;
    let mut rest = clean.trim();
    while let Some((label, after)) = take_colon_label(rest).map_err(|err| line.error(err))? {
        line.labels.push(label);
        rest = after.trim_start();
    }
    line.statement = parse_statement(rest, &line.labels).map_err(|err| line.error(err))?;
    Ok(line)
}

impl ParsedLine {
    /// Attach this line, and its file when it is an included one, to an error.
    fn error(&self, err: AsmError) -> AsmError {
        let mut err = line_error(self.line_no, err);
        if err.file.is_none() {
            err.file = self.file_name.as_deref().map(str::to_string);
        }
        err
    }
}

/// Reads `INCLUDE`, `INCBIN` and `IMPORT` files while parsing the source.
struct SourceLoader<'a> {
    files: Option<&'a dyn AsmFiles>,
    names: Vec<String>,
}

impl SourceLoader<'_> {
    /// Parse `source`, the file at index `file`, expanding includes. `stack`
    /// holds the names of the files being included, outermost first.
    fn load(
        &mut self,
        source: &str,
        file: usize,
        stack: &mut Vec<String>,
    ) -> Result<Vec<ParsedLine>, AsmError> {
        let file_name: Option<Rc<str>> = (file != 0).then(|| self.names[file].as_str().into());
        let mut out = Vec::new();
        for (index, text) in source.lines().enumerate() {
            let mut line = parse_program_line(index + 1, text, file, file_name.clone())?;
            match std::mem::replace(&mut line.statement, Statement::Empty) {
                Statement::Include(path) => {
                    let (name, bytes) = self.read(&line, &path, AsmFileKind::Source)?;
                    if stack.contains(&name) {
                        let chain = stack.join(" -> ");
                        return Err(
                            line.error(AsmError::new(format!("INCLUDE cycle: {chain} -> {name}")))
                        );
                    }
                    if stack.len() > MAX_INCLUDE_DEPTH {
                        return Err(line.error(AsmError::new(format!(
                            "INCLUDE files nest more than {MAX_INCLUDE_DEPTH} deep"
                        ))));
                    }
                    let text = String::from_utf8(bytes).map_err(|_| {
                        line.error(AsmError::new(format!("'{name}' is not UTF-8 text")))
                    })?;
                    let index = match self.names.iter().position(|known| *known == name) {
                        Some(index) => index,
                        None => {
                            self.names.push(name.clone());
                            self.names.len() - 1
                        }
                    };
                    // The INCLUDE line keeps its labels, which mark the
                    // address of the included code.
                    out.push(line);
                    stack.push(name);
                    out.extend(self.load(&text, index, stack)?);
                    stack.pop();
                }
                Statement::IncbinFile {
                    path,
                    offset,
                    length,
                } => {
                    let (_, bytes) = self.read(&line, &path, AsmFileKind::Binary)?;
                    line.statement = Statement::Incbin {
                        data: bytes.into(),
                        offset,
                        length,
                    };
                    out.push(line);
                }
                Statement::ImportFile { path, prefix } => {
                    let (name, bytes) = self.read(&line, &path, AsmFileKind::Symbols)?;
                    let entries = import_symbols(&name, &bytes, prefix.as_deref())
                        .map_err(|err| line.error(err))?;
                    line.statement = Statement::Import(entries.into());
                    out.push(line);
                }
                statement => {
                    line.statement = statement;
                    out.push(line);
                }
            }
        }
        Ok(out)
    }

    fn read(
        &self,
        line: &ParsedLine,
        path: &str,
        kind: AsmFileKind,
    ) -> Result<(String, Vec<u8>), AsmError> {
        let directive = match kind {
            AsmFileKind::Source => "INCLUDE",
            AsmFileKind::Binary => "INCBIN",
            AsmFileKind::Symbols => "IMPORT",
        };
        let Some(files) = self.files else {
            return Err(line.error(AsmError::new(format!(
                "{directive} needs file access, which this assembler call does not have"
            ))));
        };
        files
            .read(path, &self.names[line.file], kind)
            .map_err(|err| line.error(AsmError::new(err)))
    }
}

/// A `REPT` copy with a counter is parsed again from its text; give it the
/// file contents its original line already read.
fn keep_loaded_files(original: &Statement, copy: &mut Statement) {
    match (original, &mut *copy) {
        (Statement::Incbin { data, .. }, Statement::IncbinFile { offset, length, .. }) => {
            *copy = Statement::Incbin {
                data: data.clone(),
                offset: offset.take(),
                length: length.take(),
            };
        }
        (Statement::Import(entries), Statement::ImportFile { .. }) => {
            *copy = Statement::Import(entries.clone());
        }
        // The included lines follow the INCLUDE line already.
        (_, Statement::Include(_)) => *copy = Statement::Empty,
        _ => {}
    }
}

/// The byte range an `INCBIN` emits.
fn incbin_range(
    data: &[u8],
    offset: &Option<String>,
    length: &Option<String>,
    pc: u16,
    symbols: &BTreeMap<String, u16>,
    mappings: &BTreeMap<String, AsmMapping>,
) -> Result<(usize, usize), AsmError> {
    let start = match offset {
        Some(offset) => eval_nonnegative(offset, pc, symbols, mappings, false, 0)?,
        None => 0,
    };
    if start > data.len() {
        return Err(AsmError::new(format!(
            "INCBIN offset {start} is past the end of the {}-byte file",
            data.len()
        )));
    }
    let end = match length {
        Some(length) => start + eval_nonnegative(length, pc, symbols, mappings, false, 0)?,
        None => data.len(),
    };
    if end > data.len() {
        return Err(AsmError::new(format!(
            "INCBIN range {start}..{end} is past the end of the {}-byte file",
            data.len()
        )));
    }
    Ok((start, end))
}

/// Read the symbols of an `rtvc-asm-v1` TOML module for `IMPORT`.
#[cfg(feature = "asm-toml")]
fn import_symbols(
    name: &str,
    bytes: &[u8],
    prefix: Option<&str>,
) -> Result<Vec<(String, u16)>, AsmError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AsmError::new(format!("'{name}' is not UTF-8 text")))?;
    let module = parse_rtvc_asm_v1(text)
        .map_err(|err| AsmError::new(format!("cannot import '{name}': {err}")))?;
    let prefix = prefix.unwrap_or("");
    Ok(module
        .symbols
        .into_iter()
        .map(|(symbol, value)| (format!("{prefix}{symbol}"), value))
        .collect())
}

#[cfg(not(feature = "asm-toml"))]
fn import_symbols(
    _name: &str,
    _bytes: &[u8],
    _prefix: Option<&str>,
) -> Result<Vec<(String, u16)>, AsmError> {
    Err(AsmError::new(
        "IMPORT needs a build with the asm-toml feature",
    ))
}

fn parse_statement(source: &str, labels: &[String]) -> Result<Statement, AsmError> {
    if source.is_empty() {
        return Ok(Statement::Empty);
    }

    let (mnemonic, rest) = split_statement_head(source);
    let mnemonic_upper = mnemonic.to_ascii_uppercase();
    if mnemonic_upper == "EQU" {
        if labels.len() != 1 {
            return Err(AsmError::new("EQU requires exactly one label"));
        }
        if rest.trim().is_empty() {
            return Err(AsmError::new("EQU requires an expression"));
        }
        return Ok(Statement::Equ {
            label: labels[0].clone(),
            expr: rest.trim().to_string(),
        });
    }

    if labels.is_empty()
        && let Some((next, after_next)) = split_optional_head(rest)
        && next.eq_ignore_ascii_case("EQU")
    {
        validate_label(mnemonic)?;
        if after_next.trim().is_empty() {
            return Err(AsmError::new("EQU requires an expression"));
        }
        return Ok(Statement::Equ {
            label: mnemonic.to_ascii_uppercase(),
            expr: after_next.trim().to_string(),
        });
    }

    match mnemonic_upper.as_str() {
        "ORG" => {
            if rest.trim().is_empty() {
                return Err(AsmError::new("ORG requires an address"));
            }
            parse_org(rest)
        }
        "MAP" => Err(AsmError::new(
            "MAP is no longer supported; declare mappings on ORG",
        )),
        "BASIC_START" => {
            if !rest.trim().is_empty() {
                return Err(AsmError::new("BASIC_START takes no operands"));
            }
            Ok(Statement::BasicStart)
        }
        "DB" | "DEFB" => Ok(Statement::Bytes(parse_byte_values(rest)?)),
        "DW" | "DEFW" => Ok(Statement::Words(split_operands(rest)?)),
        "DS" | "DEFS" => {
            let operands = split_operands(rest)?;
            match operands.as_slice() {
                [count] => Ok(Statement::Space {
                    count: count.clone(),
                    fill: None,
                }),
                [count, fill] => Ok(Statement::Space {
                    count: count.clone(),
                    fill: Some(fill.clone()),
                }),
                _ => Err(AsmError::new("DS requires count and optional fill byte")),
            }
        }
        "REPT" => {
            let operands = split_operands(rest)?;
            match operands.as_slice() {
                [count] => Ok(Statement::Rept {
                    count: count.clone(),
                    counter: None,
                }),
                [count, counter] => {
                    validate_label(counter)?;
                    Ok(Statement::Rept {
                        count: count.clone(),
                        counter: Some(counter.to_ascii_uppercase()),
                    })
                }
                _ => Err(AsmError::new(
                    "REPT requires a count and optional counter name",
                )),
            }
        }
        "ENDR" => {
            if !rest.trim().is_empty() {
                return Err(AsmError::new("ENDR takes no operands"));
            }
            Ok(Statement::Endr)
        }
        "INCLUDE" => match split_operands(rest)?.as_slice() {
            [path] => Ok(Statement::Include(parse_path(path)?)),
            _ => Err(AsmError::new("INCLUDE requires one quoted path")),
        },
        "INCBIN" => match split_operands(rest)?.as_slice() {
            [path, rest @ ..] if rest.len() <= 2 => Ok(Statement::IncbinFile {
                path: parse_path(path)?,
                offset: rest.first().cloned(),
                length: rest.get(1).cloned(),
            }),
            _ => Err(AsmError::new(
                "INCBIN requires a quoted path, an optional offset and an optional length",
            )),
        },
        "IMPORT" => match split_operands(rest)?.as_slice() {
            [path] => Ok(Statement::ImportFile {
                path: parse_path(path)?,
                prefix: None,
            }),
            [path, prefix] => {
                validate_label(prefix)?;
                Ok(Statement::ImportFile {
                    path: parse_path(path)?,
                    prefix: Some(prefix.to_ascii_uppercase()),
                })
            }
            _ => Err(AsmError::new(
                "IMPORT requires a quoted path and an optional name prefix",
            )),
        },
        "BREAK" => {
            if !rest.trim().is_empty() {
                return Err(AsmError::new("BREAK takes no operands"));
            }
            Ok(Statement::Break)
        }
        "PHASE" => {
            if rest.trim().is_empty() {
                return Err(AsmError::new("PHASE requires a run address"));
            }
            Ok(Statement::Phase(rest.trim().to_string()))
        }
        "DEPHASE" => {
            if !rest.trim().is_empty() {
                return Err(AsmError::new("DEPHASE takes no operands"));
            }
            Ok(Statement::Dephase)
        }
        "ASSERT" => {
            let operands = split_operands(rest)?;
            match operands.as_slice() {
                [condition] => Ok(Statement::Assert {
                    condition: condition.clone(),
                    message: None,
                }),
                [condition, message] if is_quoted(message) => {
                    let message = String::from_utf8(parse_string_literal(message)?)
                        .expect("ASCII string literal");
                    Ok(Statement::Assert {
                        condition: condition.clone(),
                        message: Some(message),
                    })
                }
                _ => Err(AsmError::new(
                    "ASSERT requires a condition and optional quoted message",
                )),
            }
        }
        _ => Ok(Statement::Instruction(source.to_string())),
    }
}

/// Index of the `ENDR` that closes the `REPT` at `start`, counting nested blocks.
fn matching_endr(lines: &[ParsedLine], start: usize) -> Result<usize, AsmError> {
    let mut depth = 0usize;
    for (index, line) in lines.iter().enumerate().skip(start) {
        match line.statement {
            Statement::Rept { .. } => depth += 1,
            Statement::Endr => {
                depth -= 1;
                if depth == 0 {
                    return Ok(index);
                }
            }
            _ => {}
        }
    }
    Err(lines[start].error(AsmError::new("REPT without ENDR")))
}

/// Keep an `ENDR` line's labels, which mark the address after the block.
fn labels_only(line: &ParsedLine) -> ParsedLine {
    ParsedLine {
        line_no: line.line_no,
        file: line.file,
        file_name: line.file_name.clone(),
        source: line.source.clone(),
        labels: line.labels.clone(),
        statement: Statement::Empty,
    }
}

/// Replace whole-word, case-insensitive uses of `name` outside string literals
/// with `value`, for a `REPT` counter.
fn substitute_symbol(source: &str, name: &str, value: usize) -> String {
    let is_word = |ch: char| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.';
    let mut out = String::with_capacity(source.len());
    let mut quote = None;
    let mut rest = source;
    while let Some(ch) = rest.chars().next() {
        if let Some(quote_ch) = quote {
            out.push(ch);
            rest = &rest[ch.len_utf8()..];
            if ch == '\\' {
                if let Some(escaped) = rest.chars().next() {
                    out.push(escaped);
                    rest = &rest[escaped.len_utf8()..];
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if ch == '"' {
            quote = Some(ch);
        }
        if is_word(ch) {
            let len = rest.find(|ch: char| !is_word(ch)).unwrap_or(rest.len());
            let word = &rest[..len];
            if word.eq_ignore_ascii_case(name) {
                out.push_str(&value.to_string());
            } else {
                out.push_str(word);
            }
            rest = &rest[len..];
            continue;
        }
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

const COMPARISONS: [&str; 8] = ["==", "!=", "<>", "<=", ">=", "=", "<", ">"];

/// Split an `ASSERT` condition at its single comparison operator, if any.
fn split_comparison(condition: &str) -> Result<Option<(&str, &str, &str)>, AsmError> {
    let mut found = None;
    let mut quote = None;
    let mut index = 0;
    while index < condition.len() {
        let rest = &condition[index..];
        let ch = rest.chars().next().unwrap();
        if let Some(quote_ch) = quote {
            if ch == quote_ch {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if rest.starts_with("<<") || rest.starts_with(">>") {
            index += 2;
            continue;
        } else if let Some(op) = COMPARISONS.iter().find(|op| rest.starts_with(**op)) {
            if found.is_some() {
                return Err(AsmError::new(format!(
                    "ASSERT condition '{condition}' has more than one comparison"
                )));
            }
            found = Some((&condition[..index], *op, &condition[index + op.len()..]));
            index += op.len();
            continue;
        }
        index += ch.len_utf8();
    }
    Ok(found)
}

fn check_assert(
    condition: &str,
    message: Option<&str>,
    pc: u16,
    symbols: &BTreeMap<String, u16>,
    mappings: &BTreeMap<String, AsmMapping>,
) -> Result<(), AsmError> {
    let eval = |expr: &str| eval_expr(expr, pc, symbols, mappings, false, 0);
    let (holds, values) = match split_comparison(condition)? {
        Some((left, op, right)) => {
            let (left, right) = (eval(left)?, eval(right)?);
            let holds = match op {
                "=" | "==" => left == right,
                "!=" | "<>" => left != right,
                "<" => left < right,
                "<=" => left <= right,
                ">" => left > right,
                ">=" => left >= right,
                _ => unreachable!("COMPARISONS lists every operator"),
            };
            (holds, format!("left is {left}, right is {right}"))
        }
        None => {
            let value = eval(condition)?;
            (value != 0, format!("value is {value}"))
        }
    };
    if holds {
        return Ok(());
    }
    let what = message.unwrap_or(condition.trim());
    Err(AsmError::new(format!(
        "assertion failed: {what} ({values})"
    )))
}

fn parse_org(source: &str) -> Result<Statement, AsmError> {
    let operands = split_operands(source)?;
    if operands.is_empty() {
        return Err(AsmError::new("ORG requires an address"));
    }
    let mapping_operands = &operands[1..];
    if mapping_operands.len() % 2 != 0 {
        return Err(AsmError::new(
            "ORG mappings require a name and mapped address pair",
        ));
    }
    let mappings = mapping_operands
        .chunks_exact(2)
        .map(|pair| {
            validate_label(&pair[0])?;
            Ok((pair[0].to_ascii_uppercase(), pair[1].clone()))
        })
        .collect::<Result<Vec<_>, AsmError>>()?;
    Ok(Statement::Org {
        address: operands[0].clone(),
        mappings,
    })
}

fn tvc_basic_start_bytes() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&[
        0x0F, 0x0A, 0x00, 0x43, 0x9A, b'U', b'S', b'R', 0x96, b'6', b'7', b'0', b'4', 0x95, 0xFF,
        0x00,
    ]);
    out.resize((TVC_BASIC_USR_ENTRY - TVC_BASIC_LOAD_ADDR) as usize, 0x00);
    out
}

fn strip_comment(source: &str) -> Result<String, AsmError> {
    let mut quote = None;
    let mut escape = false;
    let mut out = String::new();
    for (index, ch) in source.char_indices() {
        if escape {
            out.push(ch);
            escape = false;
            continue;
        }
        if quote.is_some() && ch == '\\' {
            out.push(ch);
            escape = true;
            continue;
        }
        if let Some(quote_ch) = quote {
            out.push(ch);
            if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        match ch {
            ';' => break,
            '"' => {
                quote = Some(ch);
                out.push(ch);
            }
            '\'' if single_quote_starts_operand(&out) => {
                quote = Some(ch);
                out.push(ch);
            }
            '\'' if single_quote_starts_statement_string(&out, source, index) => {
                quote = Some(ch);
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    if quote.is_some() {
        return Err(AsmError::new("unterminated string literal"));
    }
    Ok(out)
}

fn single_quote_starts_operand(prefix: &str) -> bool {
    prefix
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
        .is_none_or(|ch| ch == ',')
}

fn single_quote_starts_statement_string(prefix: &str, source: &str, index: usize) -> bool {
    prefix.chars().last().is_some_and(|ch| ch.is_whitespace()) && source[index + 1..].contains('\'')
}

fn take_colon_label(source: &str) -> Result<Option<(String, &str)>, AsmError> {
    let Some(colon) = source.find(':') else {
        return Ok(None);
    };
    let before = source[..colon].trim();
    if before.is_empty() || before.contains(char::is_whitespace) {
        return Ok(None);
    }
    validate_label(before)?;
    Ok(Some((before.to_ascii_uppercase(), &source[colon + 1..])))
}

fn validate_label(label: &str) -> Result<(), AsmError> {
    let mut chars = label.chars();
    let Some(first) = chars.next() else {
        return Err(AsmError::new("empty label"));
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '.') {
        return Err(AsmError::new(format!("invalid label '{}'", label)));
    }
    if chars.any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')) {
        return Err(AsmError::new(format!("invalid label '{}'", label)));
    }
    Ok(())
}

fn split_statement_head(source: &str) -> (&str, &str) {
    let split = source.find(char::is_whitespace).unwrap_or(source.len());
    (&source[..split], source[split..].trim_start())
}

fn split_optional_head(source: &str) -> Option<(&str, &str)> {
    if source.trim().is_empty() {
        return None;
    }
    Some(split_statement_head(source.trim_start()))
}

fn parse_byte_values(source: &str) -> Result<Vec<ByteValue>, AsmError> {
    let operands = split_operands(source)?;
    if operands.is_empty() {
        return Err(AsmError::new("DB requires at least one byte"));
    }
    operands
        .into_iter()
        .map(|operand| {
            if is_quoted(&operand) {
                parse_string_literal(&operand).map(ByteValue::String)
            } else {
                Ok(ByteValue::Expr(operand))
            }
        })
        .collect()
}

/// A quoted file path operand.
fn parse_path(value: &str) -> Result<String, AsmError> {
    if !is_quoted(value) {
        return Err(AsmError::new(format!(
            "expected a quoted path, got {value}"
        )));
    }
    let path = String::from_utf8(parse_string_literal(value)?).expect("ASCII string literal");
    if path.is_empty() {
        return Err(AsmError::new("empty path"));
    }
    Ok(path)
}

fn is_quoted(value: &str) -> bool {
    value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
}

fn parse_string_literal(value: &str) -> Result<Vec<u8>, AsmError> {
    let inner = &value[1..value.len() - 1];
    let mut out = Vec::new();
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        let ch = if ch == '\\' {
            match chars.next() {
                Some('0') => '\0',
                Some('n') => '\n',
                Some('r') => '\r',
                Some('t') => '\t',
                Some('\\') => '\\',
                Some('"') => '"',
                Some('\'') => '\'',
                Some(other) => {
                    return Err(AsmError::new(format!("unsupported escape '\\{}'", other)));
                }
                None => return Err(AsmError::new("unterminated escape sequence")),
            }
        } else {
            ch
        };
        if !ch.is_ascii() {
            return Err(AsmError::new("DB string literals must be ASCII"));
        }
        out.push(ch as u8);
    }
    Ok(out)
}

fn byte_values_len(values: &[ByteValue]) -> Result<usize, AsmError> {
    values.iter().try_fold(0usize, |len, value| match value {
        ByteValue::Expr(_) => Ok(len + 1),
        ByteValue::String(bytes) => len
            .checked_add(bytes.len())
            .ok_or_else(|| AsmError::new("DB data is too long")),
    })
}

fn emit_byte_values(
    values: &[ByteValue],
    pc: u16,
    symbols: &BTreeMap<String, u16>,
    mappings: &BTreeMap<String, AsmMapping>,
) -> Result<Vec<u8>, AsmError> {
    let mut out = Vec::new();
    for value in values {
        match value {
            ByteValue::Expr(expr) => out.push(eval_byte(expr, pc, symbols, mappings, false, 0)?),
            ByteValue::String(bytes) => out.extend_from_slice(bytes),
        }
    }
    Ok(out)
}

fn render_instruction(
    source: &str,
    pc: u16,
    symbols: &BTreeMap<String, u16>,
    mappings: &BTreeMap<String, AsmMapping>,
    allow_undefined: bool,
) -> Result<String, AsmError> {
    let (mnemonic, rest) = split_statement_head(source);
    let operands = split_operands(rest)?;
    if operands.is_empty() {
        return Ok(mnemonic.to_string());
    }

    let mnemonic_upper = mnemonic.to_ascii_uppercase();
    let unknown_value = if mnemonic_upper == "JR" || mnemonic_upper == "DJNZ" {
        pc.wrapping_add(2)
    } else {
        0
    };

    let rendered: Vec<_> = operands
        .iter()
        .map(|operand| {
            render_operand(
                operand,
                pc,
                symbols,
                mappings,
                allow_undefined,
                unknown_value,
            )
        })
        .collect::<Result<_, _>>()?;
    Ok(format!("{} {}", mnemonic, rendered.join(",")))
}

fn render_operand(
    operand: &str,
    pc: u16,
    symbols: &BTreeMap<String, u16>,
    mappings: &BTreeMap<String, AsmMapping>,
    allow_undefined: bool,
    unknown_value: u16,
) -> Result<String, AsmError> {
    let operand = normalize(operand);
    if is_fixed_operand(&operand) {
        return Ok(operand);
    }

    if is_wrapped(&operand) {
        let inner = &operand[1..operand.len() - 1];
        if let Some(rendered) =
            render_indexed_operand(inner, pc, symbols, mappings, allow_undefined)?
        {
            return Ok(format!("({rendered})"));
        }
        if is_fixed_operand(inner) {
            return Ok(operand);
        }
        let value = eval_word(inner, pc, symbols, mappings, allow_undefined, unknown_value)?;
        return Ok(format!("({:04X}H)", value));
    }

    let value = eval_expr(
        &operand,
        pc,
        symbols,
        mappings,
        allow_undefined,
        unknown_value,
    )?;
    if value < 0 {
        Ok(value.to_string())
    } else {
        Ok(format!("{:X}H", value))
    }
}

/// Whether the whole operand is one parenthesised group, which makes it a
/// memory operand such as `(LABEL+1)`, rather than an expression that starts
/// and ends with parentheses, such as `(A+1)*(B+1)`.
fn is_wrapped(operand: &str) -> bool {
    if !operand.starts_with('(') || !operand.ends_with(')') {
        return false;
    }
    let mut depth = 0usize;
    for (index, ch) in operand.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return index == operand.len() - 1;
                }
            }
            _ => {}
        }
    }
    false
}

fn is_fixed_operand(operand: &str) -> bool {
    matches!(
        operand,
        "A" | "B"
            | "C"
            | "D"
            | "E"
            | "H"
            | "L"
            | "F"
            | "I"
            | "R"
            | "AF"
            | "AF'"
            | "BC"
            | "DE"
            | "HL"
            | "SP"
            | "IX"
            | "IY"
            | "IXH"
            | "IXL"
            | "IYH"
            | "IYL"
            | "NZ"
            | "Z"
            | "NC"
            | "PO"
            | "PE"
            | "P"
            | "M"
            | "0"
            | "(HL)"
            | "(BC)"
            | "(DE)"
            | "(SP)"
            | "(C)"
            | "(IX)"
            | "(IY)"
    )
}

fn render_indexed_operand(
    inner: &str,
    pc: u16,
    symbols: &BTreeMap<String, u16>,
    mappings: &BTreeMap<String, AsmMapping>,
    allow_undefined: bool,
) -> Result<Option<String>, AsmError> {
    let (index, rest) = if let Some(rest) = inner.strip_prefix("IX") {
        ("IX", rest)
    } else if let Some(rest) = inner.strip_prefix("IY") {
        ("IY", rest)
    } else {
        return Ok(None);
    };

    if rest.is_empty() {
        return Ok(Some(index.to_string()));
    }
    let value = eval_expr(rest, pc, symbols, mappings, allow_undefined, 0)?;
    i8::try_from(value).map_err(|_| {
        AsmError::new(format!(
            "index displacement '{}' is outside -128..127",
            rest
        ))
    })?;
    if value < 0 {
        Ok(Some(format!("{index}{value}")))
    } else {
        Ok(Some(format!("{index}+{value}")))
    }
}

fn emit_segment(segments: &mut Vec<AssembledSegment>, addr: u16, bytes: &[u8]) {
    if let Some(last) = segments.last_mut() {
        if last.addr.wrapping_add(last.bytes.len() as u16) == addr {
            last.bytes.extend_from_slice(bytes);
            return;
        }
    }
    segments.push(AssembledSegment {
        addr,
        bytes: bytes.to_vec(),
    });
}

fn insert_symbol(
    symbols: &mut BTreeMap<String, u16>,
    label: &str,
    value: u16,
    line: &ParsedLine,
) -> Result<(), AsmError> {
    if symbols.insert(label.to_string(), value).is_some() {
        return Err(line.error(AsmError::new(format!("duplicate label '{}'", label))));
    }
    Ok(())
}

/// Attach a source line to an error, keeping a line it already has.
fn line_error(line_no: usize, mut err: AsmError) -> AsmError {
    err.line.get_or_insert(line_no);
    err
}

fn split_operands(source: &str) -> Result<Vec<String>, AsmError> {
    if source.is_empty() {
        return Ok(Vec::new());
    }

    let mut depth = 0u8;
    let mut quote = None;
    let mut escape = false;
    let mut start = 0usize;
    let mut operands = Vec::new();
    for (index, ch) in source.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if quote.is_some() && ch == '\\' {
            escape = true;
            continue;
        }
        if let Some(quote_ch) = quote {
            if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' => quote = Some(ch),
            '\'' if source[start..index].trim().is_empty() => quote = Some(ch),
            '(' => depth = depth.saturating_add(1),
            ')' => {
                if depth == 0 {
                    return Err(AsmError::new("unmatched ')'"));
                }
                depth -= 1;
            }
            ',' if depth == 0 => {
                let operand = source[start..index].trim();
                if operand.is_empty() {
                    return Err(AsmError::new("missing operand"));
                }
                operands.push(operand.to_string());
                start = index + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(AsmError::new("unmatched '('"));
    }
    if quote.is_some() {
        return Err(AsmError::new("unterminated string literal"));
    }

    let operand = source[start..].trim();
    if operand.is_empty() {
        return Err(AsmError::new("missing operand"));
    }
    operands.push(operand.to_string());
    Ok(operands)
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(char::to_uppercase)
        .collect()
}

fn assemble_db(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    if op.is_empty() {
        return Err(AsmError::new("DB requires at least one byte"));
    }
    op.iter().map(|value| byte(value)).collect()
}

fn assemble_ld(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    let dst = op[0];
    let src = op[1];

    if let (Some(d), Some(s)) = (reg8(dst), reg8(src)) {
        if d == 6 && s == 6 {
            return Err(AsmError::new("LD (HL),(HL) is not an instruction"));
        }
        return Ok(vec![0x40 | (d << 3) | s]);
    }
    match (dst, src) {
        ("(BC)", "A") => return Ok(vec![0x02]),
        ("A", "(BC)") => return Ok(vec![0x0A]),
        ("(DE)", "A") => return Ok(vec![0x12]),
        ("A", "(DE)") => return Ok(vec![0x1A]),
        ("SP", "HL") => return Ok(vec![0xF9]),
        ("I", "A") => return Ok(vec![0xED, 0x47]),
        ("R", "A") => return Ok(vec![0xED, 0x4F]),
        ("A", "I") => return Ok(vec![0xED, 0x57]),
        ("A", "R") => return Ok(vec![0xED, 0x5F]),
        ("SP", "IX") => return Ok(vec![0xDD, 0xF9]),
        ("SP", "IY") => return Ok(vec![0xFD, 0xF9]),
        _ => {}
    }

    if let Some(addr) = indirect_word(dst) {
        if src == "A" {
            let mut out = vec![0x32];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if src == "HL" {
            let mut out = vec![0x22];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if let Some(prefix) = index_prefix(src) {
            let mut out = vec![prefix, 0x22];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if let Some(p) = reg16(src) {
            let mut out = vec![0xED, 0x43 | (p << 4)];
            push_word(&mut out, addr);
            return Ok(out);
        }
    }

    if let Some(addr) = indirect_word(src) {
        if dst == "A" {
            let mut out = vec![0x3A];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if dst == "HL" {
            let mut out = vec![0x2A];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if let Some(prefix) = index_prefix(dst) {
            let mut out = vec![prefix, 0x2A];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if let Some(p) = reg16(dst) {
            let mut out = vec![0xED, 0x4B | (p << 4)];
            push_word(&mut out, addr);
            return Ok(out);
        }
    }

    if let Some((prefix, d)) = indexed(dst)? {
        if let Some(s) = reg8(src) {
            return Ok(vec![prefix, 0x70 | s, d as u8]);
        }
        return Ok(vec![prefix, 0x36, d as u8, byte(src)?]);
    }
    if let Some((prefix, d)) = indexed(src)? {
        if let Some(r) = reg8(dst) {
            return Ok(vec![prefix, 0x46 | (r << 3), d as u8]);
        }
    }

    if let Some(out) = assemble_ld_index_half(dst, src)? {
        return Ok(out);
    }
    if let Some(d) = reg8(dst) {
        return Ok(vec![0x06 | (d << 3), byte(src)?]);
    }
    if let Some(p) = reg16(dst) {
        let mut out = vec![0x01 | (p << 4)];
        push_word(&mut out, word(src)?);
        return Ok(out);
    }
    if let Some(prefix) = index_prefix(dst) {
        let mut out = vec![prefix, 0x21];
        push_word(&mut out, word(src)?);
        return Ok(out);
    }

    Err(form_error("LD", op))
}

/// Undocumented `LD` forms that use the IX/IY high or low byte.
///
/// A DD/FD prefix turns H/L into the index half, so the other operand may not
/// be H, L, (HL), or a half of the other index register.
fn assemble_ld_index_half(dst: &str, src: &str) -> Result<Option<Vec<u8>>, AsmError> {
    let dst_half = index_half(dst);
    let src_half = index_half(src);
    let prefix = match (dst_half, src_half) {
        (None, None) => return Ok(None),
        (Some((a, _)), Some((b, _))) if a != b => {
            return Err(AsmError::new("cannot mix IX and IY halves"));
        }
        (Some((prefix, _)), _) | (_, Some((prefix, _))) => prefix,
    };
    let operand = |value: &str, half: Option<(u8, u8)>| match half {
        Some((_, r)) => Ok(r),
        None => match reg8(value) {
            Some(r) if r != 4 && r != 5 && r != 6 => Ok(r),
            _ => Err(form_error("LD", &[dst, src])),
        },
    };
    let d = operand(dst, dst_half)?;
    if src_half.is_none() && reg8(src).is_none() {
        if dst_half.is_none() {
            return Err(form_error("LD", &[dst, src]));
        }
        return Ok(Some(vec![prefix, 0x06 | (d << 3), byte(src)?]));
    }
    let s = operand(src, src_half)?;
    Ok(Some(vec![prefix, 0x40 | (d << 3) | s]))
}

fn assemble_inc_dec(op: &[&str], decrement: bool) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    let base = if decrement { 0x05 } else { 0x04 };
    if let Some(r) = reg8(op[0]) {
        return Ok(vec![base | (r << 3)]);
    }
    if let Some((prefix, r)) = index_half(op[0]) {
        return Ok(vec![prefix, base | (r << 3)]);
    }
    if let Some(p) = reg16(op[0]) {
        return Ok(vec![(if decrement { 0x0B } else { 0x03 }) | (p << 4)]);
    }
    if let Some(prefix) = index_prefix(op[0]) {
        return Ok(vec![prefix, if decrement { 0x2B } else { 0x23 }]);
    }
    if let Some((prefix, d)) = indexed(op[0])? {
        return Ok(vec![prefix, base | (6 << 3), d as u8]);
    }
    Err(form_error(if decrement { "DEC" } else { "INC" }, op))
}

fn assemble_add(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    if op[0] == "HL" {
        if let Some(p) = reg16(op[1]) {
            return Ok(vec![0x09 | (p << 4)]);
        }
    }
    if let Some(prefix) = index_prefix(op[0]) {
        let p = match op[1] {
            "BC" => 0,
            "DE" => 1,
            value if value == op[0] => 2,
            "SP" => 3,
            _ => return Err(form_error("ADD", op)),
        };
        return Ok(vec![prefix, 0x09 | (p << 4)]);
    }
    if op[0] == "A" {
        return assemble_alu(&op[1..], 0);
    }
    Err(form_error("ADD", op))
}

fn assemble_adc_sbc(op: &[&str], subtract: bool) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    if op[0] == "HL" {
        if let Some(p) = reg16(op[1]) {
            let base = if subtract { 0x42 } else { 0x4A };
            return Ok(vec![0xED, base | (p << 4)]);
        }
    }
    if op[0] == "A" {
        return assemble_alu(&op[1..], if subtract { 3 } else { 1 });
    }
    Err(form_error(if subtract { "SBC" } else { "ADC" }, op))
}

fn assemble_alu(op: &[&str], operation: u8) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    if let Some(r) = reg8(op[0]) {
        return Ok(vec![0x80 | (operation << 3) | r]);
    }
    if let Some((prefix, r)) = index_half(op[0]) {
        return Ok(vec![prefix, 0x80 | (operation << 3) | r]);
    }
    if let Some((prefix, d)) = indexed(op[0])? {
        return Ok(vec![prefix, 0x80 | (operation << 3) | 6, d as u8]);
    }
    Ok(vec![0xC6 | (operation << 3), byte(op[0])?])
}

fn assemble_jp(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    match op {
        ["(HL)"] => Ok(vec![0xE9]),
        ["(IX)"] => Ok(vec![0xDD, 0xE9]),
        ["(IY)"] => Ok(vec![0xFD, 0xE9]),
        [target] => {
            let mut out = vec![0xC3];
            push_word(&mut out, word(target)?);
            Ok(out)
        }
        [condition, target] => {
            let cc = condition_code(condition)
                .ok_or_else(|| AsmError::new(format!("invalid JP condition '{}'", condition)))?;
            let mut out = vec![0xC2 | (cc << 3)];
            push_word(&mut out, word(target)?);
            Ok(out)
        }
        _ => Err(form_error("JP", op)),
    }
}

fn assemble_jr(op: &[&str], pc: u16) -> Result<Vec<u8>, AsmError> {
    match op {
        [target] => Ok(vec![0x18, relative(target, pc)?]),
        [condition, target] => {
            let cc = match *condition {
                "NZ" => 0,
                "Z" => 1,
                "NC" => 2,
                "C" => 3,
                _ => {
                    return Err(AsmError::new(format!(
                        "JR condition must be NZ, Z, NC, or C, got '{}'",
                        condition
                    )));
                }
            };
            Ok(vec![0x20 | (cc << 3), relative(target, pc)?])
        }
        _ => Err(form_error("JR", op)),
    }
}

fn assemble_call(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    let (opcode, target) = match op {
        [target] => (0xCD, *target),
        [condition, target] => {
            let cc = condition_code(condition)
                .ok_or_else(|| AsmError::new(format!("invalid CALL condition '{}'", condition)))?;
            (0xC4 | (cc << 3), *target)
        }
        _ => return Err(form_error("CALL", op)),
    };
    let mut out = vec![opcode];
    push_word(&mut out, word(target)?);
    Ok(out)
}

fn assemble_ret(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    match op {
        [] => Ok(vec![0xC9]),
        [condition] => {
            let cc = condition_code(condition)
                .ok_or_else(|| AsmError::new(format!("invalid RET condition '{}'", condition)))?;
            Ok(vec![0xC0 | (cc << 3)])
        }
        _ => Err(form_error("RET", op)),
    }
}

fn assemble_rst(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    let address = number(op[0])?;
    if !(0..=0x38).contains(&address) || address % 8 != 0 {
        return Err(AsmError::new("RST address must be 00H, 08H, ..., or 38H"));
    }
    Ok(vec![0xC7 | address as u8])
}

fn assemble_push_pop(op: &[&str], push: bool) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    if let Some(prefix) = index_prefix(op[0]) {
        return Ok(vec![prefix, if push { 0xE5 } else { 0xE1 }]);
    }
    let p = reg16_stack(op[0]).ok_or_else(|| {
        AsmError::new(format!(
            "{} requires BC, DE, HL, AF, IX, or IY",
            if push { "PUSH" } else { "POP" }
        ))
    })?;
    Ok(vec![(if push { 0xC5 } else { 0xC1 }) | (p << 4)])
}

fn assemble_ex(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    match (op[0], op[1]) {
        ("AF", "AF'") => Ok(vec![0x08]),
        ("DE", "HL") => Ok(vec![0xEB]),
        ("(SP)", "HL") => Ok(vec![0xE3]),
        ("(SP)", "IX") => Ok(vec![0xDD, 0xE3]),
        ("(SP)", "IY") => Ok(vec![0xFD, 0xE3]),
        _ => Err(form_error("EX", op)),
    }
}

fn assemble_in(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    if op[0] == "A" {
        if let Some(port) = parenthesized_byte(op[1]) {
            if let Ok(port) = port {
                return Ok(vec![0xDB, port]);
            }
        }
    }
    if op[1] == "(C)" {
        let r = match op[0] {
            "F" => 6,
            value => reg8(value)
                .filter(|r| *r != 6)
                .ok_or_else(|| AsmError::new(format!("invalid IN register '{}'", value)))?,
        };
        return Ok(vec![0xED, 0x40 | (r << 3)]);
    }
    Err(form_error("IN", op))
}

fn assemble_out(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    if op[1] == "A" {
        if let Some(port) = parenthesized_byte(op[0]) {
            if let Ok(port) = port {
                return Ok(vec![0xD3, port]);
            }
        }
    }
    if op[0] == "(C)" {
        if op[1] == "0" {
            return Ok(vec![0xED, 0x71]);
        }
        let r = reg8(op[1])
            .filter(|r| *r != 6)
            .ok_or_else(|| AsmError::new(format!("invalid OUT register '{}'", op[1])))?;
        return Ok(vec![0xED, 0x41 | (r << 3)]);
    }
    Err(form_error("OUT", op))
}

fn assemble_im(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    let opcode = match number(op[0])? {
        0 => 0x46,
        1 => 0x56,
        2 => 0x5E,
        _ => return Err(AsmError::new("IM mode must be 0, 1, or 2")),
    };
    Ok(vec![0xED, opcode])
}

fn assemble_bit(op: &[&str], group: u8) -> Result<Vec<u8>, AsmError> {
    // RES/SET b,(IX+d),r also copy the result into r (undocumented).
    if group != 1 && op.len() == 3 {
        let bit = number(op[0])?;
        if !(0..=7).contains(&bit) {
            return Err(AsmError::new("bit number must be between 0 and 7"));
        }
        if let (Some((prefix, d)), Some(r)) = (indexed(op[1])?, copy_reg8(op[2])) {
            return Ok(vec![
                prefix,
                0xCB,
                d as u8,
                (group << 6) | ((bit as u8) << 3) | r,
            ]);
        }
        return Err(form_error(if group == 2 { "RES" } else { "SET" }, op));
    }
    expect_count(op, 2)?;
    let bit = number(op[0])?;
    if !(0..=7).contains(&bit) {
        return Err(AsmError::new("bit number must be between 0 and 7"));
    }
    let operation = (group << 6) | ((bit as u8) << 3);
    if let Some(r) = reg8(op[1]) {
        return Ok(vec![0xCB, operation | r]);
    }
    if let Some((prefix, d)) = indexed(op[1])? {
        return Ok(vec![prefix, 0xCB, d as u8, operation | 6]);
    }
    Err(form_error(
        match group {
            1 => "BIT",
            2 => "RES",
            _ => "SET",
        },
        op,
    ))
}

fn assemble_rotate(op: &[&str], operation: u8) -> Result<Vec<u8>, AsmError> {
    // Rotate/shift (IX+d),r also copies the result into r (undocumented).
    if op.len() == 2 {
        if let (Some((prefix, d)), Some(r)) = (indexed(op[0])?, copy_reg8(op[1])) {
            return Ok(vec![prefix, 0xCB, d as u8, (operation << 3) | r]);
        }
        return Err(AsmError::new(format!(
            "invalid rotate/shift operands '{},{}'",
            op[0], op[1]
        )));
    }
    expect_count(op, 1)?;
    if let Some(r) = reg8(op[0]) {
        return Ok(vec![0xCB, (operation << 3) | r]);
    }
    if let Some((prefix, d)) = indexed(op[0])? {
        return Ok(vec![prefix, 0xCB, d as u8, (operation << 3) | 6]);
    }
    Err(AsmError::new(format!(
        "invalid rotate/shift operand '{}'",
        op[0]
    )))
}

fn assemble_fixed(mnemonic: &str, op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 0)?;
    let bytes: &[u8] = match mnemonic {
        "NOP" => &[0x00],
        "EXX" => &[0xD9],
        "HALT" => &[0x76],
        "RLCA" => &[0x07],
        "RRCA" => &[0x0F],
        "RLA" => &[0x17],
        "RRA" => &[0x1F],
        "DAA" => &[0x27],
        "CPL" => &[0x2F],
        "SCF" => &[0x37],
        "CCF" => &[0x3F],
        "DI" => &[0xF3],
        "EI" => &[0xFB],
        "NEG" => &[0xED, 0x44],
        "RETN" => &[0xED, 0x45],
        "RETI" => &[0xED, 0x4D],
        "RRD" => &[0xED, 0x67],
        "RLD" => &[0xED, 0x6F],
        "LDI" => &[0xED, 0xA0],
        "CPI" => &[0xED, 0xA1],
        "INI" => &[0xED, 0xA2],
        "OUTI" => &[0xED, 0xA3],
        "LDD" => &[0xED, 0xA8],
        "CPD" => &[0xED, 0xA9],
        "IND" => &[0xED, 0xAA],
        "OUTD" => &[0xED, 0xAB],
        "LDIR" => &[0xED, 0xB0],
        "CPIR" => &[0xED, 0xB1],
        "INIR" => &[0xED, 0xB2],
        "OTIR" => &[0xED, 0xB3],
        "LDDR" => &[0xED, 0xB8],
        "CPDR" => &[0xED, 0xB9],
        "INDR" => &[0xED, 0xBA],
        "OTDR" => &[0xED, 0xBB],
        _ => return Err(AsmError::new(format!("unknown mnemonic '{}'", mnemonic))),
    };
    Ok(bytes.to_vec())
}

fn reg8(value: &str) -> Option<u8> {
    match value {
        "B" => Some(0),
        "C" => Some(1),
        "D" => Some(2),
        "E" => Some(3),
        "H" => Some(4),
        "L" => Some(5),
        "(HL)" => Some(6),
        "A" => Some(7),
        _ => None,
    }
}

/// Register target for the undocumented DD/FD CB result copy (no (HL)).
fn copy_reg8(value: &str) -> Option<u8> {
    reg8(value).filter(|&r| r != 6)
}

/// Undocumented IX/IY byte halves as (prefix, H/L register code).
fn index_half(value: &str) -> Option<(u8, u8)> {
    match value {
        "IXH" => Some((0xDD, 4)),
        "IXL" => Some((0xDD, 5)),
        "IYH" => Some((0xFD, 4)),
        "IYL" => Some((0xFD, 5)),
        _ => None,
    }
}

fn reg16(value: &str) -> Option<u8> {
    match value {
        "BC" => Some(0),
        "DE" => Some(1),
        "HL" => Some(2),
        "SP" => Some(3),
        _ => None,
    }
}

fn reg16_stack(value: &str) -> Option<u8> {
    match value {
        "BC" => Some(0),
        "DE" => Some(1),
        "HL" => Some(2),
        "AF" => Some(3),
        _ => None,
    }
}

fn condition_code(value: &str) -> Option<u8> {
    match value {
        "NZ" => Some(0),
        "Z" => Some(1),
        "NC" => Some(2),
        "C" => Some(3),
        "PO" => Some(4),
        "PE" => Some(5),
        "P" => Some(6),
        "M" => Some(7),
        _ => None,
    }
}

fn index_prefix(value: &str) -> Option<u8> {
    match value {
        "IX" => Some(0xDD),
        "IY" => Some(0xFD),
        _ => None,
    }
}

fn indexed(value: &str) -> Result<Option<(u8, i8)>, AsmError> {
    if !value.starts_with('(') || !value.ends_with(')') {
        return Ok(None);
    }
    let inner = &value[1..value.len() - 1];
    let (prefix, rest) = if let Some(rest) = inner.strip_prefix("IX") {
        (0xDD, rest)
    } else if let Some(rest) = inner.strip_prefix("IY") {
        (0xFD, rest)
    } else {
        return Ok(None);
    };

    let displacement = if rest.is_empty() {
        0
    } else {
        let value = number(rest)?;
        i8::try_from(value).map_err(|_| {
            AsmError::new(format!(
                "index displacement '{}' is outside -128..127",
                rest
            ))
        })?
    };
    Ok(Some((prefix, displacement)))
}

fn indirect_word(value: &str) -> Option<u16> {
    if !value.starts_with('(') || !value.ends_with(')') {
        return None;
    }
    word(&value[1..value.len() - 1]).ok()
}

fn parenthesized_byte(value: &str) -> Option<Result<u8, AsmError>> {
    if !value.starts_with('(') || !value.ends_with(')') {
        return None;
    }
    Some(byte(&value[1..value.len() - 1]))
}

fn number(value: &str) -> Result<i32, AsmError> {
    let value = value.replace('_', "");
    let (sign, unsigned) = if let Some(rest) = value.strip_prefix('-') {
        (-1, rest)
    } else if let Some(rest) = value.strip_prefix('+') {
        (1, rest)
    } else {
        (1, value.as_str())
    };
    if unsigned.is_empty() {
        return Err(AsmError::new(format!("invalid number '{}'", value)));
    }

    let (digits, radix) = if let Some(rest) = unsigned.strip_prefix("0X") {
        (rest, 16)
    } else if let Some(rest) = unsigned.strip_prefix('$') {
        (rest, 16)
    } else if let Some(rest) = unsigned.strip_suffix('H') {
        (rest, 16)
    } else if let Some(rest) = unsigned.strip_prefix("0B") {
        (rest, 2)
    } else if let Some(rest) = unsigned.strip_suffix('B') {
        (rest, 2)
    } else {
        (unsigned, 10)
    };

    i32::from_str_radix(digits, radix)
        .map(|number| sign * number)
        .map_err(|_| AsmError::new(format!("invalid number '{}'", value)))
}

fn eval_byte(
    value: &str,
    pc: u16,
    symbols: &BTreeMap<String, u16>,
    mappings: &BTreeMap<String, AsmMapping>,
    allow_undefined: bool,
    unknown_value: u16,
) -> Result<u8, AsmError> {
    let value_number = eval_expr(value, pc, symbols, mappings, allow_undefined, unknown_value)?;
    u8::try_from(value_number)
        .map_err(|_| AsmError::new(format!("byte '{}' is outside 0..255", value)))
}

fn eval_word(
    value: &str,
    pc: u16,
    symbols: &BTreeMap<String, u16>,
    mappings: &BTreeMap<String, AsmMapping>,
    allow_undefined: bool,
    unknown_value: u16,
) -> Result<u16, AsmError> {
    let value_number = eval_expr(value, pc, symbols, mappings, allow_undefined, unknown_value)?;
    u16::try_from(value_number)
        .map_err(|_| AsmError::new(format!("word '{}' is outside 0..65535", value)))
}

fn eval_nonnegative(
    value: &str,
    pc: u16,
    symbols: &BTreeMap<String, u16>,
    mappings: &BTreeMap<String, AsmMapping>,
    allow_undefined: bool,
    unknown_value: u16,
) -> Result<usize, AsmError> {
    let value_number = eval_expr(value, pc, symbols, mappings, allow_undefined, unknown_value)?;
    usize::try_from(value_number)
        .map_err(|_| AsmError::new(format!("count '{}' must be non-negative", value)))
}

/// Evaluate an expression with C-like precedence.
///
/// With `allow_undefined` (pass 1), an unknown symbol or mapping makes the
/// whole expression evaluate to `unknown_value`, and arithmetic errors that
/// may be caused by the placeholder are suppressed.
fn eval_expr(
    value: &str,
    pc: u16,
    symbols: &BTreeMap<String, u16>,
    mappings: &BTreeMap<String, AsmMapping>,
    allow_undefined: bool,
    unknown_value: u16,
) -> Result<i32, AsmError> {
    let text = normalize(value);
    if text.is_empty() {
        return Err(AsmError::new("empty expression"));
    }
    let mut parser = ExprParser {
        text: &text,
        pos: 0,
        pc,
        symbols,
        mappings,
        allow_undefined,
        undefined: false,
    };
    let result = parser.parse_binary(0)?;
    if parser.pos != text.len() {
        return Err(AsmError::new(format!("invalid expression '{}'", text)));
    }
    if parser.undefined {
        return Ok(unknown_value as i32);
    }
    Ok(result)
}

/// Binary operators from lowest to highest precedence level.
const BINARY_LEVELS: [&[&str]; 6] = [
    &["|"],
    &["^"],
    &["&"],
    &["<<", ">>"],
    &["+", "-"],
    &["*", "/", "%"],
];

struct ExprParser<'a> {
    text: &'a str,
    pos: usize,
    pc: u16,
    symbols: &'a BTreeMap<String, u16>,
    mappings: &'a BTreeMap<String, AsmMapping>,
    allow_undefined: bool,
    undefined: bool,
}

impl<'a> ExprParser<'a> {
    fn rest(&self) -> &str {
        &self.text[self.pos..]
    }

    fn invalid(&self) -> AsmError {
        AsmError::new(format!("invalid expression '{}'", self.text))
    }

    fn parse_binary(&mut self, level: usize) -> Result<i32, AsmError> {
        if level == BINARY_LEVELS.len() {
            return self.parse_unary();
        }
        let mut left = self.parse_binary(level + 1)?;
        loop {
            let rest = self.rest();
            let Some(op) = BINARY_LEVELS[level]
                .iter()
                .copied()
                .find(|op| rest.starts_with(op))
            else {
                return Ok(left);
            };
            self.pos += op.len();
            let right = self.parse_binary(level + 1)?;
            left = self.apply(op, left, right)?;
        }
    }

    fn apply(&self, op: &str, left: i32, right: i32) -> Result<i32, AsmError> {
        Ok(match op {
            "|" => left | right,
            "^" => left ^ right,
            "&" => left & right,
            "+" => left.wrapping_add(right),
            "-" => left.wrapping_sub(right),
            "*" => left.wrapping_mul(right),
            "<<" | ">>" => {
                let Ok(count) = u32::try_from(right) else {
                    return self.arith_error(format!("shift count {right} is negative"));
                };
                if count > 31 {
                    return self.arith_error(format!("shift count {right} is larger than 31"));
                }
                if op == "<<" {
                    left.wrapping_shl(count)
                } else {
                    left >> count
                }
            }
            "/" | "%" => {
                if right == 0 {
                    return self.arith_error(format!("division by zero in '{}'", self.text));
                }
                if op == "/" {
                    left.wrapping_div(right)
                } else {
                    left.wrapping_rem(right)
                }
            }
            _ => unreachable!("BINARY_LEVELS lists every operator"),
        })
    }

    /// Report an arithmetic error, unless an undefined symbol in pass 1 may
    /// have caused it.
    fn arith_error(&self, message: String) -> Result<i32, AsmError> {
        if self.undefined {
            Ok(0)
        } else {
            Err(AsmError::new(message))
        }
    }

    fn parse_unary(&mut self) -> Result<i32, AsmError> {
        let Some(ch) = self.rest().chars().next() else {
            return Err(self.invalid());
        };
        match ch {
            '+' => {
                self.pos += 1;
                self.parse_unary()
            }
            '-' => {
                self.pos += 1;
                Ok(self.parse_unary()?.wrapping_neg())
            }
            '~' => {
                self.pos += 1;
                Ok(!self.parse_unary()?)
            }
            '!' => {
                self.pos += 1;
                Ok((self.parse_unary()? == 0) as i32)
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<i32, AsmError> {
        if self.rest().starts_with('(') {
            self.pos += 1;
            let value = self.parse_binary(0)?;
            if !self.rest().starts_with(')') {
                return Err(self.invalid());
            }
            self.pos += 1;
            return Ok(value);
        }

        let word = self.take_word();
        if word.is_empty() {
            return Err(self.invalid());
        }
        if self.rest().starts_with('(') {
            return self.parse_function(word);
        }

        let value = self.word_value(word)?;
        if self.rest().starts_with('@') {
            self.pos += 1;
            let name = self.take_word();
            if name.is_empty() || !is_label_like(name) || self.rest().starts_with('@') {
                return Err(AsmError::new(format!(
                    "invalid mapped expression '{}'",
                    self.text
                )));
            }
            let Some(mapping) = self.mappings.get(name) else {
                if self.allow_undefined {
                    self.undefined = true;
                    return Ok(0);
                }
                return Err(AsmError::new(format!("unknown mapping '{}'", name)));
            };
            return Ok(value
                .wrapping_sub(mapping.source_base as i32)
                .wrapping_add(mapping.mapped_base as i32));
        }
        Ok(value)
    }

    /// Take a number, symbol, or `$` token.
    fn take_word(&mut self) -> &'a str {
        let text = self.text;
        let rest = &text[self.pos..];
        let mut len = 0;
        for (index, ch) in rest.char_indices() {
            let word_char = ch.is_ascii_alphanumeric() || ch == '_' || ch == '.';
            if !(word_char || (index == 0 && ch == '$')) {
                break;
            }
            len = index + ch.len_utf8();
        }
        let start = self.pos;
        self.pos += len;
        &text[start..start + len]
    }

    fn word_value(&mut self, word: &str) -> Result<i32, AsmError> {
        if word == "$" {
            return Ok(self.pc as i32);
        }
        // Numbers win over symbols, as before: `C000H` is a number.
        if let Ok(value) = number(word) {
            return Ok(value);
        }
        if word.starts_with('$') || word.starts_with(|ch: char| ch.is_ascii_digit()) {
            return number(word);
        }
        if let Some(value) = self.symbols.get(word) {
            return Ok(*value as i32);
        }
        if self.allow_undefined && is_label_like(word) {
            self.undefined = true;
            return Ok(0);
        }
        Err(AsmError::new(format!("unknown symbol '{}'", word)))
    }

    fn parse_function(&mut self, name: &str) -> Result<i32, AsmError> {
        self.pos += 1;
        let argument = self.parse_binary(0)?;
        if !self.rest().starts_with(')') {
            return Err(self.invalid());
        }
        self.pos += 1;
        match name {
            "LOW" => Ok(argument & 0xFF),
            "HIGH" => Ok((argument >> 8) & 0xFF),
            _ => Err(AsmError::new(format!("unknown function '{}'", name))),
        }
    }
}

fn is_label_like(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '.')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
}

fn byte(value: &str) -> Result<u8, AsmError> {
    let value_number = number(value)?;
    u8::try_from(value_number)
        .map_err(|_| AsmError::new(format!("byte '{}' is outside 0..255", value)))
}

fn word(value: &str) -> Result<u16, AsmError> {
    let value_number = number(value)?;
    u16::try_from(value_number)
        .map_err(|_| AsmError::new(format!("word '{}' is outside 0..65535", value)))
}

fn relative(target: &str, pc: u16) -> Result<u8, AsmError> {
    let target = word(target)?;
    let next = pc.wrapping_add(2);
    let displacement = target.wrapping_sub(next) as i16;
    if !(-128..=127).contains(&displacement) {
        return Err(AsmError::new(format!(
            "relative target {:04X}H is out of range from {:04X}H",
            target, pc
        )));
    }
    Ok(displacement as i8 as u8)
}

fn push_word(out: &mut Vec<u8>, value: u16) {
    out.push(value as u8);
    out.push((value >> 8) as u8);
}

fn expect_count(op: &[&str], count: usize) -> Result<(), AsmError> {
    if op.len() == count {
        Ok(())
    } else {
        Err(AsmError::new(format!(
            "expected {} operand{}, got {}",
            count,
            if count == 1 { "" } else { "s" },
            op.len()
        )))
    }
}

fn form_error(mnemonic: &str, op: &[&str]) -> AsmError {
    let suffix = if op.is_empty() {
        String::new()
    } else {
        format!(" {}", op.join(","))
    };
    AsmError::new(format!("unsupported instruction '{}{}'", mnemonic, suffix))
}

/// Parse `rtvc-asm-v1` TOML produced by `rtvc-asm` or `rtvc-c80 --emit-segments`.
#[cfg(feature = "asm-toml")]
pub fn parse_rtvc_asm_v1(text: &str) -> Result<AssembledProgram, String> {
    let value: toml::Value =
        toml::from_str(text).map_err(|err| format!("invalid rtvc-asm-v1 TOML: {err}"))?;
    let format = value.get("format").and_then(|v| v.as_str()).unwrap_or("");
    if format != "rtvc-asm-v1" {
        return Err(format!("unsupported assembler format '{format}'"));
    }
    let origin = toml_u16(value.get("origin"), "origin")?;
    let next_addr = toml_u16(value.get("next_addr"), "next_addr")?;
    let mut symbols = BTreeMap::new();
    if let Some(table) = value.get("symbols").and_then(|v| v.as_table()) {
        for (name, raw) in table {
            symbols.insert(name.clone(), toml_u16(Some(raw), name)?);
        }
    }
    let mut segments = Vec::new();
    if let Some(rows) = value.get("segments").and_then(|v| v.as_array()) {
        for (i, row) in rows.iter().enumerate() {
            let addr = toml_u16(row.get("addr"), &format!("segments[{i}].addr"))?;
            let bytes = match row.get("bytes").and_then(|v| v.as_array()) {
                Some(items) => items
                    .iter()
                    .enumerate()
                    .map(|(j, item)| toml_u8(item, &format!("segments[{i}].bytes[{j}]")))
                    .collect::<Result<Vec<_>, _>>()?,
                None => {
                    return Err(format!("segments[{i}].bytes is missing"));
                }
            };
            if let Some(len) = row.get("len").and_then(|v| v.as_integer()) {
                if len as usize != bytes.len() {
                    return Err(format!(
                        "segments[{i}].len {len} does not match {} bytes",
                        bytes.len()
                    ));
                }
            }
            segments.push(AssembledSegment { addr, bytes });
        }
    }
    let mut breakpoints = Vec::new();
    if let Some(rows) = value.get("breakpoints").and_then(|v| v.as_array()) {
        for (i, row) in rows.iter().enumerate() {
            let addr = toml_u16(row.get("addr"), &format!("breakpoints[{i}].addr"))?;
            let index = |key: &str| {
                row.get(key)
                    .and_then(|v| v.as_integer())
                    .and_then(|v| usize::try_from(v).ok())
                    .unwrap_or(0)
            };
            breakpoints.push(AsmBreakpoint {
                addr,
                file: index("file"),
                line: index("line"),
            });
        }
    }
    let flat: Vec<u8> = segments
        .iter()
        .flat_map(|s| s.bytes.iter().copied())
        .collect();
    Ok(AssembledProgram {
        origin,
        bytes: flat,
        segments,
        symbols,
        mappings: Vec::new(),
        lines: Vec::new(),
        next_addr,
        files: Vec::new(),
        breakpoints,
    })
}

#[cfg(feature = "asm-toml")]
fn toml_u16(value: Option<&toml::Value>, name: &str) -> Result<u16, String> {
    let Some(value) = value else {
        return Err(format!("{name} is missing"));
    };
    let n = value
        .as_integer()
        .ok_or_else(|| format!("{name} is not an integer"))?;
    u16::try_from(n).map_err(|_| format!("{name} {n} is not a u16"))
}

#[cfg(feature = "asm-toml")]
fn toml_u8(value: &toml::Value, name: &str) -> Result<u8, String> {
    let n = value
        .as_integer()
        .ok_or_else(|| format!("{name} is not an integer"))?;
    u8::try_from(n).map_err(|_| format!("{name} {n} is not a u8"))
}

#[cfg(test)]
#[path = "asm_tests.rs"]
mod tests;
