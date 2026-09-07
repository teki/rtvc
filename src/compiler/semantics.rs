//! Name resolution, scalar typing, definite assignment, and IR lowering.

use super::ast::*;
use super::diagnostic::{DiagCode, Diagnostic};
use super::ir::*;
use super::source::{NodeId, SourceSpan};
use super::types::{ArrayElem, ArrayType, CType, PtrType};
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

struct Value {
    ty: CType,
    reg: VReg,
    bits: Option<u16>,
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

    fn collect(&mut self, unit: &TranslationUnit) {
        for item in &unit.items {
            match item {
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
        TypedProgram { globals, functions }
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
            let size = u32::from(elem.byte_width()) * u32::from(len);
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
        let init = if matches!(ty, CType::Str | CType::Array(_)) {
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

    fn check_function(&mut self, func: &Function) -> TypedFunction {
        let ret = self.type_of(&func.return_ty);
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
            Stmt::Error { .. } => {}
        }
    }

    fn check_local(&mut self, ctx: &mut FnCtx, decl: &VarDecl) {
        if decl.array_len.is_some() {
            self.emit(
                DiagCode::TyMismatch,
                decl.name.span,
                "local arrays are not supported; use a global array or a pointer",
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
            ExprKind::Call { callee, args } => self.check_call(ctx, callee, args, expr.span)?,
            ExprKind::Index { base, index } => self.check_index(ctx, base, index, expr.span)?,
            ExprKind::Field { base, name } => self.check_field(ctx, base, name, expr.span)?,
            ExprKind::Cast { ty, expr: inner } => self.check_cast(ctx, ty, inner, expr.span)?,
            ExprKind::Sizeof { ty } => {
                let cty = self.type_of(ty);
                match cty.byte_width() {
                    Some(w) => {
                        let out_ty = expected.filter(|t| t.is_integer()).unwrap_or(CType::U16);
                        if !out_ty.is_integer() {
                            self.emit(DiagCode::TyMismatch, expr.span, "sizeof result is u16");
                            return None;
                        }
                        let val = self.const_val(ctx, CType::U16, u16::from(w), expr.span);
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
        let scale = elem.byte_width().unwrap_or(1);
        let idx16 = if idx.ty.byte_width() == Some(2) {
            idx
        } else {
            self.cast_val(ctx, idx, CType::U16, index.span)
        };
        let mut off = idx16.reg;
        if scale == 2 {
            let doubled = self.vreg();
            self.emit_op(
                ctx,
                IrOp::Binary {
                    dst: doubled,
                    ty: CType::U16,
                    op: IrBinary::Add,
                    lhs: off,
                    rhs: off,
                    span,
                },
            );
            off = doubled;
        }
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
        let base_val = self.check_expr(ctx, base, None)?;
        if name.name != "len" || base_val.ty != CType::Str {
            self.emit(
                DiagCode::TyMismatch,
                name.span,
                format!("unknown field '{}'", name.name),
            );
            return None;
        }
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
        Some(Value {
            ty: CType::U8,
            reg: dst,
            bits: None,
        })
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

    fn ptr_offset(
        &mut self,
        ctx: &mut FnCtx,
        ptr: Value,
        offset: Value,
        subtract: bool,
        span: SourceSpan,
    ) -> Option<Value> {
        let elem = ptr.ty.pointee()?;
        let scale = elem.byte_width().unwrap_or(1);
        let idx16 = if offset.ty.byte_width() == Some(2) {
            offset
        } else {
            self.cast_val(ctx, offset, CType::U16, span)
        };
        let mut off = idx16.reg;
        if scale == 2 {
            let doubled = self.vreg();
            self.emit_op(
                ctx,
                IrOp::Binary {
                    dst: doubled,
                    ty: CType::U16,
                    op: IrBinary::Add,
                    lhs: off,
                    rhs: off,
                    span,
                },
            );
            off = doubled;
        }
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
                    if matches!(ty, CType::Array(_)) {
                        self.emit(
                            DiagCode::TyNotLvalue,
                            lhs.span,
                            "cannot assign to an array as a whole",
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
            if matches!(sym.ty, CType::Array(_)) {
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

fn lookup_local<'a>(scopes: &'a [HashMap<String, LocalBind>], name: &str) -> Option<&'a LocalBind> {
    scopes.iter().rev().find_map(|scope| scope.get(name))
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
