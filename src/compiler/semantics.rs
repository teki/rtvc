//! Name resolution, scalar typing, definite assignment, and IR lowering.

use super::ast::*;
use super::diagnostic::{DiagCode, Diagnostic};
use super::inline_asm::{self, RES_GP, clobber_mask, gpr_mask, out_mask};
use super::ir::*;
use super::source::{NodeId, SourceSpan};
use super::types::{ArrayElem, ArrayType, CType, PtrType, StructDef, StructField, StructId};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
pub struct StackLayout {
    pub base: u16,
    pub size: u16,
}

#[derive(Debug, Clone, Default)]
pub struct UnitExports {
    pub symbols: HashMap<String, ExportedSymbol>,
}

#[derive(Debug, Clone)]
pub enum ExportedSymbol {
    Function {
        id: FuncId,
        params: Vec<CType>,
        conv: CallConv,
        ret: CType,
        is_pub: bool,
    },
    Value {
        id: GlobalId,
        ty: CType,
        is_pub: bool,
        is_const: bool,
        bits: Option<u16>,
    },
}

impl ExportedSymbol {
    fn is_pub(&self) -> bool {
        match self {
            Self::Function { is_pub, .. } | Self::Value { is_pub, .. } => *is_pub,
        }
    }
}

pub fn analyze_unit(unit: &TranslationUnit, diagnostics: &mut Vec<Diagnostic>) -> TypedProgram {
    analyze_unit_in_project(unit, "", None, None, diagnostics).0
}

pub fn analyze_unit_in_project(
    unit: &TranslationUnit,
    unit_name: &str,
    exports: Option<&HashMap<String, UnitExports>>,
    stack: Option<StackLayout>,
    diagnostics: &mut Vec<Diagnostic>,
) -> (TypedProgram, Vec<(String, String, SourceSpan)>) {
    let mut analyzer = Analyzer {
        diagnostics,
        symbols: HashMap::new(),
        func_ids: HashMap::new(),
        calls: Vec::new(),
        next_vreg: 1,
        next_block: 1,
        next_anon: 0,
        anon_globals: Vec::new(),
        str_lens: HashMap::new(),
        imports: HashSet::new(),
        unit_name: unit_name.to_string(),
        exports,
        stack,
        structs: Vec::new(),
        struct_ids: HashMap::new(),
        pending_structs: HashMap::new(),
        layouting: HashSet::new(),
    };
    analyzer.collect(unit);
    let program = analyzer.check_unit(unit);
    let calls = analyzer.calls.clone();
    if exports.is_none() {
        analyzer.check_recursion();
    }
    (program, calls)
}

pub fn collect_exports(unit: &TranslationUnit, diagnostics: &mut Vec<Diagnostic>) -> UnitExports {
    let mut analyzer = Analyzer {
        diagnostics,
        symbols: HashMap::new(),
        func_ids: HashMap::new(),
        calls: Vec::new(),
        next_vreg: 1,
        next_block: 1,
        next_anon: 0,
        anon_globals: Vec::new(),
        str_lens: HashMap::new(),
        imports: HashSet::new(),
        unit_name: String::new(),
        exports: None,
        stack: None,
        structs: Vec::new(),
        struct_ids: HashMap::new(),
        pending_structs: HashMap::new(),
        layouting: HashSet::new(),
    };
    analyzer.collect(unit);
    let mut symbols = HashMap::new();
    for (name, sym) in &analyzer.symbols {
        match &sym.kind {
            SymbolKind::Function { params, conv } => {
                symbols.insert(
                    name.clone(),
                    ExportedSymbol::Function {
                        id: FuncId(sym.id),
                        params: params.clone(),
                        conv: *conv,
                        ret: sym.ty,
                        is_pub: sym.is_pub,
                    },
                );
            }
            SymbolKind::Global | SymbolKind::Const => {
                symbols.insert(
                    name.clone(),
                    ExportedSymbol::Value {
                        id: GlobalId(sym.id),
                        ty: sym.ty,
                        is_pub: sym.is_pub,
                        is_const: sym.is_const,
                        bits: None,
                    },
                );
            }
            SymbolKind::Struct => {}
        }
    }
    UnitExports { symbols }
}

pub fn apply_const_bits(exports: &mut UnitExports, program: &TypedProgram) {
    for global in &program.globals {
        if !global.is_const {
            continue;
        }
        if let Some(ExportedSymbol::Value { bits, .. }) = exports.symbols.get_mut(&global.name) {
            *bits = global.init;
        }
    }
}

pub fn check_call_graph(calls: &[(String, String, SourceSpan)], diagnostics: &mut Vec<Diagnostic>) {
    let mut adj: HashMap<String, Vec<(String, SourceSpan)>> = HashMap::new();
    let mut starts = Vec::new();
    for (from, to, span) in calls {
        adj.entry(from.clone())
            .or_default()
            .push((to.clone(), *span));
        if !starts.contains(from) {
            starts.push(from.clone());
        }
    }
    for start in &starts {
        if let Some(span) = cycle_from(&adj, start) {
            diagnostics.push(Diagnostic::error(
                DiagCode::TyRecursion,
                span,
                format!("recursive call graph involving '{start}' is not allowed"),
            ));
        }
    }
}

struct Symbol {
    kind: SymbolKind,
    ty: CType,
    id: NodeId,
    #[allow(dead_code)]
    span: SourceSpan,
    is_const: bool,
    is_pub: bool,
    const_bits: Option<u16>,
}

enum SymbolKind {
    Function {
        params: Vec<CType>,
        #[allow(dead_code)]
        conv: CallConv,
    },
    Global,
    Const,
    Struct,
}

enum QSym {
    Function {
        id: FuncId,
        params: Vec<CType>,
        #[allow(dead_code)]
        conv: CallConv,
        ret: CType,
        qname: String,
    },
    Value {
        id: GlobalId,
        ty: CType,
        is_const: bool,
        bits: Option<u16>,
    },
}

struct Analyzer<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
    symbols: HashMap<String, Symbol>,
    func_ids: HashMap<String, FuncId>,
    calls: Vec<(String, String, SourceSpan)>,
    next_vreg: u32,
    next_block: u32,
    next_anon: u32,
    anon_globals: Vec<TypedGlobal>,
    str_lens: HashMap<NodeId, u8>,
    imports: HashSet<String>,
    unit_name: String,
    exports: Option<&'a HashMap<String, UnitExports>>,
    stack: Option<StackLayout>,
    structs: Vec<StructDef>,
    struct_ids: HashMap<String, StructId>,
    pending_structs: HashMap<StructId, StructDecl>,
    layouting: HashSet<StructId>,
}

struct FnCtx {
    ret: CType,
    name: String,
    scopes: Vec<HashMap<String, LocalBind>>,
    assigned: HashSet<NodeId>,
    locals: Vec<TypedLocal>,
    params: Vec<TypedParam>,
    blocks: Vec<IrBlock>,
    current: BlockId,
    loop_stack: Vec<LoopCtx>,
    reachable: bool,
    owner_span: SourceSpan,
}

#[derive(Clone)]
struct LocalBind {
    id: LocalId,
    ty: CType,
    #[allow(dead_code)]
    span: SourceSpan,
    is_const: bool,
    #[allow(dead_code)]
    is_param: bool,
}

struct LoopCtx {
    break_blk: BlockId,
    continue_blk: BlockId,
    #[allow(dead_code)]
    assigned_at_entry: HashSet<NodeId>,
}

#[derive(Clone)]
struct Value {
    ty: CType,
    reg: VReg,
    bits: Option<u16>,
}

enum Place {
    Local {
        id: LocalId,
        ty: CType,
        is_const: bool,
    },
    Global {
        id: GlobalId,
        ty: CType,
        is_const: bool,
    },
    Indirect {
        ptr: VReg,
        ty: CType,
        immutable: bool,
    },
}

enum AsmDest {
    Local(LocalId),
    Global(GlobalId),
    Indirect { ptr: VReg, ty: CType },
}

impl<'a> Analyzer<'a> {
    fn emit(&mut self, code: DiagCode, span: SourceSpan, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(code, span, message));
    }

    fn warn(&mut self, code: DiagCode, span: SourceSpan, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::warning(code, span, message));
    }

    fn type_of(&mut self, ty: &TypeExpr) -> CType {
        match &ty.kind {
            TypeKind::Ptr(inner) => {
                let inner = self.type_of(inner);
                match PtrType::of(inner) {
                    Some(p) => CType::Ptr(p),
                    None => {
                        self.emit(
                            DiagCode::TyMismatch,
                            ty.span,
                            format!("ptr<{}> is not a supported pointer type", inner.as_str()),
                        );
                        CType::Void
                    }
                }
            }
            TypeKind::Named(name) => {
                if let Some(&id) = self.struct_ids.get(&name.name) {
                    CType::Struct(id)
                } else {
                    self.emit(
                        DiagCode::TyUnresolvedName,
                        name.span,
                        format!("unknown type '{}'", name.name),
                    );
                    CType::Void
                }
            }
            _ => CType::from_ast(&ty.kind),
        }
    }

    fn intern_string(
        &mut self,
        bytes: &[u8],
        lit_span: SourceSpan,
        owner: SourceSpan,
    ) -> Option<GlobalId> {
        if bytes.len() > 255 {
            self.emit(
                DiagCode::TyLiteralRange,
                lit_span,
                "str payload is limited to 255 bytes",
            );
            return None;
        }
        self.next_anon += 1;
        let id = NodeId(0x8000_0000 + self.next_anon);
        let mut extra = Vec::with_capacity(bytes.len() + 1);
        extra.push(bytes.len() as u8);
        extra.extend_from_slice(bytes);
        self.str_lens.insert(id, bytes.len() as u8);
        self.anon_globals.push(TypedGlobal {
            id: GlobalId(id),
            name: format!("s{}", self.next_anon),
            ty: CType::Str,
            is_pub: false,
            is_const: true,
            init: None,
            extra,
            span: SourceSpan::new(owner.file, owner.end, owner.end),
        });
        Some(GlobalId(id))
    }

    fn vreg(&mut self) -> VReg {
        let id = VReg(self.next_vreg);
        self.next_vreg += 1;
        id
    }

    fn block_id(&mut self) -> BlockId {
        let id = BlockId(self.next_block);
        self.next_block += 1;
        id
    }

    fn intern_structs(&mut self, unit: &TranslationUnit) {
        for item in &unit.items {
            let Item::Struct(decl) = item else {
                continue;
            };
            if self.symbols.contains_key(&decl.name.name) {
                self.emit(
                    DiagCode::TyDuplicateName,
                    decl.name.span,
                    format!("duplicate name '{}'", decl.name.name),
                );
                continue;
            }
            let id = StructId(decl.id);
            self.symbols.insert(
                decl.name.name.clone(),
                Symbol {
                    kind: SymbolKind::Struct,
                    ty: CType::Struct(id),
                    id: decl.id,
                    span: decl.name.span,
                    is_const: true,
                    is_pub: false,
                    const_bits: None,
                },
            );
            self.struct_ids.insert(decl.name.name.clone(), id);
            self.pending_structs.insert(id, decl.clone());
        }
    }

    fn layout_all_structs(&mut self) {
        let ids: Vec<StructId> = self.pending_structs.keys().copied().collect();
        for id in ids {
            self.ensure_layout(id);
        }
    }

    fn ensure_layout(&mut self, id: StructId) {
        if self.structs.iter().any(|s| s.id == id) {
            return;
        }
        if !self.layouting.insert(id) {
            if let Some(decl) = self.pending_structs.get(&id) {
                self.emit(
                    DiagCode::TyMismatch,
                    decl.name.span,
                    format!(
                        "recursive by-value layout of struct '{}' is not allowed",
                        decl.name.name
                    ),
                );
            }
            return;
        }
        let Some(decl) = self.pending_structs.get(&id).cloned() else {
            self.layouting.remove(&id);
            return;
        };
        if decl.fields.is_empty() {
            self.emit(
                DiagCode::TyMismatch,
                decl.name.span,
                "empty structs are not allowed",
            );
            self.layouting.remove(&id);
            return;
        }
        let mut fields = Vec::new();
        let mut offset: u32 = 0;
        let mut failed = false;
        let mut seen_names = HashSet::new();
        for field in &decl.fields {
            if !seen_names.insert(field.name.name.clone()) {
                self.emit(
                    DiagCode::TyDuplicateName,
                    field.name.span,
                    format!("duplicate field '{}'", field.name.name),
                );
                failed = true;
                continue;
            }
            let Some(ty) = self.field_type(field) else {
                failed = true;
                continue;
            };
            if ty == CType::Str {
                self.emit(
                    DiagCode::TyMismatch,
                    field.span,
                    "str struct fields are not supported",
                );
                failed = true;
                continue;
            }
            if let CType::Struct(inner) = ty {
                self.ensure_layout(inner);
            }
            if let CType::Array(arr) = ty {
                if let ArrayElem::Struct(inner) = arr.elem {
                    self.ensure_layout(inner);
                }
            }
            let Some(size) = ty.storage_size(&self.structs) else {
                self.emit(
                    DiagCode::TyMismatch,
                    field.span,
                    format!("incomplete field type '{}'", ty.as_str()),
                );
                failed = true;
                continue;
            };
            if offset + u32::from(size) > 65535 {
                self.emit(
                    DiagCode::TyLiteralRange,
                    field.span,
                    "struct is larger than 65535 bytes",
                );
                failed = true;
                continue;
            }
            fields.push(StructField {
                name: field.name.name.clone(),
                ty,
                offset: offset as u16,
                size,
                span: field.span,
            });
            offset += u32::from(size);
        }
        self.layouting.remove(&id);
        if failed {
            return;
        }
        self.structs.push(StructDef {
            id,
            name: decl.name.name.clone(),
            fields,
            size: offset as u16,
            span: decl.span,
        });
    }

    fn field_type(&mut self, field: &StructFieldDecl) -> Option<CType> {
        let mut ty = self.type_of(&field.ty);
        if let Some(len_expr) = &field.array_len {
            let Some(len) = self.eval_const_expr(len_expr, Some(CType::U16)) else {
                self.emit(
                    DiagCode::TyMismatch,
                    len_expr.span,
                    "array length must be a compile-time integer",
                );
                return None;
            };
            if len == 0 {
                self.emit(
                    DiagCode::TyLiteralRange,
                    len_expr.span,
                    "array length must be at least 1",
                );
                return None;
            }
            let Some(elem) = ArrayElem::from_ctype(ty) else {
                self.emit(
                    DiagCode::TyMismatch,
                    field.ty.span,
                    "arrays of this element type are not supported",
                );
                return None;
            };
            ty = CType::Array(ArrayType { elem, len });
        }
        if ty == CType::Void {
            self.emit(
                DiagCode::TyVoidValue,
                field.ty.span,
                "void is not a value type",
            );
            return None;
        }
        Some(ty)
    }

    fn collect(&mut self, unit: &TranslationUnit) {
        self.intern_structs(unit);
        self.layout_all_structs();
        for item in &unit.items {
            match item {
                Item::Struct(_) => {}
                Item::Function(func) => {
                    let ret = self.type_of(&func.return_ty);
                    let params: Vec<CType> =
                        func.params.iter().map(|p| self.type_of(&p.ty)).collect();
                    if self.symbols.contains_key(&func.name.name) {
                        self.emit(
                            DiagCode::TyDuplicateName,
                            func.name.span,
                            format!("duplicate name '{}'", func.name.name),
                        );
                        continue;
                    }
                    self.symbols.insert(
                        func.name.name.clone(),
                        Symbol {
                            kind: SymbolKind::Function {
                                params: params.clone(),
                                conv: func.conv,
                            },
                            ty: ret,
                            id: func.id,
                            span: func.name.span,
                            is_const: false,
                            is_pub: func.is_pub,
                            const_bits: None,
                        },
                    );
                    self.func_ids
                        .insert(self.func_key(&func.name.name), FuncId(func.id));
                }
                Item::Decl(decl) => {
                    let mut ty = self.type_of(&decl.ty);
                    if let Some(len_expr) = &decl.array_len {
                        if let ExprKind::Int(lit) = &len_expr.kind {
                            if let (Some(n), Some(elem)) = (
                                lit.value.and_then(|v| u16::try_from(v).ok()),
                                ArrayElem::from_ctype(ty),
                            ) {
                                ty = CType::Array(ArrayType { elem, len: n });
                            }
                        }
                    }
                    if self.symbols.contains_key(&decl.name.name) {
                        self.emit(
                            DiagCode::TyDuplicateName,
                            decl.name.span,
                            format!("duplicate name '{}'", decl.name.name),
                        );
                        continue;
                    }
                    if ty == CType::Void {
                        self.emit(
                            DiagCode::TyVoidValue,
                            decl.ty.span,
                            "void is not a value type",
                        );
                    }
                    let kind = if decl.is_const {
                        SymbolKind::Const
                    } else {
                        SymbolKind::Global
                    };
                    self.symbols.insert(
                        decl.name.name.clone(),
                        Symbol {
                            kind,
                            ty,
                            id: decl.id,
                            span: decl.name.span,
                            is_const: decl.is_const,
                            is_pub: decl.is_pub,
                            const_bits: None,
                        },
                    );
                }
                Item::Import(import) => {
                    if import.name.name == "project" {
                        self.emit(
                            DiagCode::TyDuplicateName,
                            import.name.span,
                            "namespace 'project' is built-in and cannot be imported",
                        );
                    } else if let Some(exports) = self.exports {
                        if !exports.contains_key(&import.name.name) {
                            self.emit(
                                DiagCode::TyUnresolvedName,
                                import.name.span,
                                format!("unknown unit '{}'", import.name.name),
                            );
                        }
                    }
                    if !self.imports.insert(import.name.name.clone()) {
                        self.emit(
                            DiagCode::TyDuplicateName,
                            import.name.span,
                            format!("duplicate import '{}'", import.name.name),
                        );
                    }
                }
            }
        }
    }

    fn func_key(&self, name: &str) -> String {
        if self.unit_name.is_empty() {
            name.to_string()
        } else {
            format!("{}::{name}", self.unit_name)
        }
    }

    fn check_unit(&mut self, unit: &TranslationUnit) -> TypedProgram {
        let mut globals = Vec::new();
        let mut functions = Vec::new();
        for item in &unit.items {
            match item {
                Item::Struct(_) => {}
                Item::Decl(decl) => {
                    if let Some(g) = self.check_global(decl) {
                        globals.push(g);
                    }
                }
                Item::Function(func) => {
                    functions.push(self.check_function(func));
                    globals.extend(self.anon_globals.drain(..));
                }
                Item::Import(_) => {}
            }
        }
        TypedProgram {
            structs: self.structs.clone(),
            globals,
            functions,
        }
    }

    fn check_global(&mut self, decl: &VarDecl) -> Option<TypedGlobal> {
        let mut ty = self.type_of(&decl.ty);
        let mut extra = Vec::new();
        if let Some(len_expr) = &decl.array_len {
            let Some(len) = self.eval_const_expr(len_expr, Some(CType::U16)) else {
                self.emit(
                    DiagCode::TyMismatch,
                    len_expr.span,
                    "array length must be a compile-time integer",
                );
                return None;
            };
            if len == 0 {
                self.emit(
                    DiagCode::TyLiteralRange,
                    len_expr.span,
                    "array length must be at least 1",
                );
                return None;
            }
            let Some(elem) = ArrayElem::from_ctype(ty) else {
                self.emit(
                    DiagCode::TyMismatch,
                    decl.ty.span,
                    "arrays of this element type are not supported",
                );
                return None;
            };
            let Some(stride) = elem.stride(&self.structs) else {
                self.emit(
                    DiagCode::TyMismatch,
                    decl.ty.span,
                    "array element type is incomplete",
                );
                return None;
            };
            let size = u32::from(stride) * u32::from(len);
            if size > 65535 {
                self.emit(
                    DiagCode::TyLiteralRange,
                    decl.span,
                    "array is larger than 65535 bytes",
                );
                return None;
            }
            ty = CType::Array(ArrayType { elem, len });
            extra = vec![0; size as usize];
        } else if let CType::Struct(id) = ty {
            let Some(def) = self.structs.iter().find(|s| s.id == id) else {
                self.emit(
                    DiagCode::TyMismatch,
                    decl.ty.span,
                    "struct type is incomplete",
                );
                return None;
            };
            extra = vec![0; usize::from(def.size)];
        }
        if ty == CType::Str {
            match &decl.init {
                Some(Expr {
                    kind: ExprKind::String(bytes),
                    span: init_span,
                    ..
                }) => {
                    if bytes.len() > 255 {
                        self.emit(
                            DiagCode::TyLiteralRange,
                            *init_span,
                            "str payload is limited to 255 bytes",
                        );
                        return None;
                    }
                    extra.push(bytes.len() as u8);
                    extra.extend_from_slice(bytes);
                    self.str_lens.insert(decl.id, bytes.len() as u8);
                }
                Some(expr) => {
                    self.emit(
                        DiagCode::TyMismatch,
                        expr.span,
                        "str globals require a string literal initializer",
                    );
                    return None;
                }
                None => {
                    self.emit(
                        DiagCode::TyMismatch,
                        decl.span,
                        "str globals require a string literal initializer",
                    );
                    return None;
                }
            }
        }
        if ty == CType::Void {
            return None;
        }
        if matches!(ty, CType::Array(_) | CType::Struct(_)) {
            if decl.is_const {
                self.emit(
                    DiagCode::TyMismatch,
                    decl.span,
                    "const aggregates are not supported; use a mutable global",
                );
            }
            if let Some(expr) = &decl.init {
                if !self.write_static_init(&mut extra, 0, ty, expr) {
                    extra.fill(0);
                }
            }
            if let Some(sym) = self.symbols.get_mut(&decl.name.name) {
                sym.ty = ty;
            }
            return Some(TypedGlobal {
                id: GlobalId(decl.id),
                name: decl.name.name.clone(),
                ty,
                is_pub: decl.is_pub,
                is_const: decl.is_const,
                init: None,
                extra,
                span: decl.span,
            });
        }
        let init = if matches!(ty, CType::Str) {
            None
        } else if let Some(expr) = &decl.init {
            match self.eval_const_expr(expr, Some(ty)) {
                Some(bits) => Some(bits),
                None => {
                    if decl.is_const {
                        self.emit(
                            DiagCode::TyMismatch,
                            expr.span,
                            "const initializer must be a compile-time value",
                        );
                    }
                    None
                }
            }
        } else if decl.is_const {
            self.emit(
                DiagCode::TyMismatch,
                decl.span,
                "const declaration requires an initializer",
            );
            None
        } else {
            Some(0)
        };
        if decl.is_const {
            if let Some(sym) = self.symbols.get_mut(&decl.name.name) {
                sym.const_bits = init;
            }
        }
        Some(TypedGlobal {
            id: GlobalId(decl.id),
            name: decl.name.name.clone(),
            ty,
            is_pub: decl.is_pub,
            is_const: decl.is_const,
            init,
            extra,
            span: decl.span,
        })
    }

    fn write_static_init(&mut self, buf: &mut [u8], offset: usize, ty: CType, expr: &Expr) -> bool {
        let Some(size) = ty.storage_size(&self.structs) else {
            self.emit(
                DiagCode::TyMismatch,
                expr.span,
                format!("cannot initialize incomplete type '{}'", ty.as_str()),
            );
            return false;
        };
        let size = usize::from(size);
        if offset + size > buf.len() {
            self.emit(
                DiagCode::TyMismatch,
                expr.span,
                "initializer does not fit in the destination",
            );
            return false;
        }
        match ty {
            CType::Array(arr) => {
                let ExprKind::InitList { elems } = &expr.kind else {
                    self.emit(
                        DiagCode::TyMismatch,
                        expr.span,
                        "array initializers use positional braces",
                    );
                    return false;
                };
                if elems.len() > usize::from(arr.len) {
                    self.emit(
                        DiagCode::TyLiteralRange,
                        expr.span,
                        "too many array initializers",
                    );
                    return false;
                }
                let Some(stride) = arr.elem.stride(&self.structs) else {
                    return false;
                };
                let mut ok = true;
                for (i, elem) in elems.iter().enumerate() {
                    ok &= self.write_static_init(
                        buf,
                        offset + i * usize::from(stride),
                        arr.elem.to_ctype(),
                        elem,
                    );
                }
                ok
            }
            CType::Struct(id) => {
                let ExprKind::InitList { elems } = &expr.kind else {
                    self.emit(
                        DiagCode::TyMismatch,
                        expr.span,
                        "struct initializers use positional braces",
                    );
                    return false;
                };
                let Some(def) = self.structs.iter().find(|s| s.id == id).cloned() else {
                    self.emit(DiagCode::TyMismatch, expr.span, "struct type is incomplete");
                    return false;
                };
                if elems.len() > def.fields.len() {
                    self.emit(
                        DiagCode::TyLiteralRange,
                        expr.span,
                        "too many struct initializers",
                    );
                    return false;
                }
                let mut ok = true;
                for (field, elem) in def.fields.iter().zip(elems.iter()) {
                    ok &= self.write_static_init(
                        buf,
                        offset + usize::from(field.offset),
                        field.ty,
                        elem,
                    );
                }
                ok
            }
            _ => {
                if matches!(expr.kind, ExprKind::InitList { .. }) {
                    self.emit(
                        DiagCode::TyMismatch,
                        expr.span,
                        "brace initializer is only for arrays and structs",
                    );
                    return false;
                }
                let Some(bits) = self.eval_const_expr(expr, Some(ty)) else {
                    self.emit(
                        DiagCode::TyMismatch,
                        expr.span,
                        "static initializer must be a compile-time value",
                    );
                    return false;
                };
                match ty.byte_width() {
                    Some(1) => buf[offset] = bits as u8,
                    Some(2) => {
                        buf[offset] = bits as u8;
                        buf[offset + 1] = (bits >> 8) as u8;
                    }
                    _ => {
                        self.emit(
                            DiagCode::TyMismatch,
                            expr.span,
                            "cannot initialize this type with a scalar",
                        );
                        return false;
                    }
                }
                true
            }
        }
    }

    fn check_function(&mut self, func: &Function) -> TypedFunction {
        let ret = self.type_of(&func.return_ty);
        if ret.is_aggregate() {
            self.emit(
                DiagCode::TyMismatch,
                func.return_ty.span,
                "by-value aggregate returns are not supported; return a pointer",
            );
        }
        let entry = self.block_id();
        let mut ctx = FnCtx {
            ret,
            name: func.name.name.clone(),
            scopes: vec![HashMap::new()],
            assigned: HashSet::new(),
            locals: Vec::new(),
            params: Vec::new(),
            blocks: vec![IrBlock {
                id: entry,
                ops: Vec::new(),
            }],
            current: entry,
            loop_stack: Vec::new(),
            reachable: true,
            owner_span: func.span,
        };
        let mut param_names = HashSet::new();
        for param in &func.params {
            let ty = self.type_of(&param.ty);
            if ty == CType::Void {
                self.emit(
                    DiagCode::TyVoidValue,
                    param.ty.span,
                    "void parameters are not allowed",
                );
            }
            if ty.is_aggregate() {
                self.emit(
                    DiagCode::TyMismatch,
                    param.ty.span,
                    "by-value aggregate parameters are not supported; pass a pointer",
                );
            }
            if !param_names.insert(param.name.name.clone()) {
                self.emit(
                    DiagCode::TyDuplicateName,
                    param.name.span,
                    format!("duplicate parameter '{}'", param.name.name),
                );
            }
            let bind = LocalBind {
                id: LocalId(param.id),
                ty,
                span: param.name.span,
                is_const: false,
                is_param: true,
            };
            ctx.scopes[0].insert(param.name.name.clone(), bind);
            ctx.assigned.insert(param.id);
            ctx.params.push(TypedParam {
                id: LocalId(param.id),
                name: param.name.name.clone(),
                ty,
                span: param.span,
            });
        }
        self.check_block_stmts(&mut ctx, &func.body);
        if ctx.reachable {
            if ret == CType::Void {
                self.emit_op(
                    &mut ctx,
                    IrOp::Return {
                        value: None,
                        span: func.body.span,
                    },
                );
            } else {
                self.emit(
                    DiagCode::TyMissingReturn,
                    func.name.span,
                    format!(
                        "non-void function '{}' must return a value on every path",
                        func.name.name
                    ),
                );
            }
        }
        TypedFunction {
            id: FuncId(func.id),
            name: func.name.name.clone(),
            is_pub: func.is_pub,
            conv: func.conv,
            ret,
            params: ctx.params,
            locals: ctx.locals,
            blocks: ctx.blocks,
            span: func.span,
        }
    }

    fn check_block_stmts(&mut self, ctx: &mut FnCtx, block: &Block) {
        ctx.scopes.push(HashMap::new());
        for stmt in &block.stmts {
            self.check_stmt(ctx, stmt);
        }
        ctx.scopes.pop();
    }

    fn check_stmt(&mut self, ctx: &mut FnCtx, stmt: &Stmt) {
        if !ctx.reachable && !matches!(stmt, Stmt::Error { .. }) {
            return;
        }
        match stmt {
            Stmt::Block(block) => self.check_block_stmts(ctx, block),
            Stmt::Decl(decl) => self.check_local(ctx, decl),
            Stmt::Expr(expr) => {
                let _ = self.check_expr(ctx, &expr.expr, None);
            }
            Stmt::If(if_stmt) => self.check_if(ctx, if_stmt),
            Stmt::While(while_stmt) => self.check_while(ctx, while_stmt),
            Stmt::For(for_stmt) => self.check_for(ctx, for_stmt),
            Stmt::DoWhile(stmt) => self.check_do_while(ctx, stmt),
            Stmt::Return(ret) => self.check_return(ctx, ret),
            Stmt::Break { span, .. } => {
                if let Some(loop_ctx) = ctx.loop_stack.last() {
                    let target = loop_ctx.break_blk;
                    self.emit_op(
                        ctx,
                        IrOp::Jump {
                            blk: target,
                            span: *span,
                        },
                    );
                    ctx.reachable = false;
                } else {
                    self.emit(DiagCode::TyMismatch, *span, "'break' outside of a loop");
                }
            }
            Stmt::Continue { span, .. } => {
                if let Some(loop_ctx) = ctx.loop_stack.last() {
                    let target = loop_ctx.continue_blk;
                    self.emit_op(
                        ctx,
                        IrOp::Jump {
                            blk: target,
                            span: *span,
                        },
                    );
                    ctx.reachable = false;
                } else {
                    self.emit(DiagCode::TyMismatch, *span, "'continue' outside of a loop");
                }
            }
            Stmt::Asm(stmt) => self.check_asm(ctx, stmt),
            Stmt::Error { .. } => {}
        }
    }

    fn check_asm(&mut self, ctx: &mut FnCtx, stmt: &AsmStmt) {
        let mut in_mask = 0u16;
        let mut out_bits = 0u16;
        let mut clobber_bits = 0u16;
        let mut inputs = Vec::new();
        let mut pending_outs: Vec<(IrAsmOutput, AsmDest)> = Vec::new();
        let mut stack: Option<u16> = None;
        let mut failed = false;

        if !stmt.has_header {
            clobber_bits = inline_asm::RES_GP | inline_asm::RES_FLAGS | inline_asm::RES_MEMORY;
        }

        for clause in &stmt.clauses {
            match clause {
                AsmClause::In { reg, expr, span } => {
                    let m = gpr_mask(*reg);
                    if in_mask & m != 0 {
                        self.emit(
                            DiagCode::TyMismatch,
                            *span,
                            format!("asm input '{}' overlaps another input", reg.as_str()),
                        );
                        failed = true;
                    }
                    in_mask |= m;
                    let expected = literal_gpr_context(expr, *reg);
                    let Some(val) = self.check_expr(ctx, expr, expected) else {
                        failed = true;
                        continue;
                    };
                    if !gpr_type_ok(*reg, val.ty) {
                        self.emit(
                            DiagCode::TyMismatch,
                            expr.span,
                            format!(
                                "asm input '{}' needs {}, found {}",
                                reg.as_str(),
                                gpr_expect_desc(*reg),
                                val.ty.as_str()
                            ),
                        );
                        failed = true;
                        continue;
                    }
                    inputs.push((*reg, val.reg));
                }
                AsmClause::Out { reg, dest, span } => {
                    let m = out_mask(*reg);
                    if out_bits & m != 0 {
                        self.emit(
                            DiagCode::TyMismatch,
                            *span,
                            format!("asm output '{}' overlaps another output", reg.as_str()),
                        );
                        failed = true;
                    }
                    out_bits |= m;
                    let Some((dest_info, ty)) = self.eval_asm_dest(ctx, dest, false) else {
                        failed = true;
                        continue;
                    };
                    if !out_type_ok(*reg, ty) {
                        self.emit(
                            DiagCode::TyMismatch,
                            dest.span,
                            format!(
                                "asm output '{}' needs {}, found {}",
                                reg.as_str(),
                                out_expect_desc(*reg),
                                ty.as_str()
                            ),
                        );
                        failed = true;
                        continue;
                    }
                    self.mark_asm_assigned(ctx, &dest_info);
                    let dst = self.vreg();
                    pending_outs.push((
                        IrAsmOutput {
                            dst,
                            ty,
                            kind: out_kind(*reg),
                        },
                        dest_info,
                    ));
                }
                AsmClause::Inout { reg, dest, span } => {
                    let m = gpr_mask(*reg);
                    if in_mask & m != 0 {
                        self.emit(
                            DiagCode::TyMismatch,
                            *span,
                            format!("asm inout '{}' overlaps another input", reg.as_str()),
                        );
                        failed = true;
                    }
                    if out_bits & m != 0 {
                        self.emit(
                            DiagCode::TyMismatch,
                            *span,
                            format!("asm inout '{}' overlaps another output", reg.as_str()),
                        );
                        failed = true;
                    }
                    in_mask |= m;
                    out_bits |= m;
                    let Some((dest_info, ty, loaded)) = self.eval_asm_inout(ctx, dest) else {
                        failed = true;
                        continue;
                    };
                    if !gpr_type_ok(*reg, ty) {
                        self.emit(
                            DiagCode::TyMismatch,
                            dest.span,
                            format!(
                                "asm inout '{}' needs {}, found {}",
                                reg.as_str(),
                                gpr_expect_desc(*reg),
                                ty.as_str()
                            ),
                        );
                        failed = true;
                        continue;
                    }
                    inputs.push((*reg, loaded));
                    self.mark_asm_assigned(ctx, &dest_info);
                    let dst = self.vreg();
                    pending_outs.push((
                        IrAsmOutput {
                            dst,
                            ty,
                            kind: IrAsmOutKind::Gpr(*reg),
                        },
                        dest_info,
                    ));
                }
                AsmClause::Clobber { names, span } => {
                    let _ = span;
                    for (name, nspan) in names {
                        let m = clobber_mask(*name);
                        if clobber_bits & m != 0 {
                            self.emit(
                                DiagCode::TyMismatch,
                                *nspan,
                                format!("duplicate asm clobber '{}'", name.as_str()),
                            );
                            failed = true;
                        }
                        clobber_bits |= m;
                    }
                }
                AsmClause::Stack { bytes, span } => {
                    if stack.is_some() {
                        self.emit(
                            DiagCode::TyMismatch,
                            *span,
                            "asm stack: specified more than once",
                        );
                        failed = true;
                    }
                    stack = Some(*bytes);
                }
            }
        }

        let gp_out = out_bits & RES_GP;
        let gp_clobber = clobber_bits & RES_GP;
        if gp_out & gp_clobber != 0 {
            self.emit(
                DiagCode::TyMismatch,
                stmt.span,
                "asm clobber overlaps an output register",
            );
            failed = true;
        }

        let validated = match inline_asm::validate_body(&stmt.body, stmt.body_span) {
            Ok(v) => v,
            Err(ds) => {
                self.diagnostics.extend(ds);
                return;
            }
        };
        if let Some(n) = stack {
            if validated.evident_stack > n {
                self.emit(
                    DiagCode::CgUnsupported,
                    stmt.span,
                    format!(
                        "inline asm uses at least {} stack bytes, more than stack:{n}",
                        validated.evident_stack
                    ),
                );
                failed = true;
            }
        }
        let unknown_stack = validated.has_call && stack.is_none();
        if unknown_stack {
            self.warn(
                DiagCode::LnStack,
                stmt.span,
                "opaque CALL/RST in inline asm without stack:N leaves stack usage unknown",
            );
        }
        if failed {
            return;
        }
        let prefix = format!("IA{}_", stmt.id.0);
        let lines = inline_asm::rewrite_lines(&validated.lines, &validated.labels, &prefix);
        let outputs: Vec<IrAsmOutput> = pending_outs.iter().map(|(o, _)| o.clone()).collect();
        self.emit_op(
            ctx,
            IrOp::InlineAsm {
                inputs,
                outputs,
                clobbers: clobbers_of(&stmt.clauses),
                lines,
                stack,
                unknown_stack,
                plain: !stmt.has_header,
                span: stmt.span,
                node: stmt.id,
            },
        );
        for (out, dest) in pending_outs {
            match dest {
                AsmDest::Local(local) => self.emit_op(
                    ctx,
                    IrOp::StoreLocal {
                        local,
                        src: out.dst,
                        span: stmt.span,
                    },
                ),
                AsmDest::Global(global) => self.emit_op(
                    ctx,
                    IrOp::StoreGlobal {
                        global,
                        src: out.dst,
                        span: stmt.span,
                    },
                ),
                AsmDest::Indirect { ptr, ty } => self.emit_op(
                    ctx,
                    IrOp::StoreIndirect {
                        ptr,
                        src: out.dst,
                        ty,
                        span: stmt.span,
                    },
                ),
            }
        }
    }

    fn mark_asm_assigned(&self, ctx: &mut FnCtx, dest: &AsmDest) {
        if let AsmDest::Local(id) = dest {
            ctx.assigned.insert(id.0);
        }
    }

    fn eval_asm_dest(
        &mut self,
        ctx: &mut FnCtx,
        expr: &Expr,
        _read: bool,
    ) -> Option<(AsmDest, CType)> {
        if let ExprKind::Unary {
            op: UnaryOp::Deref,
            expr: inner,
        } = &expr.kind
        {
            let ptr = self.check_expr(ctx, inner, None)?;
            let Some(elem) = ptr.ty.pointee() else {
                self.emit(
                    DiagCode::TyMismatch,
                    expr.span,
                    format!("cannot dereference {}", ptr.ty.as_str()),
                );
                return None;
            };
            return Some((
                AsmDest::Indirect {
                    ptr: ptr.reg,
                    ty: elem,
                },
                elem,
            ));
        }
        if let ExprKind::Index { base, index } = &expr.kind {
            let (addr, immutable) = self.check_index_addr(ctx, base, index, expr.span, false)?;
            if immutable {
                self.emit(
                    DiagCode::TyAssignConst,
                    expr.span,
                    "cannot mutate a str payload",
                );
                return None;
            }
            let elem = addr.ty.pointee().unwrap_or(CType::U8);
            return Some((
                AsmDest::Indirect {
                    ptr: addr.reg,
                    ty: elem,
                },
                elem,
            ));
        }
        if let ExprKind::Field { base, name } = &expr.kind {
            let place = self.check_field_place(ctx, base, name, expr.span)?;
            let ty = place_ty(&place);
            if ty.is_aggregate() {
                self.emit(
                    DiagCode::TyNotLvalue,
                    expr.span,
                    "asm output needs a writable scalar lvalue",
                );
                return None;
            }
            return match place {
                Place::Indirect { ptr, ty, immutable } => {
                    if immutable {
                        self.emit(
                            DiagCode::TyAssignConst,
                            expr.span,
                            "cannot mutate this location",
                        );
                        return None;
                    }
                    Some((AsmDest::Indirect { ptr, ty }, ty))
                }
                Place::Local { id, ty, is_const } => {
                    if is_const {
                        self.emit(
                            DiagCode::TyAssignConst,
                            expr.span,
                            "cannot assign to a const local",
                        );
                        return None;
                    }
                    Some((AsmDest::Local(id), ty))
                }
                Place::Global { id, ty, is_const } => {
                    if is_const {
                        self.emit(
                            DiagCode::TyAssignConst,
                            expr.span,
                            "cannot assign to a const global",
                        );
                        return None;
                    }
                    Some((AsmDest::Global(id), ty))
                }
            };
        }
        if let ExprKind::Qualified { unit, name } = &expr.kind {
            match self.lookup_qualified(unit, name)? {
                QSym::Function { qname, .. } => {
                    self.emit(
                        DiagCode::TyNotLvalue,
                        expr.span,
                        format!("cannot assign to function '{qname}'"),
                    );
                    return None;
                }
                QSym::Value {
                    id, ty, is_const, ..
                } => {
                    if is_const {
                        self.emit(
                            DiagCode::TyAssignConst,
                            expr.span,
                            format!("cannot assign to const '{}::{}'", unit.name, name.name),
                        );
                        return None;
                    }
                    if matches!(ty, CType::Array(_)) || ty == CType::Str {
                        self.emit(
                            DiagCode::TyNotLvalue,
                            expr.span,
                            "asm output needs a writable scalar lvalue",
                        );
                        return None;
                    }
                    return Some((AsmDest::Global(id), ty));
                }
            }
        }
        let ExprKind::Name(name) = &expr.kind else {
            self.emit(
                DiagCode::TyNotLvalue,
                expr.span,
                "asm output needs a writable scalar lvalue",
            );
            return None;
        };
        if let Some(bind) = lookup_local(&ctx.scopes, &name.name).cloned() {
            if bind.is_const {
                self.emit(
                    DiagCode::TyAssignConst,
                    name.span,
                    format!("cannot assign to const '{}'", name.name),
                );
                return None;
            }
            return Some((AsmDest::Local(bind.id), bind.ty));
        }
        if let Some(sym) = self.symbols.get(&name.name) {
            if matches!(sym.kind, SymbolKind::Function { .. }) {
                self.emit(
                    DiagCode::TyNotLvalue,
                    name.span,
                    "cannot assign to a function",
                );
                return None;
            }
            if sym.is_const || matches!(sym.ty, CType::Array(_)) || sym.ty == CType::Str {
                self.emit(
                    DiagCode::TyNotLvalue,
                    name.span,
                    "asm output needs a writable scalar lvalue",
                );
                return None;
            }
            return Some((AsmDest::Global(GlobalId(sym.id)), sym.ty));
        }
        self.emit(
            DiagCode::TyUnresolvedName,
            name.span,
            format!("unresolved name '{}'", name.name),
        );
        None
    }

    fn eval_asm_inout(&mut self, ctx: &mut FnCtx, expr: &Expr) -> Option<(AsmDest, CType, VReg)> {
        let (dest, ty) = self.eval_asm_dest(ctx, expr, true)?;
        let loaded = match &dest {
            AsmDest::Local(local) => {
                if let Some(bind) = lookup_local_id(&ctx.scopes, *local) {
                    if !bind.is_const && !ctx.assigned.contains(&local.0) {
                        self.emit(
                            DiagCode::TyUseBeforeAssign,
                            expr.span,
                            "inout operand is used before assignment",
                        );
                        return None;
                    }
                }
                let dst = self.vreg();
                self.emit_op(
                    ctx,
                    IrOp::LoadLocal {
                        dst,
                        local: *local,
                        span: expr.span,
                    },
                );
                dst
            }
            AsmDest::Global(global) => {
                let dst = self.vreg();
                self.emit_op(
                    ctx,
                    IrOp::LoadGlobal {
                        dst,
                        global: *global,
                        span: expr.span,
                    },
                );
                dst
            }
            AsmDest::Indirect { ptr, ty } => {
                let dst = self.vreg();
                self.emit_op(
                    ctx,
                    IrOp::LoadIndirect {
                        dst,
                        ptr: *ptr,
                        ty: *ty,
                        span: expr.span,
                    },
                );
                dst
            }
        };
        Some((dest, ty, loaded))
    }

    fn check_local(&mut self, ctx: &mut FnCtx, decl: &VarDecl) {
        if decl.array_len.is_some() || self.type_of(&decl.ty).is_aggregate() {
            self.emit(
                DiagCode::TyMismatch,
                decl.name.span,
                "local arrays and structs are not supported; use a global or a pointer",
            );
        }
        let ty = self.type_of(&decl.ty);
        if ty == CType::Void {
            self.emit(
                DiagCode::TyVoidValue,
                decl.ty.span,
                "void is not a value type",
            );
        }
        if let Some(prev) = ctx.scopes.last().unwrap().get(&decl.name.name) {
            self.emit(
                DiagCode::TyDuplicateName,
                decl.name.span,
                format!("duplicate name '{}' in this scope", decl.name.name),
            );
            let _ = prev;
            return;
        }
        let bind = LocalBind {
            id: LocalId(decl.id),
            ty,
            span: decl.name.span,
            is_const: decl.is_const,
            is_param: false,
        };
        ctx.scopes
            .last_mut()
            .unwrap()
            .insert(decl.name.name.clone(), bind);
        ctx.locals.push(TypedLocal {
            id: LocalId(decl.id),
            name: decl.name.name.clone(),
            ty,
            span: decl.span,
        });
        if let Some(init) = &decl.init {
            if let Some(val) = self.check_expr(ctx, init, Some(ty)) {
                self.emit_op(
                    ctx,
                    IrOp::StoreLocal {
                        local: LocalId(decl.id),
                        src: val.reg,
                        span: decl.span,
                    },
                );
                ctx.assigned.insert(decl.id);
            }
        } else if decl.is_const {
            self.emit(
                DiagCode::TyMismatch,
                decl.span,
                "const declaration requires an initializer",
            );
        }
    }

    fn check_if(&mut self, ctx: &mut FnCtx, stmt: &IfStmt) {
        let Some(cond) = self.check_expr(ctx, &stmt.cond, Some(CType::Bool)) else {
            return;
        };
        let then_blk = self.block_id();
        let else_blk = self.block_id();
        let join_blk = self.block_id();
        self.emit_op(
            ctx,
            IrOp::Branch {
                cond: cond.reg,
                true_blk: then_blk,
                false_blk: else_blk,
                span: stmt.cond.span,
            },
        );
        let assigned_before = ctx.assigned.clone();
        self.start_block(ctx, then_blk);
        ctx.reachable = true;
        self.check_stmt(ctx, &stmt.then_branch);
        let then_assigned = ctx.assigned.clone();
        let then_reach = ctx.reachable;
        if then_reach {
            self.emit_op(
                ctx,
                IrOp::Jump {
                    blk: join_blk,
                    span: stmt.span,
                },
            );
        }
        self.start_block(ctx, else_blk);
        ctx.assigned = assigned_before.clone();
        ctx.reachable = true;
        if let Some(else_branch) = &stmt.else_branch {
            self.check_stmt(ctx, else_branch);
        }
        let else_assigned = ctx.assigned.clone();
        let else_reach = ctx.reachable;
        if else_reach {
            self.emit_op(
                ctx,
                IrOp::Jump {
                    blk: join_blk,
                    span: stmt.span,
                },
            );
        }
        ctx.assigned = match (then_reach, else_reach) {
            (true, true) => then_assigned
                .intersection(&else_assigned)
                .copied()
                .collect(),
            (true, false) => then_assigned,
            (false, true) => else_assigned,
            (false, false) => assigned_before,
        };
        ctx.reachable = then_reach || else_reach;
        if ctx.reachable {
            self.start_block(ctx, join_blk);
        } else {
            self.start_block(ctx, join_blk);
            ctx.reachable = false;
        }
    }

    fn check_while(&mut self, ctx: &mut FnCtx, stmt: &WhileStmt) {
        let cond_blk = self.block_id();
        let body_blk = self.block_id();
        let after_blk = self.block_id();
        self.emit_op(
            ctx,
            IrOp::Jump {
                blk: cond_blk,
                span: stmt.span,
            },
        );
        let assigned_entry = ctx.assigned.clone();
        self.start_block(ctx, cond_blk);
        let Some(cond) = self.check_expr(ctx, &stmt.cond, Some(CType::Bool)) else {
            return;
        };
        self.emit_op(
            ctx,
            IrOp::Branch {
                cond: cond.reg,
                true_blk: body_blk,
                false_blk: after_blk,
                span: stmt.cond.span,
            },
        );
        ctx.loop_stack.push(LoopCtx {
            break_blk: after_blk,
            continue_blk: cond_blk,
            assigned_at_entry: assigned_entry.clone(),
        });
        self.start_block(ctx, body_blk);
        ctx.reachable = true;
        ctx.assigned = assigned_entry.clone();
        self.check_stmt(ctx, &stmt.body);
        if ctx.reachable {
            self.emit_op(
                ctx,
                IrOp::Jump {
                    blk: cond_blk,
                    span: stmt.span,
                },
            );
        }
        ctx.loop_stack.pop();
        // Zero-trip: assignments only in the body do not hold afterward.
        ctx.assigned = assigned_entry;
        ctx.reachable = true;
        self.start_block(ctx, after_blk);
    }

    fn check_for(&mut self, ctx: &mut FnCtx, stmt: &ForStmt) {
        ctx.scopes.push(HashMap::new());
        match &stmt.init {
            Some(ForInit::Decl(decl)) => self.check_local(ctx, decl),
            Some(ForInit::Expr(expr)) => {
                let _ = self.check_expr(ctx, expr, None);
            }
            None => {}
        }
        let cond_blk = self.block_id();
        let body_blk = self.block_id();
        let update_blk = self.block_id();
        let after_blk = self.block_id();
        self.emit_op(
            ctx,
            IrOp::Jump {
                blk: cond_blk,
                span: stmt.span,
            },
        );
        let assigned_entry = ctx.assigned.clone();
        self.start_block(ctx, cond_blk);
        let cond = if let Some(cond) = &stmt.cond {
            self.check_expr(ctx, cond, Some(CType::Bool))
        } else {
            Some(self.const_val(ctx, CType::Bool, 1, stmt.span))
        };
        let Some(cond) = cond else {
            ctx.scopes.pop();
            return;
        };
        self.emit_op(
            ctx,
            IrOp::Branch {
                cond: cond.reg,
                true_blk: body_blk,
                false_blk: after_blk,
                span: stmt.cond.as_ref().map(|c| c.span).unwrap_or(stmt.span),
            },
        );
        ctx.loop_stack.push(LoopCtx {
            break_blk: after_blk,
            continue_blk: update_blk,
            assigned_at_entry: assigned_entry.clone(),
        });
        self.start_block(ctx, body_blk);
        ctx.reachable = true;
        ctx.assigned = assigned_entry.clone();
        self.check_stmt(ctx, &stmt.body);
        if ctx.reachable {
            self.emit_op(
                ctx,
                IrOp::Jump {
                    blk: update_blk,
                    span: stmt.span,
                },
            );
        }
        self.start_block(ctx, update_blk);
        ctx.reachable = true;
        if let Some(update) = &stmt.update {
            let _ = self.check_expr(ctx, update, None);
        }
        self.emit_op(
            ctx,
            IrOp::Jump {
                blk: cond_blk,
                span: stmt.span,
            },
        );
        ctx.loop_stack.pop();
        ctx.assigned = assigned_entry;
        ctx.reachable = true;
        self.start_block(ctx, after_blk);
        ctx.scopes.pop();
    }

    fn check_do_while(&mut self, ctx: &mut FnCtx, stmt: &DoWhileStmt) {
        let body_blk = self.block_id();
        let cond_blk = self.block_id();
        let after_blk = self.block_id();
        self.emit_op(
            ctx,
            IrOp::Jump {
                blk: body_blk,
                span: stmt.span,
            },
        );
        let assigned_entry = ctx.assigned.clone();
        ctx.loop_stack.push(LoopCtx {
            break_blk: after_blk,
            continue_blk: cond_blk,
            assigned_at_entry: assigned_entry.clone(),
        });
        self.start_block(ctx, body_blk);
        ctx.reachable = true;
        ctx.assigned = assigned_entry.clone();
        self.check_stmt(ctx, &stmt.body);
        if ctx.reachable {
            self.emit_op(
                ctx,
                IrOp::Jump {
                    blk: cond_blk,
                    span: stmt.span,
                },
            );
        }
        let assigned_after_body = ctx.assigned.clone();
        ctx.loop_stack.pop();
        self.start_block(ctx, cond_blk);
        ctx.reachable = true;
        let Some(cond) = self.check_expr(ctx, &stmt.cond, Some(CType::Bool)) else {
            return;
        };
        self.emit_op(
            ctx,
            IrOp::Branch {
                cond: cond.reg,
                true_blk: body_blk,
                false_blk: after_blk,
                span: stmt.cond.span,
            },
        );
        ctx.assigned = assigned_after_body;
        ctx.reachable = true;
        self.start_block(ctx, after_blk);
    }

    fn check_return(&mut self, ctx: &mut FnCtx, stmt: &ReturnStmt) {
        match (ctx.ret, &stmt.value) {
            (CType::Void, None) => {
                self.emit_op(
                    ctx,
                    IrOp::Return {
                        value: None,
                        span: stmt.span,
                    },
                );
            }
            (CType::Void, Some(expr)) => {
                self.emit(
                    DiagCode::TyMismatch,
                    expr.span,
                    "void function cannot return a value",
                );
                let _ = self.check_expr(ctx, expr, None);
            }
            (ty, Some(expr)) => {
                if matches!(
                    &expr.kind,
                    ExprKind::Unary {
                        op: UnaryOp::AddrOf,
                        expr: inner
                    } if matches!(&inner.kind, ExprKind::Name(name) if lookup_local(&ctx.scopes, &name.name).is_some())
                ) {
                    self.warn(
                        DiagCode::TyReturnLocalAddr,
                        expr.span,
                        "returning the address of a local; the pointer is only valid for this call",
                    );
                }
                if let Some(val) = self.check_expr(ctx, expr, Some(ty)) {
                    self.emit_op(
                        ctx,
                        IrOp::Return {
                            value: Some(val.reg),
                            span: stmt.span,
                        },
                    );
                }
            }
            (ty, None) => {
                self.emit(
                    DiagCode::TyMissingReturn,
                    stmt.span,
                    format!("function returns {}, so return needs a value", ty.as_str()),
                );
            }
        }
        ctx.reachable = false;
    }

    fn check_expr(
        &mut self,
        ctx: &mut FnCtx,
        expr: &Expr,
        expected: Option<CType>,
    ) -> Option<Value> {
        let value = match &expr.kind {
            ExprKind::Error => return None,
            ExprKind::Name(name) => self.check_name(ctx, name)?,
            ExprKind::Qualified { unit, name } => self.check_qualified(ctx, unit, name)?,
            ExprKind::Int(lit) => self.check_int(ctx, expr.span, lit.value, false, expected)?,
            ExprKind::Char(b) => {
                let ty = expected.filter(|t| t.is_integer()).unwrap_or(CType::U8);
                self.const_val(ctx, ty, u16::from(*b), expr.span)
            }
            ExprKind::Bool(b) => {
                if let Some(ty) = expected {
                    if ty != CType::Bool {
                        self.emit(
                            DiagCode::TyMismatch,
                            expr.span,
                            format!("bool literal used where {} was expected", ty.as_str()),
                        );
                        return None;
                    }
                }
                self.const_val(ctx, CType::Bool, u16::from(*b), expr.span)
            }
            ExprKind::String(bytes) => {
                let want = expected.unwrap_or(CType::Str);
                if want != CType::Str {
                    self.emit(
                        DiagCode::TyMismatch,
                        expr.span,
                        format!("string literal used where {} was expected", want.as_str()),
                    );
                    return None;
                }
                let id = self.intern_string(bytes, expr.span, ctx.owner_span)?;
                let dst = self.vreg();
                self.emit_op(
                    ctx,
                    IrOp::AddrGlobal {
                        dst,
                        global: id,
                        span: expr.span,
                    },
                );
                Value {
                    ty: CType::Str,
                    reg: dst,
                    bits: None,
                }
            }
            ExprKind::Unary { op, expr: inner } => {
                self.check_unary(ctx, *op, inner, expr.span, expected)?
            }
            ExprKind::Binary { op, lhs, rhs } => {
                self.check_binary(ctx, *op, lhs, rhs, expr.span, expected)?
            }
            ExprKind::Assign { lhs, rhs } => self.check_assign(ctx, lhs, rhs, expr.span)?,
            ExprKind::CompoundAssign { op, lhs, rhs } => {
                self.check_compound(ctx, *op, lhs, rhs, expr.span)?
            }
            ExprKind::PrefixInc { op, expr: inner } => {
                self.check_inc(ctx, *op, inner, true, expr.span)?
            }
            ExprKind::PostfixInc { op, expr: inner } => {
                self.check_inc(ctx, *op, inner, false, expr.span)?
            }
            ExprKind::InitList { .. } => {
                self.emit(
                    DiagCode::TyMismatch,
                    expr.span,
                    "brace initializers are only valid for static arrays and structs",
                );
                return None;
            }
            ExprKind::Call { callee, args } => self.check_call(ctx, callee, args, expr.span)?,
            ExprKind::Index { base, index } => self.check_index(ctx, base, index, expr.span)?,
            ExprKind::Field { base, name } => self.check_field(ctx, base, name, expr.span)?,
            ExprKind::Cast { ty, expr: inner } => self.check_cast(ctx, ty, inner, expr.span)?,
            ExprKind::Sizeof { ty } => {
                let cty = self.type_of(ty);
                let width = if matches!(cty, CType::Struct(_)) {
                    cty.storage_size(&self.structs)
                } else {
                    cty.byte_width().map(u16::from)
                };
                match width {
                    Some(w) => {
                        let out_ty = expected.filter(|t| t.is_integer()).unwrap_or(CType::U16);
                        if !out_ty.is_integer() {
                            self.emit(DiagCode::TyMismatch, expr.span, "sizeof result is u16");
                            return None;
                        }
                        let val = self.const_val(ctx, CType::U16, w, expr.span);
                        if out_ty != CType::U16 {
                            self.cast_val(ctx, val, out_ty, expr.span)
                        } else {
                            val
                        }
                    }
                    None if cty == CType::Void => {
                        self.emit(
                            DiagCode::TyVoidValue,
                            ty.span,
                            "sizeof(void) is not defined",
                        );
                        return None;
                    }
                    None => {
                        self.emit(
                            DiagCode::TyMismatch,
                            ty.span,
                            format!("sizeof({}) is not defined in this slice", cty.as_str()),
                        );
                        return None;
                    }
                }
            }
        };
        if let Some(want) = expected {
            if value.ty != want {
                self.emit(
                    DiagCode::TyMismatch,
                    expr.span,
                    format!("expected {}, found {}", want.as_str(), value.ty.as_str()),
                );
                return None;
            }
        }
        Some(value)
    }

    fn check_int(
        &mut self,
        ctx: &mut FnCtx,
        span: SourceSpan,
        value: Option<u32>,
        negative: bool,
        expected: Option<CType>,
    ) -> Option<Value> {
        let Some(raw) = value else {
            return None;
        };
        let signed_val = if negative {
            if raw > i32::MAX as u32 {
                self.emit(
                    DiagCode::TyLiteralRange,
                    span,
                    "integer literal is out of range",
                );
                return None;
            }
            -(raw as i32)
        } else if raw > i32::MAX as u32 {
            self.emit(
                DiagCode::TyLiteralRange,
                span,
                "integer literal is out of range",
            );
            return None;
        } else {
            raw as i32
        };
        let ty = match expected {
            Some(CType::Bool) => {
                self.emit(
                    DiagCode::TyMismatch,
                    span,
                    "integer literal does not become bool without an explicit conversion",
                );
                return None;
            }
            Some(ty) if ty.is_integer() => ty,
            Some(CType::Void) => {
                self.emit(DiagCode::TyVoidValue, span, "void is not a value type");
                return None;
            }
            Some(_) => {
                self.emit(
                    DiagCode::TyMismatch,
                    span,
                    "integer literal used in a non-integer context",
                );
                return None;
            }
            None => {
                if negative {
                    CType::I16
                } else {
                    CType::U16
                }
            }
        };
        if !ty.contains_int(signed_val) {
            self.emit(
                DiagCode::TyLiteralRange,
                span,
                format!("literal {signed_val} does not fit in {}", ty.as_str()),
            );
            return None;
        }
        Some(self.const_val(ctx, ty, ty.wrap_bits(signed_val), span))
    }

    fn check_name(&mut self, ctx: &mut FnCtx, name: &Ident) -> Option<Value> {
        if let Some(bind) = lookup_local(&ctx.scopes, &name.name).cloned() {
            if !bind.is_const && !ctx.assigned.contains(&bind.id.0) {
                self.emit(
                    DiagCode::TyUseBeforeAssign,
                    name.span,
                    format!("'{}' is used before assignment", name.name),
                );
                return None;
            }
            let dst = self.vreg();
            self.emit_op(
                ctx,
                IrOp::LoadLocal {
                    dst,
                    local: bind.id,
                    span: name.span,
                },
            );
            return Some(Value {
                ty: bind.ty,
                reg: dst,
                bits: None,
            });
        }
        let global = self.symbols.get(&name.name).map(|sym| {
            (
                matches!(sym.kind, SymbolKind::Function { .. }),
                sym.ty,
                sym.id,
                sym.is_const,
                sym.const_bits,
            )
        });
        if let Some((is_fn, ty, id, is_const, const_bits)) = global {
            if is_fn {
                self.emit(
                    DiagCode::TyMismatch,
                    name.span,
                    format!("'{}' is a function", name.name),
                );
                return None;
            }
            if matches!(ty, CType::Array(_)) {
                self.emit(
                    DiagCode::TyMismatch,
                    name.span,
                    format!(
                        "array '{}' does not decay to a pointer; use &{}[0]",
                        name.name, name.name
                    ),
                );
                return None;
            }
            if matches!(ty, CType::Struct(_)) {
                self.emit(
                    DiagCode::TyMismatch,
                    name.span,
                    format!(
                        "struct '{}' cannot be used as a value; select a field or take a field address",
                        name.name
                    ),
                );
                return None;
            }
            if is_const {
                let Some(bits) = const_bits else {
                    self.emit(
                        DiagCode::TyMismatch,
                        name.span,
                        format!("const '{}' has no compile-time value", name.name),
                    );
                    return None;
                };
                return Some(self.const_val(ctx, ty, bits, name.span));
            }
            let dst = self.vreg();
            if ty == CType::Str {
                self.emit_op(
                    ctx,
                    IrOp::AddrGlobal {
                        dst,
                        global: GlobalId(id),
                        span: name.span,
                    },
                );
            } else {
                self.emit_op(
                    ctx,
                    IrOp::LoadGlobal {
                        dst,
                        global: GlobalId(id),
                        span: name.span,
                    },
                );
            }
            return Some(Value {
                ty,
                reg: dst,
                bits: None,
            });
        }
        self.emit(
            DiagCode::TyUnresolvedName,
            name.span,
            format!("unresolved name '{}'", name.name),
        );
        None
    }

    fn check_qualified(&mut self, ctx: &mut FnCtx, unit: &Ident, name: &Ident) -> Option<Value> {
        match self.lookup_qualified(unit, name)? {
            QSym::Function { qname, .. } => {
                self.emit(
                    DiagCode::TyMismatch,
                    name.span,
                    format!("'{qname}' is a function"),
                );
                None
            }
            QSym::Value {
                id,
                ty,
                is_const,
                bits,
            } => {
                if matches!(ty, CType::Array(_)) {
                    self.emit(
                        DiagCode::TyMismatch,
                        name.span,
                        format!(
                            "array '{}::{}' does not decay to a pointer; use &{}::{}[0]",
                            unit.name, name.name, unit.name, name.name
                        ),
                    );
                    return None;
                }
                if is_const {
                    let Some(bits) = bits else {
                        self.emit(
                            DiagCode::TyMismatch,
                            name.span,
                            format!(
                                "const '{}::{}' has no compile-time value",
                                unit.name, name.name
                            ),
                        );
                        return None;
                    };
                    return Some(self.const_val(ctx, ty, bits, name.span));
                }
                let dst = self.vreg();
                if ty == CType::Str {
                    self.emit_op(
                        ctx,
                        IrOp::AddrGlobal {
                            dst,
                            global: id,
                            span: name.span,
                        },
                    );
                } else {
                    self.emit_op(
                        ctx,
                        IrOp::LoadGlobal {
                            dst,
                            global: id,
                            span: name.span,
                        },
                    );
                }
                Some(Value {
                    ty,
                    reg: dst,
                    bits: None,
                })
            }
        }
    }

    fn lookup_qualified(&mut self, unit: &Ident, name: &Ident) -> Option<QSym> {
        if unit.name == "project" {
            return self.lookup_project(name);
        }
        if !self.imports.contains(&unit.name) {
            self.emit(
                DiagCode::TyUnresolvedName,
                unit.span,
                format!("'{}' is not imported", unit.name),
            );
            return None;
        }
        let Some(exports) = self.exports else {
            self.emit(
                DiagCode::TyUnresolvedName,
                name.span,
                format!("unresolved name '{}::{}'", unit.name, name.name),
            );
            return None;
        };
        let Some(unit_ex) = exports.get(&unit.name) else {
            self.emit(
                DiagCode::TyUnresolvedName,
                unit.span,
                format!("unknown unit '{}'", unit.name),
            );
            return None;
        };
        let Some(sym) = unit_ex.symbols.get(&name.name) else {
            self.emit(
                DiagCode::TyUnresolvedName,
                name.span,
                format!("unresolved name '{}::{}'", unit.name, name.name),
            );
            return None;
        };
        if !sym.is_pub() {
            self.emit(
                DiagCode::TyUnresolvedName,
                name.span,
                format!("'{}::{}' is private", unit.name, name.name),
            );
            return None;
        }
        Some(match sym {
            ExportedSymbol::Function {
                id,
                params,
                conv,
                ret,
                ..
            } => QSym::Function {
                id: *id,
                params: params.clone(),
                conv: *conv,
                ret: *ret,
                qname: format!("{}::{}", unit.name, name.name),
            },
            ExportedSymbol::Value {
                id,
                ty,
                is_const,
                bits,
                ..
            } => QSym::Value {
                id: *id,
                ty: *ty,
                is_const: *is_const,
                bits: *bits,
            },
        })
    }

    fn lookup_project(&mut self, name: &Ident) -> Option<QSym> {
        let Some(stack) = self.stack else {
            self.emit(
                DiagCode::TyUnresolvedName,
                name.span,
                "project stack is not configured",
            );
            return None;
        };
        let base = u32::from(stack.base);
        let size = u32::from(stack.size);
        let end = base + size;
        let bits = match name.name.as_str() {
            "stack_base" => stack.base,
            "stack_size" => stack.size,
            "stack_top" => (end % 65536) as u16,
            "stack_end" => {
                if end == 65536 {
                    self.emit(
                        DiagCode::TyLiteralRange,
                        name.span,
                        "project::stack_end is 65536 and is not a u16 value",
                    );
                    return None;
                }
                end as u16
            }
            _ => {
                self.emit(
                    DiagCode::TyUnresolvedName,
                    name.span,
                    format!("unknown project builtin '{}'", name.name),
                );
                return None;
            }
        };
        Some(QSym::Value {
            id: GlobalId(NodeId(0)),
            ty: CType::U16,
            is_const: true,
            bits: Some(bits),
        })
    }

    fn check_unary(
        &mut self,
        ctx: &mut FnCtx,
        op: UnaryOp,
        inner: &Expr,
        span: SourceSpan,
        expected: Option<CType>,
    ) -> Option<Value> {
        if op == UnaryOp::Minus {
            if let ExprKind::Int(lit) = &inner.kind {
                return self.check_int(ctx, span, lit.value, true, expected);
            }
        }
        match op {
            UnaryOp::Not => {
                let inner = self.check_expr(ctx, inner, Some(CType::Bool))?;
                let dst = self.vreg();
                let bits = inner.bits.map(|b| u16::from(b == 0));
                self.emit_op(
                    ctx,
                    IrOp::Unary {
                        dst,
                        ty: CType::Bool,
                        op: IrUnary::Not,
                        src: inner.reg,
                        span,
                    },
                );
                Some(Value {
                    ty: CType::Bool,
                    reg: dst,
                    bits,
                })
            }
            UnaryOp::Plus | UnaryOp::Minus | UnaryOp::BitNot => {
                let inner = self.check_expr(ctx, inner, expected.filter(|t| t.is_integer()))?;
                if !inner.ty.is_integer() {
                    self.emit(
                        DiagCode::TyMismatch,
                        span,
                        format!(
                            "unary operator requires an integer, found {}",
                            inner.ty.as_str()
                        ),
                    );
                    return None;
                }
                let dst = self.vreg();
                let ir_op = match op {
                    UnaryOp::Plus => IrUnary::Plus,
                    UnaryOp::Minus => IrUnary::Neg,
                    UnaryOp::BitNot => IrUnary::BitNot,
                    UnaryOp::Not | UnaryOp::Deref | UnaryOp::AddrOf => unreachable!(),
                };
                let bits = inner.bits.map(|b| {
                    let v = inner.ty.interpret_bits(b);
                    let out = match op {
                        UnaryOp::Plus => v,
                        UnaryOp::Minus => v.wrapping_neg(),
                        UnaryOp::BitNot => !v,
                        UnaryOp::Not | UnaryOp::Deref | UnaryOp::AddrOf => unreachable!(),
                    };
                    inner.ty.wrap_bits(out)
                });
                self.emit_op(
                    ctx,
                    IrOp::Unary {
                        dst,
                        ty: inner.ty,
                        op: ir_op,
                        src: inner.reg,
                        span,
                    },
                );
                Some(Value {
                    ty: inner.ty,
                    reg: dst,
                    bits,
                })
            }
            UnaryOp::Deref => {
                let inner = self.check_expr(ctx, inner, None)?;
                let Some(elem) = inner.ty.pointee() else {
                    self.emit(
                        DiagCode::TyMismatch,
                        span,
                        format!("cannot dereference {}", inner.ty.as_str()),
                    );
                    return None;
                };
                if elem.is_aggregate() {
                    self.emit(
                        DiagCode::TyMismatch,
                        span,
                        "cannot load an aggregate as a value; select a field",
                    );
                    return None;
                }
                let dst = self.vreg();
                self.emit_op(
                    ctx,
                    IrOp::LoadIndirect {
                        dst,
                        ptr: inner.reg,
                        ty: elem,
                        span,
                    },
                );
                Some(Value {
                    ty: elem,
                    reg: dst,
                    bits: None,
                })
            }
            UnaryOp::AddrOf => self.check_addr_of(ctx, inner, span),
        }
    }

    fn check_index(
        &mut self,
        ctx: &mut FnCtx,
        base: &Expr,
        index: &Expr,
        span: SourceSpan,
    ) -> Option<Value> {
        let (addr, _) = self.check_index_addr(ctx, base, index, span, false)?;
        let elem = addr.ty.pointee().unwrap_or(CType::U8);
        if elem.is_aggregate() {
            self.emit(
                DiagCode::TyMismatch,
                span,
                "cannot load an aggregate as a value; select a field",
            );
            return None;
        }
        let dst = self.vreg();
        self.emit_op(
            ctx,
            IrOp::LoadIndirect {
                dst,
                ptr: addr.reg,
                ty: addr.ty.pointee().unwrap_or(CType::U8),
                span,
            },
        );
        Some(Value {
            ty: addr.ty.pointee().unwrap_or(CType::U8),
            reg: dst,
            bits: None,
        })
    }

    fn check_index_addr(
        &mut self,
        ctx: &mut FnCtx,
        base: &Expr,
        index: &Expr,
        span: SourceSpan,
        for_addr_of: bool,
    ) -> Option<(Value, bool)> {
        let idx = self.check_expr(ctx, index, None)?;
        if !idx.ty.is_integer() {
            self.emit(
                DiagCode::TyMismatch,
                index.span,
                "array and pointer indexes must be integers",
            );
            return None;
        }
        let const_idx = const_index_i32(index);
        let mut known_len: Option<u32> = None;
        let mut immutable = false;
        if let ExprKind::String(bytes) = &base.kind {
            known_len = Some(bytes.len() as u32);
        }
        let base_val = if let ExprKind::Name(name) = &base.kind {
            let array = self.symbols.get(&name.name).and_then(|sym| {
                if let CType::Array(arr) = sym.ty {
                    Some((arr, sym.id, name.span))
                } else {
                    None
                }
            });
            if let Some((arr, id, name_span)) = array {
                known_len = Some(u32::from(arr.len));
                let addr = self.vreg();
                self.emit_op(
                    ctx,
                    IrOp::AddrGlobal {
                        dst: addr,
                        global: GlobalId(id),
                        span: name_span,
                    },
                );
                Some(Value {
                    ty: PtrType::of(arr.elem.to_ctype())
                        .map(CType::Ptr)
                        .unwrap_or(CType::Void),
                    reg: addr,
                    bits: None,
                })
            } else {
                None
            }
        } else if let ExprKind::Qualified { unit, name } = &base.kind {
            match self.lookup_qualified(unit, name) {
                None => return None,
                Some(QSym::Value {
                    id,
                    ty: CType::Array(arr),
                    ..
                }) => {
                    known_len = Some(u32::from(arr.len));
                    let addr = self.vreg();
                    self.emit_op(
                        ctx,
                        IrOp::AddrGlobal {
                            dst: addr,
                            global: id,
                            span: name.span,
                        },
                    );
                    Some(Value {
                        ty: PtrType::of(arr.elem.to_ctype())
                            .map(CType::Ptr)
                            .unwrap_or(CType::Void),
                        reg: addr,
                        bits: None,
                    })
                }
                Some(_) => None,
            }
        } else if let ExprKind::Field { base: fbase, name } = &base.kind {
            match self.check_field_place(ctx, fbase, name, base.span) {
                Some(Place::Indirect {
                    ptr,
                    ty: CType::Array(arr),
                    immutable: imm,
                }) => {
                    known_len = Some(u32::from(arr.len));
                    immutable = imm;
                    Some(Value {
                        ty: PtrType::of(arr.elem.to_ctype())
                            .map(CType::Ptr)
                            .unwrap_or(CType::Void),
                        reg: ptr,
                        bits: None,
                    })
                }
                Some(_) => {
                    self.emit(DiagCode::TyMismatch, span, "cannot index a non-array field");
                    return None;
                }
                None => return None,
            }
        } else {
            None
        };
        let base_val = match base_val {
            Some(v) => v,
            None => self.check_expr(ctx, base, None)?,
        };
        let elem = if let Some(elem) = base_val.ty.pointee() {
            elem
        } else if base_val.ty == CType::Str {
            immutable = true;
            if let ExprKind::Name(name) = &base.kind {
                if let Some(sym) = self.symbols.get(&name.name) {
                    if let Some(&len) = self.str_lens.get(&sym.id) {
                        known_len = Some(u32::from(len));
                    }
                }
            }
            CType::U8
        } else {
            self.emit(
                DiagCode::TyMismatch,
                span,
                format!("cannot index {}", base_val.ty.as_str()),
            );
            return None;
        };
        if let Some(len) = known_len {
            if let Some(v) = const_idx {
                if v < 0 || v as u32 >= len {
                    self.emit(
                        DiagCode::TyLiteralRange,
                        index.span,
                        "constant index is outside the array or string",
                    );
                    return None;
                }
            }
        }
        if for_addr_of && immutable {
            self.emit(
                DiagCode::TyNotLvalue,
                span,
                "cannot take the address of a string payload",
            );
            return None;
        }
        let ptr = if base_val.ty == CType::Str {
            let one = self.const_val(ctx, CType::U16, 1, span);
            let with_prefix = self.vreg();
            self.emit_op(
                ctx,
                IrOp::Binary {
                    dst: with_prefix,
                    ty: CType::U16,
                    op: IrBinary::Add,
                    lhs: base_val.reg,
                    rhs: one.reg,
                    span,
                },
            );
            with_prefix
        } else {
            base_val.reg
        };
        let scale = elem.storage_size(&self.structs).unwrap_or(1);
        let idx16 = if idx.ty.byte_width() == Some(2) {
            idx
        } else {
            self.cast_val(ctx, idx, CType::U16, index.span)
        };
        let off = self.scale_index(ctx, idx16, scale, span);
        let addr = self.vreg();
        self.emit_op(
            ctx,
            IrOp::Binary {
                dst: addr,
                ty: CType::U16,
                op: IrBinary::Add,
                lhs: ptr,
                rhs: off,
                span,
            },
        );
        Some((
            Value {
                ty: PtrType::of(elem).map(CType::Ptr).unwrap_or(CType::Void),
                reg: addr,
                bits: None,
            },
            immutable,
        ))
    }

    fn check_addr_of(&mut self, ctx: &mut FnCtx, inner: &Expr, span: SourceSpan) -> Option<Value> {
        match &inner.kind {
            ExprKind::Unary {
                op: UnaryOp::Deref,
                expr,
            } => self.check_expr(ctx, expr, None),
            ExprKind::Index { base, index } => {
                let (addr, _) = self.check_index_addr(ctx, base, index, span, true)?;
                Some(addr)
            }
            ExprKind::Name(name) => {
                if let Some(bind) = lookup_local(&ctx.scopes, &name.name).cloned() {
                    if bind.is_const {
                        self.emit(
                            DiagCode::TyNotLvalue,
                            span,
                            "cannot take the address of a constant",
                        );
                        return None;
                    }
                    let Some(ptr_ty) = PtrType::of(bind.ty).map(CType::Ptr) else {
                        self.emit(
                            DiagCode::TyMismatch,
                            span,
                            format!("cannot take the address of {}", bind.ty.as_str()),
                        );
                        return None;
                    };
                    let dst = self.vreg();
                    self.emit_op(
                        ctx,
                        IrOp::AddrLocal {
                            dst,
                            local: bind.id,
                            span,
                        },
                    );
                    return Some(Value {
                        ty: ptr_ty,
                        reg: dst,
                        bits: None,
                    });
                }
                let global = self.symbols.get(&name.name).map(|sym| {
                    (
                        matches!(sym.kind, SymbolKind::Function { .. }),
                        sym.ty,
                        sym.id,
                        sym.is_const,
                    )
                });
                if let Some((is_fn, ty, id, is_const)) = global {
                    if is_fn {
                        self.emit(
                            DiagCode::TyNotLvalue,
                            span,
                            "cannot take the address of a function",
                        );
                        return None;
                    }
                    if is_const {
                        self.emit(
                            DiagCode::TyNotLvalue,
                            span,
                            "cannot take the address of a constant",
                        );
                        return None;
                    }
                    if matches!(ty, CType::Array(_)) {
                        self.emit(
                            DiagCode::TyMismatch,
                            span,
                            format!(
                                "cannot take the address of array '{}'; use &{}[0]",
                                name.name, name.name
                            ),
                        );
                        return None;
                    }
                    let Some(ptr_ty) = PtrType::of(ty).map(CType::Ptr) else {
                        self.emit(
                            DiagCode::TyMismatch,
                            span,
                            format!("cannot take the address of {}", ty.as_str()),
                        );
                        return None;
                    };
                    let dst = self.vreg();
                    self.emit_op(
                        ctx,
                        IrOp::AddrGlobal {
                            dst,
                            global: GlobalId(id),
                            span,
                        },
                    );
                    return Some(Value {
                        ty: ptr_ty,
                        reg: dst,
                        bits: None,
                    });
                }
                self.emit(
                    DiagCode::TyUnresolvedName,
                    name.span,
                    format!("unresolved name '{}'", name.name),
                );
                None
            }
            ExprKind::Qualified { unit, name } => match self.lookup_qualified(unit, name)? {
                QSym::Function { qname, .. } => {
                    self.emit(
                        DiagCode::TyNotLvalue,
                        span,
                        format!("cannot take the address of function '{qname}'"),
                    );
                    None
                }
                QSym::Value {
                    id, ty, is_const, ..
                } => {
                    if is_const {
                        self.emit(
                            DiagCode::TyNotLvalue,
                            span,
                            "cannot take the address of a constant",
                        );
                        return None;
                    }
                    if matches!(ty, CType::Array(_)) {
                        self.emit(
                            DiagCode::TyMismatch,
                            span,
                            format!(
                                "cannot take the address of array '{}::{}'; use &{}::{}[0]",
                                unit.name, name.name, unit.name, name.name
                            ),
                        );
                        return None;
                    }
                    let Some(ptr_ty) = PtrType::of(ty).map(CType::Ptr) else {
                        self.emit(
                            DiagCode::TyMismatch,
                            span,
                            format!("cannot take the address of {}", ty.as_str()),
                        );
                        return None;
                    };
                    let dst = self.vreg();
                    self.emit_op(
                        ctx,
                        IrOp::AddrGlobal {
                            dst,
                            global: id,
                            span,
                        },
                    );
                    Some(Value {
                        ty: ptr_ty,
                        reg: dst,
                        bits: None,
                    })
                }
            },
            ExprKind::Field { base, name } => {
                let place = self.check_field_place(ctx, base, name, span)?;
                if place_immutable(&place) {
                    self.emit(
                        DiagCode::TyNotLvalue,
                        span,
                        "cannot take the address of this field",
                    );
                    return None;
                }
                let ty = place_ty(&place);
                if matches!(ty, CType::Array(_)) {
                    self.emit(
                        DiagCode::TyMismatch,
                        span,
                        "cannot take the address of an array field; index an element",
                    );
                    return None;
                }
                let Some(ptr_ty) = PtrType::of(ty).map(CType::Ptr) else {
                    self.emit(
                        DiagCode::TyMismatch,
                        span,
                        format!("cannot take the address of {}", ty.as_str()),
                    );
                    return None;
                };
                match place {
                    Place::Indirect { ptr, .. } => Some(Value {
                        ty: ptr_ty,
                        reg: ptr,
                        bits: None,
                    }),
                    Place::Local { id, .. } => {
                        let dst = self.vreg();
                        self.emit_op(
                            ctx,
                            IrOp::AddrLocal {
                                dst,
                                local: id,
                                span,
                            },
                        );
                        Some(Value {
                            ty: ptr_ty,
                            reg: dst,
                            bits: None,
                        })
                    }
                    Place::Global { id, .. } => {
                        let dst = self.vreg();
                        self.emit_op(
                            ctx,
                            IrOp::AddrGlobal {
                                dst,
                                global: id,
                                span,
                            },
                        );
                        Some(Value {
                            ty: ptr_ty,
                            reg: dst,
                            bits: None,
                        })
                    }
                }
            }
            ExprKind::String(_) => {
                self.emit(
                    DiagCode::TyNotLvalue,
                    span,
                    "cannot take the address of a string payload",
                );
                None
            }
            _ => {
                self.emit(
                    DiagCode::TyNotLvalue,
                    span,
                    "address-of requires a scalar or array-element lvalue",
                );
                None
            }
        }
    }

    fn check_field(
        &mut self,
        ctx: &mut FnCtx,
        base: &Expr,
        name: &Ident,
        span: SourceSpan,
    ) -> Option<Value> {
        if name.name == "len" {
            let base_val = self.check_expr(ctx, base, None)?;
            if base_val.ty == CType::Str {
                let dst = self.vreg();
                self.emit_op(
                    ctx,
                    IrOp::LoadIndirect {
                        dst,
                        ptr: base_val.reg,
                        ty: CType::U8,
                        span,
                    },
                );
                return Some(Value {
                    ty: CType::U8,
                    reg: dst,
                    bits: None,
                });
            }
        }
        let place = self.check_field_place(ctx, base, name, span)?;
        if place_ty(&place).is_aggregate() {
            self.emit(
                DiagCode::TyMismatch,
                span,
                "cannot load an aggregate field as a value; index or select a nested field",
            );
            return None;
        }
        self.load_place(ctx, &place, span)
    }

    fn check_field_place(
        &mut self,
        ctx: &mut FnCtx,
        base: &Expr,
        name: &Ident,
        span: SourceSpan,
    ) -> Option<Place> {
        let (addr, sid, immutable) = self.struct_base_addr(ctx, base, span)?;
        let Some(def) = self.structs.iter().find(|s| s.id == sid).cloned() else {
            self.emit(DiagCode::TyMismatch, span, "struct type is incomplete");
            return None;
        };
        let Some(field) = def.fields.iter().find(|f| f.name == name.name).cloned() else {
            self.emit(
                DiagCode::TyMismatch,
                name.span,
                format!("unknown field '{}'", name.name),
            );
            return None;
        };
        let ptr = if field.offset == 0 {
            addr
        } else {
            let off = self.const_val(ctx, CType::U16, field.offset, span);
            let dst = self.vreg();
            self.emit_op(
                ctx,
                IrOp::Binary {
                    dst,
                    ty: CType::U16,
                    op: IrBinary::Add,
                    lhs: addr,
                    rhs: off.reg,
                    span,
                },
            );
            dst
        };
        Some(Place::Indirect {
            ptr,
            ty: field.ty,
            immutable,
        })
    }

    fn struct_base_addr(
        &mut self,
        ctx: &mut FnCtx,
        base: &Expr,
        span: SourceSpan,
    ) -> Option<(VReg, StructId, bool)> {
        match &base.kind {
            ExprKind::Unary {
                op: UnaryOp::Deref,
                expr,
            } => {
                let ptr = self.check_expr(ctx, expr, None)?;
                match ptr.ty.pointee() {
                    Some(CType::Struct(id)) => Some((ptr.reg, id, false)),
                    _ => {
                        self.emit(
                            DiagCode::TyMismatch,
                            span,
                            "field access requires a struct or pointer to struct",
                        );
                        None
                    }
                }
            }
            ExprKind::Index { base, index } => {
                let (addr, immutable) =
                    self.check_index_addr(ctx, base, index, base.span, false)?;
                match addr.ty.pointee() {
                    Some(CType::Struct(id)) => Some((addr.reg, id, immutable)),
                    _ => {
                        self.emit(
                            DiagCode::TyMismatch,
                            span,
                            "field access requires a struct element",
                        );
                        None
                    }
                }
            }
            ExprKind::Field { base: inner, name } => {
                let place = self.check_field_place(ctx, inner, name, inner.span)?;
                match place {
                    Place::Indirect {
                        ptr,
                        ty: CType::Struct(id),
                        immutable,
                    } => Some((ptr, id, immutable)),
                    _ => {
                        self.emit(
                            DiagCode::TyMismatch,
                            span,
                            "field access requires a nested struct",
                        );
                        None
                    }
                }
            }
            ExprKind::Name(name) => {
                if let Some(bind) = lookup_local(&ctx.scopes, &name.name).cloned() {
                    if let CType::Struct(id) = bind.ty {
                        let dst = self.vreg();
                        self.emit_op(
                            ctx,
                            IrOp::AddrLocal {
                                dst,
                                local: bind.id,
                                span: name.span,
                            },
                        );
                        return Some((dst, id, bind.is_const));
                    }
                    self.emit(
                        DiagCode::TyMismatch,
                        span,
                        format!("'{}' is not a struct", name.name),
                    );
                    return None;
                }
                let global = self.symbols.get(&name.name).map(|sym| {
                    (
                        sym.ty,
                        sym.id,
                        sym.is_const,
                        matches!(sym.kind, SymbolKind::Function { .. } | SymbolKind::Struct),
                    )
                });
                if let Some((ty, id, is_const, is_type_or_fn)) = global {
                    if is_type_or_fn {
                        self.emit(
                            DiagCode::TyMismatch,
                            name.span,
                            format!("'{}' is not a struct object", name.name),
                        );
                        return None;
                    }
                    if let CType::Struct(sid) = ty {
                        let dst = self.vreg();
                        self.emit_op(
                            ctx,
                            IrOp::AddrGlobal {
                                dst,
                                global: GlobalId(id),
                                span: name.span,
                            },
                        );
                        return Some((dst, sid, is_const));
                    }
                    self.emit(
                        DiagCode::TyMismatch,
                        span,
                        format!("'{}' is not a struct", name.name),
                    );
                    return None;
                }
                self.emit(
                    DiagCode::TyUnresolvedName,
                    name.span,
                    format!("unresolved name '{}'", name.name),
                );
                None
            }
            _ => {
                self.emit(
                    DiagCode::TyMismatch,
                    span,
                    "field access requires a struct lvalue",
                );
                None
            }
        }
    }

    fn check_binary(
        &mut self,
        ctx: &mut FnCtx,
        op: BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
        span: SourceSpan,
        expected: Option<CType>,
    ) -> Option<Value> {
        if matches!(op, BinaryOp::And | BinaryOp::Or) {
            return self.check_short_circuit(ctx, op, lhs, rhs, span);
        }
        let cmp = matches!(
            op,
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
        );
        let int_expected = if cmp {
            None
        } else {
            expected.filter(|t| t.is_integer())
        };
        let lhs_untyped = untyped_int_tree(lhs);
        let rhs_untyped = untyped_int_tree(rhs);
        let (left, right) = if matches!(op, BinaryOp::Shl | BinaryOp::Shr) {
            let left = if lhs_untyped.is_some() {
                self.check_expr(ctx, lhs, int_expected.or(Some(CType::U16)))?
            } else {
                self.check_expr(ctx, lhs, int_expected)?
            };
            let right = self.check_expr(ctx, rhs, Some(CType::U8))?;
            (left, right)
        } else if lhs_untyped.is_some() && rhs_untyped.is_some() {
            let ty = int_expected.unwrap_or_else(|| {
                if contains_neg(lhs) || contains_neg(rhs) {
                    CType::I16
                } else {
                    CType::U16
                }
            });
            (
                self.check_expr(ctx, lhs, Some(ty))?,
                self.check_expr(ctx, rhs, Some(ty))?,
            )
        } else if lhs_untyped.is_some() {
            let right = self.check_expr(ctx, rhs, int_expected)?;
            let left_ty =
                if right.ty.pointee().is_some() && matches!(op, BinaryOp::Add | BinaryOp::Sub) {
                    Some(CType::U16)
                } else {
                    Some(right.ty)
                };
            let left = self.check_expr(ctx, lhs, left_ty)?;
            (left, right)
        } else if rhs_untyped.is_some() {
            let left = self.check_expr(ctx, lhs, int_expected)?;
            let right_ty =
                if left.ty.pointee().is_some() && matches!(op, BinaryOp::Add | BinaryOp::Sub) {
                    Some(CType::U16)
                } else {
                    Some(left.ty)
                };
            let right = self.check_expr(ctx, rhs, right_ty)?;
            (left, right)
        } else {
            (
                self.check_expr(ctx, lhs, int_expected)?,
                self.check_expr(ctx, rhs, int_expected)?,
            )
        };
        if matches!(op, BinaryOp::Add | BinaryOp::Sub) {
            if left.ty.pointee().is_some() && right.ty.is_integer() {
                return self.ptr_offset(ctx, left, right, op == BinaryOp::Sub, span);
            }
            if op == BinaryOp::Add && right.ty.pointee().is_some() && left.ty.is_integer() {
                return self.ptr_offset(ctx, right, left, false, span);
            }
        }
        if left.ty != right.ty {
            self.emit(
                DiagCode::TyMismatch,
                span,
                format!(
                    "operands have different types {} and {}",
                    left.ty.as_str(),
                    right.ty.as_str()
                ),
            );
            return None;
        }
        if cmp {
            if left.ty == CType::Str {
                self.emit(DiagCode::TyMismatch, span, "str values cannot be compared");
                return None;
            }
            if matches!(
                op,
                BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
            ) && !left.ty.is_integer()
            {
                self.emit(
                    DiagCode::TyMismatch,
                    span,
                    "ordered comparison requires the same integer type",
                );
                return None;
            }
            let dst = self.vreg();
            let bits = match (left.bits, right.bits) {
                (Some(a), Some(b)) => {
                    let la = left.ty.interpret_bits(a);
                    let rb = right.ty.interpret_bits(b);
                    Some(u16::from(match op {
                        BinaryOp::Eq => la == rb,
                        BinaryOp::Ne => la != rb,
                        BinaryOp::Lt => la < rb,
                        BinaryOp::Le => la <= rb,
                        BinaryOp::Gt => la > rb,
                        BinaryOp::Ge => la >= rb,
                        _ => unreachable!(),
                    }))
                }
                _ => None,
            };
            self.emit_op(
                ctx,
                IrOp::Binary {
                    dst,
                    ty: CType::Bool,
                    op: ir_binary(op),
                    lhs: left.reg,
                    rhs: right.reg,
                    span,
                },
            );
            return Some(Value {
                ty: CType::Bool,
                reg: dst,
                bits,
            });
        }
        if !left.ty.is_integer() {
            self.emit(
                DiagCode::TyMismatch,
                span,
                format!("{} requires integer operands", op.as_str()),
            );
            return None;
        }
        if matches!(op, BinaryOp::Shl | BinaryOp::Shr) && right.ty != CType::U8 {
            self.emit(DiagCode::TyMismatch, span, "shift count must be u8");
            return None;
        }
        let dst = self.vreg();
        let bits = match (left.bits, right.bits) {
            (Some(a), Some(b)) => Some(fold_binary(op, left.ty, a, b)),
            _ => None,
        };
        self.emit_op(
            ctx,
            IrOp::Binary {
                dst,
                ty: left.ty,
                op: ir_binary(op),
                lhs: left.reg,
                rhs: right.reg,
                span,
            },
        );
        Some(Value {
            ty: left.ty,
            reg: dst,
            bits,
        })
    }

    fn check_short_circuit(
        &mut self,
        ctx: &mut FnCtx,
        op: BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
        span: SourceSpan,
    ) -> Option<Value> {
        let left = self.check_expr(ctx, lhs, Some(CType::Bool))?;
        let rhs_blk = self.block_id();
        let join_blk = self.block_id();
        let skip_blk = self.block_id();
        if op == BinaryOp::And {
            self.emit_op(
                ctx,
                IrOp::Branch {
                    cond: left.reg,
                    true_blk: rhs_blk,
                    false_blk: skip_blk,
                    span,
                },
            );
        } else {
            self.emit_op(
                ctx,
                IrOp::Branch {
                    cond: left.reg,
                    true_blk: skip_blk,
                    false_blk: rhs_blk,
                    span,
                },
            );
        }
        self.start_block(ctx, rhs_blk);
        let right = self.check_expr(ctx, rhs, Some(CType::Bool))?;
        self.emit_op(
            ctx,
            IrOp::Jump {
                blk: join_blk,
                span,
            },
        );
        let rhs_reg = right.reg;
        self.start_block(ctx, skip_blk);
        self.emit_op(
            ctx,
            IrOp::Jump {
                blk: join_blk,
                span,
            },
        );
        self.start_block(ctx, join_blk);
        let dst = self.vreg();
        let bits = match (left.bits, right.bits) {
            (Some(a), Some(b)) => Some(if op == BinaryOp::And {
                u16::from(a != 0 && b != 0)
            } else {
                u16::from(a != 0 || b != 0)
            }),
            (Some(0), _) if op == BinaryOp::And => Some(0),
            (Some(1), _) if op == BinaryOp::Or => Some(1),
            _ => None,
        };
        self.emit_op(
            ctx,
            IrOp::Binary {
                dst,
                ty: CType::Bool,
                op: ir_binary(op),
                lhs: left.reg,
                rhs: rhs_reg,
                span,
            },
        );
        Some(Value {
            ty: CType::Bool,
            reg: dst,
            bits,
        })
    }

    fn scale_index(&mut self, ctx: &mut FnCtx, idx: Value, scale: u16, span: SourceSpan) -> VReg {
        if scale <= 1 {
            if scale == 0 {
                return self.const_val(ctx, CType::U16, 0, span).reg;
            }
            return idx.reg;
        }
        if let Some(bits) = idx.bits {
            return self
                .const_val(ctx, CType::U16, bits.wrapping_mul(scale), span)
                .reg;
        }
        self.scale_u16(ctx, idx.reg, scale, span)
    }

    fn scale_u16(&mut self, ctx: &mut FnCtx, src: VReg, scale: u16, span: SourceSpan) -> VReg {
        if scale == 0 {
            return self.const_val(ctx, CType::U16, 0, span).reg;
        }
        if scale == 1 {
            return src;
        }
        let mut acc: Option<VReg> = None;
        let mut part = src;
        let mut bits = scale;
        loop {
            if bits & 1 != 0 {
                acc = Some(match acc {
                    None => part,
                    Some(a) => {
                        let dst = self.vreg();
                        self.emit_op(
                            ctx,
                            IrOp::Binary {
                                dst,
                                ty: CType::U16,
                                op: IrBinary::Add,
                                lhs: a,
                                rhs: part,
                                span,
                            },
                        );
                        dst
                    }
                });
            }
            bits >>= 1;
            if bits == 0 {
                break;
            }
            let doubled = self.vreg();
            self.emit_op(
                ctx,
                IrOp::Binary {
                    dst: doubled,
                    ty: CType::U16,
                    op: IrBinary::Add,
                    lhs: part,
                    rhs: part,
                    span,
                },
            );
            part = doubled;
        }
        acc.unwrap_or(src)
    }

    fn load_place(&mut self, ctx: &mut FnCtx, place: &Place, span: SourceSpan) -> Option<Value> {
        let ty = place_ty(place);
        match *place {
            Place::Local { id, is_const, .. } => {
                if !is_const && !ctx.assigned.contains(&id.0) {
                    self.emit(
                        DiagCode::TyUseBeforeAssign,
                        span,
                        "value is used before assignment",
                    );
                    return None;
                }
                let dst = self.vreg();
                self.emit_op(
                    ctx,
                    IrOp::LoadLocal {
                        dst,
                        local: id,
                        span,
                    },
                );
                Some(Value {
                    ty,
                    reg: dst,
                    bits: None,
                })
            }
            Place::Global { id, .. } => {
                let dst = self.vreg();
                self.emit_op(
                    ctx,
                    IrOp::LoadGlobal {
                        dst,
                        global: id,
                        span,
                    },
                );
                Some(Value {
                    ty,
                    reg: dst,
                    bits: None,
                })
            }
            Place::Indirect { ptr, ty, .. } => {
                let dst = self.vreg();
                self.emit_op(ctx, IrOp::LoadIndirect { dst, ptr, ty, span });
                Some(Value {
                    ty,
                    reg: dst,
                    bits: None,
                })
            }
        }
    }

    fn store_place(&mut self, ctx: &mut FnCtx, place: &Place, src: VReg, span: SourceSpan) -> bool {
        if place_immutable(place) {
            self.emit(
                DiagCode::TyAssignConst,
                span,
                "cannot assign to a const lvalue",
            );
            return false;
        }
        match *place {
            Place::Local { id, is_const, .. } => {
                if is_const {
                    self.emit(
                        DiagCode::TyAssignConst,
                        span,
                        "cannot assign to a const local",
                    );
                    return false;
                }
                self.emit_op(
                    ctx,
                    IrOp::StoreLocal {
                        local: id,
                        src,
                        span,
                    },
                );
                ctx.assigned.insert(id.0);
                true
            }
            Place::Global { id, is_const, .. } => {
                if is_const {
                    self.emit(
                        DiagCode::TyAssignConst,
                        span,
                        "cannot assign to a const global",
                    );
                    return false;
                }
                self.emit_op(
                    ctx,
                    IrOp::StoreGlobal {
                        global: id,
                        src,
                        span,
                    },
                );
                true
            }
            Place::Indirect { ptr, ty, immutable } => {
                if immutable {
                    self.emit(DiagCode::TyAssignConst, span, "cannot mutate this location");
                    return false;
                }
                self.emit_op(ctx, IrOp::StoreIndirect { ptr, src, ty, span });
                true
            }
        }
    }

    fn prepare_lvalue(&mut self, ctx: &mut FnCtx, expr: &Expr) -> Option<Place> {
        match &expr.kind {
            ExprKind::Unary {
                op: UnaryOp::Deref,
                expr: inner,
            } => {
                let ptr = self.check_expr(ctx, inner, None)?;
                let Some(elem) = ptr.ty.pointee() else {
                    self.emit(
                        DiagCode::TyMismatch,
                        expr.span,
                        format!("cannot dereference {}", ptr.ty.as_str()),
                    );
                    return None;
                };
                Some(Place::Indirect {
                    ptr: ptr.reg,
                    ty: elem,
                    immutable: false,
                })
            }
            ExprKind::Index { base, index } => {
                let (addr, immutable) =
                    self.check_index_addr(ctx, base, index, expr.span, false)?;
                let elem = addr.ty.pointee().unwrap_or(CType::U8);
                Some(Place::Indirect {
                    ptr: addr.reg,
                    ty: elem,
                    immutable,
                })
            }
            ExprKind::Field { base, name } => self.check_field_place(ctx, base, name, expr.span),
            ExprKind::Name(name) => {
                if let Some(bind) = lookup_local(&ctx.scopes, &name.name).cloned() {
                    return Some(Place::Local {
                        id: bind.id,
                        ty: bind.ty,
                        is_const: bind.is_const,
                    });
                }
                let global = self.symbols.get(&name.name).map(|sym| {
                    (
                        matches!(sym.kind, SymbolKind::Function { .. } | SymbolKind::Struct),
                        sym.ty,
                        sym.id,
                        sym.is_const,
                    )
                });
                if let Some((is_fn, ty, id, is_const)) = global {
                    if is_fn {
                        self.emit(
                            DiagCode::TyNotLvalue,
                            expr.span,
                            "cannot assign to a type or function",
                        );
                        return None;
                    }
                    Some(Place::Global {
                        id: GlobalId(id),
                        ty,
                        is_const,
                    })
                } else {
                    self.emit(
                        DiagCode::TyUnresolvedName,
                        name.span,
                        format!("unresolved name '{}'", name.name),
                    );
                    None
                }
            }
            ExprKind::Qualified { unit, name } => match self.lookup_qualified(unit, name)? {
                QSym::Function { qname, .. } => {
                    self.emit(
                        DiagCode::TyNotLvalue,
                        expr.span,
                        format!("cannot assign to function '{qname}'"),
                    );
                    None
                }
                QSym::Value {
                    id, ty, is_const, ..
                } => Some(Place::Global { id, ty, is_const }),
            },
            _ => {
                self.emit(
                    DiagCode::TyNotLvalue,
                    expr.span,
                    "assignment needs a writable scalar lvalue",
                );
                None
            }
        }
    }

    fn check_compound(
        &mut self,
        ctx: &mut FnCtx,
        op: BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
        span: SourceSpan,
    ) -> Option<Value> {
        let place = self.prepare_lvalue(ctx, lhs)?;
        let ty = place_ty(&place);
        if ty.is_aggregate() || ty == CType::Str {
            self.emit(
                DiagCode::TyNotLvalue,
                lhs.span,
                "compound assignment needs a scalar or pointer lvalue",
            );
            return None;
        }
        let old = self.load_place(ctx, &place, lhs.span)?;
        if ty.pointee().is_some() {
            if !matches!(op, BinaryOp::Add | BinaryOp::Sub) {
                self.emit(
                    DiagCode::TyMismatch,
                    span,
                    "pointer compound assignment only allows += and -=",
                );
                return None;
            }
            let rhs_val = self.check_expr(ctx, rhs, None)?;
            if !rhs_val.ty.is_integer() {
                self.emit(
                    DiagCode::TyMismatch,
                    rhs.span,
                    "pointer offset must be an integer",
                );
                return None;
            }
            let result = self.ptr_offset(ctx, old, rhs_val, op == BinaryOp::Sub, span)?;
            if !self.store_place(ctx, &place, result.reg, span) {
                return None;
            }
            return Some(result);
        }
        if !ty.is_integer() {
            self.emit(
                DiagCode::TyMismatch,
                lhs.span,
                "compound assignment needs an integer or pointer lvalue",
            );
            return None;
        }
        let rhs_ty = if matches!(op, BinaryOp::Shl | BinaryOp::Shr) {
            Some(CType::U8)
        } else {
            Some(ty)
        };
        let rhs_val = self.check_expr(ctx, rhs, rhs_ty)?;
        if matches!(op, BinaryOp::Shl | BinaryOp::Shr) {
            if rhs_val.ty != CType::U8 {
                self.emit(DiagCode::TyMismatch, rhs.span, "shift count must be u8");
                return None;
            }
        } else if rhs_val.ty != ty {
            self.emit(
                DiagCode::TyMismatch,
                span,
                format!("expected {}, found {}", ty.as_str(), rhs_val.ty.as_str()),
            );
            return None;
        }
        let dst = self.vreg();
        self.emit_op(
            ctx,
            IrOp::Binary {
                dst,
                ty,
                op: ir_binary(op),
                lhs: old.reg,
                rhs: rhs_val.reg,
                span,
            },
        );
        let result = Value {
            ty,
            reg: dst,
            bits: None,
        };
        if !self.store_place(ctx, &place, result.reg, span) {
            return None;
        }
        Some(result)
    }

    fn check_inc(
        &mut self,
        ctx: &mut FnCtx,
        op: IncOp,
        expr: &Expr,
        prefix: bool,
        span: SourceSpan,
    ) -> Option<Value> {
        let place = self.prepare_lvalue(ctx, expr)?;
        let ty = place_ty(&place);
        if ty.is_aggregate() || ty == CType::Str {
            self.emit(
                DiagCode::TyNotLvalue,
                expr.span,
                "++/-- needs an integer or pointer lvalue",
            );
            return None;
        }
        let old = self.load_place(ctx, &place, expr.span)?;
        let updated = if ty.pointee().is_some() {
            let one = self.const_val(ctx, CType::U16, 1, span);
            self.ptr_offset(ctx, old.clone(), one, matches!(op, IncOp::Dec), span)?
        } else if ty.is_integer() {
            let one = self.const_val(ctx, ty, 1, span);
            let dst = self.vreg();
            self.emit_op(
                ctx,
                IrOp::Binary {
                    dst,
                    ty,
                    op: if matches!(op, IncOp::Dec) {
                        IrBinary::Sub
                    } else {
                        IrBinary::Add
                    },
                    lhs: old.reg,
                    rhs: one.reg,
                    span,
                },
            );
            Value {
                ty,
                reg: dst,
                bits: None,
            }
        } else {
            self.emit(
                DiagCode::TyMismatch,
                expr.span,
                "++/-- needs an integer or pointer lvalue",
            );
            return None;
        };
        if !self.store_place(ctx, &place, updated.reg, span) {
            return None;
        }
        if prefix { Some(updated) } else { Some(old) }
    }

    fn ptr_offset(
        &mut self,
        ctx: &mut FnCtx,
        ptr: Value,
        offset: Value,
        subtract: bool,
        span: SourceSpan,
    ) -> Option<Value> {
        let elem = ptr.ty.pointee()?;
        let scale = elem.storage_size(&self.structs).unwrap_or(1);
        let idx16 = if offset.ty.byte_width() == Some(2) {
            offset
        } else {
            self.cast_val(ctx, offset, CType::U16, span)
        };
        let off = self.scale_index(ctx, idx16, scale, span);
        let dst = self.vreg();
        self.emit_op(
            ctx,
            IrOp::Binary {
                dst,
                ty: CType::U16,
                op: if subtract {
                    IrBinary::Sub
                } else {
                    IrBinary::Add
                },
                lhs: ptr.reg,
                rhs: off,
                span,
            },
        );
        Some(Value {
            ty: ptr.ty,
            reg: dst,
            bits: None,
        })
    }

    fn check_assign(
        &mut self,
        ctx: &mut FnCtx,
        lhs: &Expr,
        rhs: &Expr,
        span: SourceSpan,
    ) -> Option<Value> {
        if let ExprKind::Field { base, name } = &lhs.kind {
            let place = self.check_field_place(ctx, base, name, lhs.span)?;
            let ty = place_ty(&place);
            if ty.is_aggregate() {
                self.emit(
                    DiagCode::TyNotLvalue,
                    lhs.span,
                    "cannot assign to an aggregate as a whole",
                );
                return None;
            }
            let val = self.check_expr(ctx, rhs, Some(ty))?;
            if !self.store_place(ctx, &place, val.reg, span) {
                return None;
            }
            return Some(val);
        }
        if let ExprKind::Unary {
            op: UnaryOp::Deref,
            expr: inner,
        } = &lhs.kind
        {
            let ptr = self.check_expr(ctx, inner, None)?;
            let Some(elem) = ptr.ty.pointee() else {
                self.emit(
                    DiagCode::TyMismatch,
                    lhs.span,
                    format!("cannot dereference {}", ptr.ty.as_str()),
                );
                return None;
            };
            if elem.is_aggregate() {
                self.emit(
                    DiagCode::TyNotLvalue,
                    lhs.span,
                    "cannot assign to an aggregate as a whole",
                );
                return None;
            }
            let val = self.check_expr(ctx, rhs, Some(elem))?;
            self.emit_op(
                ctx,
                IrOp::StoreIndirect {
                    ptr: ptr.reg,
                    src: val.reg,
                    ty: elem,
                    span,
                },
            );
            return Some(val);
        }
        if let ExprKind::Index { base, index } = &lhs.kind {
            let (addr, immutable) = self.check_index_addr(ctx, base, index, lhs.span, false)?;
            if immutable {
                self.emit(
                    DiagCode::TyAssignConst,
                    lhs.span,
                    "cannot mutate a str payload",
                );
                return None;
            }
            let elem = addr.ty.pointee().unwrap_or(CType::U8);
            if elem.is_aggregate() {
                self.emit(
                    DiagCode::TyNotLvalue,
                    lhs.span,
                    "cannot assign to an aggregate as a whole",
                );
                return None;
            }
            let val = self.check_expr(ctx, rhs, Some(elem))?;
            self.emit_op(
                ctx,
                IrOp::StoreIndirect {
                    ptr: addr.reg,
                    src: val.reg,
                    ty: elem,
                    span,
                },
            );
            return Some(val);
        }
        if let ExprKind::Qualified { unit, name } = &lhs.kind {
            match self.lookup_qualified(unit, name)? {
                QSym::Function { qname, .. } => {
                    self.emit(
                        DiagCode::TyNotLvalue,
                        lhs.span,
                        format!("cannot assign to function '{qname}'"),
                    );
                    return None;
                }
                QSym::Value {
                    id, ty, is_const, ..
                } => {
                    if is_const {
                        self.emit(
                            DiagCode::TyAssignConst,
                            lhs.span,
                            format!("cannot assign to const '{}::{}'", unit.name, name.name),
                        );
                        return None;
                    }
                    if matches!(ty, CType::Array(_) | CType::Struct(_)) {
                        self.emit(
                            DiagCode::TyNotLvalue,
                            lhs.span,
                            "cannot assign to an aggregate as a whole",
                        );
                        return None;
                    }
                    if ty == CType::Str {
                        self.emit(
                            DiagCode::TyAssignConst,
                            lhs.span,
                            "cannot replace an owned str global",
                        );
                        return None;
                    }
                    let val = self.check_expr(ctx, rhs, Some(ty))?;
                    self.emit_op(
                        ctx,
                        IrOp::StoreGlobal {
                            global: id,
                            src: val.reg,
                            span,
                        },
                    );
                    return Some(val);
                }
            }
        }
        let ExprKind::Name(name) = &lhs.kind else {
            self.emit(
                DiagCode::TyNotLvalue,
                lhs.span,
                "assignment needs a writable scalar lvalue",
            );
            let _ = self.check_expr(ctx, rhs, None);
            return None;
        };
        if let Some(bind) = lookup_local(&ctx.scopes, &name.name).cloned() {
            if bind.is_const {
                self.emit(
                    DiagCode::TyAssignConst,
                    lhs.span,
                    format!("cannot assign to const '{}'", name.name),
                );
                return None;
            }
            let val = self.check_expr(ctx, rhs, Some(bind.ty))?;
            self.emit_op(
                ctx,
                IrOp::StoreLocal {
                    local: bind.id,
                    src: val.reg,
                    span,
                },
            );
            ctx.assigned.insert(bind.id.0);
            return Some(val);
        }
        if let Some(sym) = self.symbols.get(&name.name) {
            if matches!(sym.kind, SymbolKind::Function { .. }) {
                self.emit(
                    DiagCode::TyNotLvalue,
                    lhs.span,
                    "cannot assign to a function",
                );
                return None;
            }
            if sym.is_const {
                self.emit(
                    DiagCode::TyAssignConst,
                    lhs.span,
                    format!("cannot assign to const '{}'", name.name),
                );
                return None;
            }
            if matches!(sym.ty, CType::Array(_) | CType::Struct(_)) {
                self.emit(
                    DiagCode::TyNotLvalue,
                    lhs.span,
                    "cannot assign to an array as a whole",
                );
                return None;
            }
            if sym.ty == CType::Str {
                self.emit(
                    DiagCode::TyAssignConst,
                    lhs.span,
                    "cannot replace an owned str global",
                );
                return None;
            }
            let ty = sym.ty;
            let id = sym.id;
            let val = self.check_expr(ctx, rhs, Some(ty))?;
            self.emit_op(
                ctx,
                IrOp::StoreGlobal {
                    global: GlobalId(id),
                    src: val.reg,
                    span,
                },
            );
            return Some(val);
        }
        self.emit(
            DiagCode::TyUnresolvedName,
            name.span,
            format!("unresolved name '{}'", name.name),
        );
        None
    }

    fn check_call(
        &mut self,
        ctx: &mut FnCtx,
        callee: &Expr,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Value> {
        let (func_name, func_id, params, ret, callee_key) = match &callee.kind {
            ExprKind::Name(name) => {
                let Some(sym) = self.symbols.get(&name.name) else {
                    self.emit(
                        DiagCode::TyUnresolvedName,
                        name.span,
                        format!("unresolved name '{}'", name.name),
                    );
                    return None;
                };
                let SymbolKind::Function { params, conv: _ } = &sym.kind else {
                    self.emit(
                        DiagCode::TyMismatch,
                        name.span,
                        format!("'{}' is not a function", name.name),
                    );
                    return None;
                };
                (
                    name.name.clone(),
                    FuncId(sym.id),
                    params.clone(),
                    sym.ty,
                    self.func_key(&name.name),
                )
            }
            ExprKind::Qualified { unit, name } => match self.lookup_qualified(unit, name)? {
                QSym::Function {
                    id,
                    params,
                    ret,
                    qname,
                    ..
                } => (qname.clone(), id, params, ret, qname),
                QSym::Value { .. } => {
                    self.emit(
                        DiagCode::TyMismatch,
                        name.span,
                        format!("'{}::{}' is not a function", unit.name, name.name),
                    );
                    return None;
                }
            },
            _ => {
                self.emit(
                    DiagCode::TyMismatch,
                    callee.span,
                    "calls require a function name",
                );
                return None;
            }
        };
        if params.len() != args.len() {
            self.emit(
                DiagCode::TyMismatch,
                span,
                format!(
                    "function '{func_name}' takes {} argument(s), found {}",
                    params.len(),
                    args.len()
                ),
            );
        }
        let mut arg_regs = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let expected = params.get(i).copied();
            if let Some(val) = self.check_expr(ctx, arg, expected) {
                arg_regs.push(val.reg);
            }
        }
        self.calls
            .push((self.func_key(&ctx.name), callee_key, span));
        let dst = if ret == CType::Void {
            None
        } else {
            Some(self.vreg())
        };
        self.emit_op(
            ctx,
            IrOp::Call {
                dst,
                func: func_id,
                args: arg_regs,
                span,
            },
        );
        Some(Value {
            ty: ret,
            reg: dst.unwrap_or_else(|| self.vreg()),
            bits: None,
        })
    }

    fn check_cast(
        &mut self,
        ctx: &mut FnCtx,
        ty: &TypeExpr,
        inner: &Expr,
        span: SourceSpan,
    ) -> Option<Value> {
        let to = self.type_of(ty);
        if to == CType::Void {
            self.emit(DiagCode::TyVoidValue, ty.span, "cannot convert to void");
            return None;
        }
        let inner = self.check_expr(ctx, inner, None)?;
        if !conversion_ok(inner.ty, to) {
            self.emit(
                DiagCode::TyInvalidConversion,
                span,
                format!("cannot convert {} to {}", inner.ty.as_str(), to.as_str()),
            );
            return None;
        }
        Some(self.cast_val(ctx, inner, to, span))
    }

    fn cast_val(&mut self, ctx: &mut FnCtx, inner: Value, to: CType, span: SourceSpan) -> Value {
        if inner.ty == to {
            return inner;
        }
        if inner.ty == CType::Void || to == CType::Void {
            self.emit(
                DiagCode::TyInvalidConversion,
                span,
                "void cannot be converted",
            );
            return inner;
        }
        let dst = self.vreg();
        let bits = inner.bits.map(|b| convert_bits(inner.ty, to, b));
        self.emit_op(
            ctx,
            IrOp::Cast {
                dst,
                to,
                from: inner.ty,
                src: inner.reg,
                span,
            },
        );
        Value {
            ty: to,
            reg: dst,
            bits,
        }
    }

    fn const_val(&mut self, ctx: &mut FnCtx, ty: CType, bits: u16, span: SourceSpan) -> Value {
        let dst = self.vreg();
        self.emit_op(
            ctx,
            IrOp::Const {
                dst,
                ty,
                bits: bits & ty.mask(),
                span,
            },
        );
        Value {
            ty,
            reg: dst,
            bits: Some(bits & ty.mask()),
        }
    }

    fn eval_const_expr(&mut self, expr: &Expr, expected: Option<CType>) -> Option<u16> {
        let mut dummy = FnCtx {
            ret: CType::Void,
            name: String::new(),
            scopes: vec![HashMap::new()],
            assigned: HashSet::new(),
            locals: Vec::new(),
            params: Vec::new(),
            blocks: vec![IrBlock {
                id: BlockId(0),
                ops: Vec::new(),
            }],
            current: BlockId(0),
            loop_stack: Vec::new(),
            reachable: true,
            owner_span: expr.span,
        };
        let val = self.check_expr(&mut dummy, expr, expected)?;
        val.bits
    }

    fn emit_op(&mut self, ctx: &mut FnCtx, op: IrOp) {
        if let Some(block) = ctx.blocks.iter_mut().find(|b| b.id == ctx.current) {
            block.ops.push(op);
        }
    }

    fn start_block(&mut self, ctx: &mut FnCtx, id: BlockId) {
        if !ctx.blocks.iter().any(|b| b.id == id) {
            ctx.blocks.push(IrBlock {
                id,
                ops: Vec::new(),
            });
        }
        ctx.current = id;
    }

    fn check_recursion(&mut self) {
        let mut adj: HashMap<String, Vec<(String, SourceSpan)>> = HashMap::new();
        for (from, to, span) in &self.calls {
            adj.entry(from.clone())
                .or_default()
                .push((to.clone(), *span));
        }
        let names: Vec<String> = self.func_ids.keys().cloned().collect();
        for start in &names {
            if let Some(span) = cycle_from(&adj, start) {
                self.emit(
                    DiagCode::TyRecursion,
                    span,
                    format!("recursive call graph involving '{start}' is not allowed"),
                );
            }
        }
    }
}

fn place_ty(place: &Place) -> CType {
    match *place {
        Place::Local { ty, .. } | Place::Global { ty, .. } | Place::Indirect { ty, .. } => ty,
    }
}

fn place_immutable(place: &Place) -> bool {
    match *place {
        Place::Local { is_const, .. } | Place::Global { is_const, .. } => is_const,
        Place::Indirect { immutable, .. } => immutable,
    }
}

fn lookup_local<'a>(scopes: &'a [HashMap<String, LocalBind>], name: &str) -> Option<&'a LocalBind> {
    scopes.iter().rev().find_map(|scope| scope.get(name))
}

fn lookup_local_id(scopes: &[HashMap<String, LocalBind>], id: LocalId) -> Option<&LocalBind> {
    scopes
        .iter()
        .rev()
        .flat_map(|scope| scope.values())
        .find(|bind| bind.id == id)
}

fn literal_gpr_context(expr: &Expr, reg: AsmGpr) -> Option<CType> {
    match &expr.kind {
        ExprKind::Int(_) | ExprKind::Char(_) => {
            Some(if reg.is_pair() { CType::U16 } else { CType::U8 })
        }
        _ => None,
    }
}

fn gpr_type_ok(reg: AsmGpr, ty: CType) -> bool {
    match (reg.is_pair(), ty.byte_width()) {
        (false, Some(1)) => true,
        (true, Some(2)) => true,
        _ => false,
    }
}

fn out_type_ok(reg: AsmOutReg, ty: CType) -> bool {
    match reg {
        AsmOutReg::Carry | AsmOutReg::Zero => ty == CType::Bool,
        AsmOutReg::Gpr(r) => gpr_type_ok(r, ty),
    }
}

fn gpr_expect_desc(reg: AsmGpr) -> &'static str {
    if reg.is_pair() {
        "a word, pointer, or str"
    } else {
        "a byte or bool"
    }
}

fn out_expect_desc(reg: AsmOutReg) -> &'static str {
    match reg {
        AsmOutReg::Carry | AsmOutReg::Zero => "bool",
        AsmOutReg::Gpr(r) => gpr_expect_desc(r),
    }
}

fn out_kind(reg: AsmOutReg) -> IrAsmOutKind {
    match reg {
        AsmOutReg::Gpr(r) => IrAsmOutKind::Gpr(r),
        AsmOutReg::Carry => IrAsmOutKind::Carry,
        AsmOutReg::Zero => IrAsmOutKind::Zero,
    }
}

fn clobbers_of(clauses: &[AsmClause]) -> Vec<AsmClobber> {
    let mut out = Vec::new();
    for clause in clauses {
        if let AsmClause::Clobber { names, .. } = clause {
            for (name, _) in names {
                out.push(*name);
            }
        }
    }
    out
}

fn const_index_i32(expr: &Expr) -> Option<i32> {
    match &expr.kind {
        ExprKind::Int(lit) => lit.value.and_then(|v| i32::try_from(v).ok()),
        ExprKind::Char(b) => Some(i32::from(*b)),
        ExprKind::Unary {
            op: UnaryOp::Minus,
            expr: inner,
        } => const_index_i32(inner).map(i32::wrapping_neg),
        ExprKind::Unary {
            op: UnaryOp::Plus,
            expr: inner,
        } => const_index_i32(inner),
        _ => None,
    }
}

fn conversion_ok(from: CType, to: CType) -> bool {
    if from == to {
        return true;
    }
    if from == CType::Void || to == CType::Void {
        return false;
    }
    if from == CType::Str || to == CType::Str {
        return false;
    }
    if to == CType::Bool {
        return from.is_integer() || from.pointee().is_some();
    }
    if from == CType::Bool {
        return to.is_integer();
    }
    if from.pointee().is_some() {
        return to.pointee().is_some() || to == CType::U16;
    }
    if to.pointee().is_some() {
        return from == CType::U16 || from.pointee().is_some();
    }
    from.is_integer() && to.is_integer()
}

fn ir_binary(op: BinaryOp) -> IrBinary {
    match op {
        BinaryOp::Or => IrBinary::Or,
        BinaryOp::And => IrBinary::And,
        BinaryOp::BitOr => IrBinary::BitOr,
        BinaryOp::BitXor => IrBinary::BitXor,
        BinaryOp::BitAnd => IrBinary::BitAnd,
        BinaryOp::Eq => IrBinary::Eq,
        BinaryOp::Ne => IrBinary::Ne,
        BinaryOp::Lt => IrBinary::Lt,
        BinaryOp::Le => IrBinary::Le,
        BinaryOp::Gt => IrBinary::Gt,
        BinaryOp::Ge => IrBinary::Ge,
        BinaryOp::Shl => IrBinary::Shl,
        BinaryOp::Shr => IrBinary::Shr,
        BinaryOp::Add => IrBinary::Add,
        BinaryOp::Sub => IrBinary::Sub,
    }
}

fn convert_bits(from: CType, to: CType, bits: u16) -> u16 {
    if from == CType::Bool {
        return to.wrap_bits(i32::from(bits != 0));
    }
    if to == CType::Bool {
        return u16::from(from.interpret_bits(bits) != 0);
    }
    to.wrap_bits(from.interpret_bits(bits))
}

fn fold_binary(op: BinaryOp, ty: CType, a: u16, b: u16) -> u16 {
    let la = ty.interpret_bits(a);
    let rb = if matches!(op, BinaryOp::Shl | BinaryOp::Shr) {
        i32::from(b)
    } else {
        ty.interpret_bits(b)
    };
    let width = ty.bit_width().unwrap_or(16);
    let out = match op {
        BinaryOp::Add => la.wrapping_add(rb),
        BinaryOp::Sub => la.wrapping_sub(rb),
        BinaryOp::BitAnd => la & rb,
        BinaryOp::BitOr => la | rb,
        BinaryOp::BitXor => la ^ rb,
        BinaryOp::Shl => {
            let count = rb as u32;
            if count == 0 {
                la
            } else if count >= width {
                0
            } else {
                la.wrapping_shl(count)
            }
        }
        BinaryOp::Shr => {
            let count = rb as u32;
            if count == 0 {
                la
            } else if ty.is_signed() {
                if count >= width {
                    if la < 0 { -1 } else { 0 }
                } else {
                    la >> count
                }
            } else if count >= width {
                0
            } else {
                ((la as u32) >> count) as i32
            }
        }
        _ => la,
    };
    ty.wrap_bits(out)
}

fn untyped_int_tree(expr: &Expr) -> Option<(i32, bool)> {
    match &expr.kind {
        ExprKind::Int(lit) => {
            let v = lit.value? as i32;
            Some((v, false))
        }
        ExprKind::Unary {
            op: UnaryOp::Minus,
            expr,
        } => {
            let (v, _) = untyped_int_tree(expr)?;
            Some((v, true))
        }
        ExprKind::Unary {
            op: UnaryOp::Plus,
            expr,
        } => untyped_int_tree(expr),
        ExprKind::Binary {
            op:
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor,
            lhs,
            rhs,
        } => {
            let _ = untyped_int_tree(lhs)?;
            let _ = untyped_int_tree(rhs)?;
            Some((0, contains_neg(lhs) || contains_neg(rhs)))
        }
        _ => None,
    }
}

fn contains_neg(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Unary {
            op: UnaryOp::Minus, ..
        } => true,
        ExprKind::Binary { lhs, rhs, .. } => contains_neg(lhs) || contains_neg(rhs),
        ExprKind::Unary { expr, .. } => contains_neg(expr),
        _ => false,
    }
}

fn cycle_from(adj: &HashMap<String, Vec<(String, SourceSpan)>>, start: &str) -> Option<SourceSpan> {
    fn dfs(
        adj: &HashMap<String, Vec<(String, SourceSpan)>>,
        start: &str,
        cur: &str,
        stack: &mut Vec<String>,
        seen: &mut HashSet<String>,
    ) -> Option<SourceSpan> {
        if !seen.insert(cur.to_string()) {
            return None;
        }
        stack.push(cur.to_string());
        if let Some(edges) = adj.get(cur) {
            for (next, span) in edges {
                if next == start && stack.len() >= 1 {
                    return Some(*span);
                }
                if stack.contains(next) {
                    continue;
                }
                if let Some(s) = dfs(adj, start, next, stack, seen) {
                    return Some(s);
                }
            }
        }
        stack.pop();
        None
    }
    let mut stack = Vec::new();
    let mut seen = HashSet::new();
    dfs(adj, start, start, &mut stack, &mut seen)
}
