use super::lexer::lex;
use super::token::TokenKind;
use super::*;

fn compile_src(src: &str) -> CompilationResult {
    compile_source("test.c80", src)
}

fn first_function<'a>(result: &'a CompilationResult) -> &'a Function {
    result
        .units
        .iter()
        .flat_map(|unit| unit.items.iter())
        .find_map(|item| match item {
            Item::Function(func) => Some(func),
            _ => None,
        })
        .expect("expected a function")
}

fn function_named<'a>(result: &'a CompilationResult, name: &str) -> &'a Function {
    result
        .units
        .iter()
        .flat_map(|unit| unit.items.iter())
        .find_map(|item| match item {
            Item::Function(func) if func.name.name == name => Some(func),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected function {name}"))
}

fn expr_tree(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Name(id) => id.name.clone(),
        ExprKind::Int(lit) => lit.text.clone(),
        ExprKind::Char(_) => "char".to_string(),
        ExprKind::String(_) => "str".to_string(),
        ExprKind::Bool(true) => "true".to_string(),
        ExprKind::Bool(false) => "false".to_string(),
        ExprKind::Unary { op, expr } => {
            let op = match op {
                UnaryOp::Plus => "+",
                UnaryOp::Minus => "-",
                UnaryOp::Not => "!",
                UnaryOp::BitNot => "~",
                UnaryOp::Deref => "*",
                UnaryOp::AddrOf => "&",
            };
            format!("({op} {})", expr_tree(expr))
        }
        ExprKind::Binary { op, lhs, rhs } => {
            format!("({} {} {})", op.as_str(), expr_tree(lhs), expr_tree(rhs))
        }
        ExprKind::Assign { lhs, rhs } => format!("(= {} {})", expr_tree(lhs), expr_tree(rhs)),
        ExprKind::Call { callee, args } => {
            let args = args.iter().map(expr_tree).collect::<Vec<_>>().join(", ");
            format!("(call {} {args})", expr_tree(callee))
        }
        ExprKind::Index { base, index } => {
            format!("([] {} {})", expr_tree(base), expr_tree(index))
        }
        ExprKind::Field { base, name } => format!("(. {} {})", expr_tree(base), name.name),
        ExprKind::Cast { ty, expr } => format!("({} {})", ty.kind.as_str(), expr_tree(expr)),
        ExprKind::Sizeof { ty } => format!("(sizeof {})", ty.kind.as_str()),
        ExprKind::Error => "<error>".to_string(),
    }
}

fn first_expr_stmt(src: &str) -> String {
    let result = compile_src(src);
    let func = first_function(&result);
    match func.body.stmts.first() {
        Some(Stmt::Expr(stmt)) => expr_tree(&stmt.expr),
        other => panic!(
            "expected expression statement, got {other:?}\n{src:?}\n{:?}",
            result.diagnostics
        ),
    }
}

fn codes(result: &CompilationResult) -> Vec<&'static str> {
    result
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| d.code.as_str())
        .collect()
}

#[test]
fn utf8_offsets_with_ascii_identifiers() {
    let src = "// café\nvoid ident_x() { u8 y = 1; }\n";
    let result = compile_src(src);
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let func = first_function(&result);
    let start = func.name.span.start as usize;
    let end = func.name.span.end as usize;
    assert!(src.is_char_boundary(start));
    assert!(src.is_char_boundary(end));
    assert_eq!(&src[start..end], "ident_x");
    let y = match func.body.stmts.first() {
        Some(Stmt::Decl(decl)) => &decl.name,
        _ => panic!("expected local"),
    };
    assert_eq!(&src[y.span.start as usize..y.span.end as usize], "y");
    let (line, col) = result.sources.line_col(func.name.span);
    assert_eq!(line, 2);
    assert!(col >= 6);
}

#[test]
fn comments_and_escapes() {
    let src = r#"
void f() {
    // line comment
    /* block
       comment */
    u8 a = '\n';
    u8 b = '\x41';
}
"#;
    let result = compile_src(src);
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let func = first_function(&result);
    let stmts: Vec<_> = func.body.stmts.iter().collect();
    match stmts[0] {
        Stmt::Decl(decl) => match &decl.init {
            Some(expr) => assert_eq!(expr.kind, ExprKind::Char(b'\n')),
            _ => panic!("missing init"),
        },
        _ => panic!("expected decl"),
    }
    match stmts[1] {
        Stmt::Decl(decl) => match &decl.init {
            Some(expr) => assert_eq!(expr.kind, ExprKind::Char(b'A')),
            _ => panic!("missing init"),
        },
        _ => panic!("expected decl"),
    }

    let file = SourceFile::new(FileId(0), "t.c80", r#"u8 s = "hi\t\x21";"#);
    let mut diags = Vec::new();
    let lexed = lex(&file, &mut diags);
    assert!(diags.is_empty(), "{diags:?}");
    assert_eq!(lexed.strings, vec![b"hi\t!".to_vec()]);
}

#[test]
fn malformed_literals() {
    let src = r#"
void f() {
    u8 a = 0x;
    u8 b = 0xG;
    u8 c = '';
    u8 d = 'ab';
    u8 e = '\q';
    u8 f = 999999999999;
}
"#;
    let result = compile_src(src);
    let found = codes(&result);
    assert!(found.contains(&"lex-malformed-literal"), "{found:?}");
    assert!(found.contains(&"lex-invalid-escape"), "{found:?}");
    assert!(found.contains(&"lex-integer-overflow"), "{found:?}");
    assert!(result.has_errors());
}

#[test]
fn unterminated_strings_and_comments() {
    let src = "void f() { u8 a = \"nope; }\n";
    let result = compile_src(src);
    assert!(
        codes(&result).contains(&"lex-unterminated-string"),
        "{:?}",
        codes(&result)
    );

    let src = "void f() { u8 a = 'x; }\n";
    let result = compile_src(src);
    assert!(
        codes(&result).contains(&"lex-unterminated-char"),
        "{:?}",
        codes(&result)
    );

    let src = "void f() { /* unterminated\n";
    let result = compile_src(src);
    assert!(
        codes(&result).contains(&"lex-unterminated-block-comment"),
        "{:?}",
        codes(&result)
    );
}

#[test]
fn precedence_and_associativity() {
    assert_eq!(first_expr_stmt("void f() { a = b = c; }"), "(= a (= b c))");
    assert_eq!(
        first_expr_stmt("void f() { a || b && c; }"),
        "(|| a (&& b c))"
    );
    assert_eq!(
        first_expr_stmt("void f() { a | b ^ c & d; }"),
        "(| a (^ b (& c d)))"
    );
    assert_eq!(
        first_expr_stmt("void f() { a == b < c; }"),
        "(== a (< b c))"
    );
    // F001: +/- binds tighter than << >>
    assert_eq!(
        first_expr_stmt("void f() { a << b + c; }"),
        "(<< a (+ b c))"
    );
    assert_eq!(
        first_expr_stmt("void f() { a + b << c; }"),
        "(<< (+ a b) c)"
    );
    assert_eq!(first_expr_stmt("void f() { a + b + c; }"), "(+ (+ a b) c)");
    assert_eq!(first_expr_stmt("void f() { -a + b; }"), "(+ (- a) b)");
    assert_eq!(first_expr_stmt("void f() { !a && b; }"), "(&& (! a) b)");
    assert_eq!(
        first_expr_stmt("void f() { f(a, b + c); }"),
        "(call f a, (+ b c))"
    );
    assert_eq!(first_expr_stmt("void f() { u8(x) + 1; }"), "(+ (u8 x) 1)");
}

#[test]
fn incomplete_declarations_and_blocks() {
    let result = compile_src("void f() { u8 ; }");
    assert!(result.has_errors());
    assert!(
        codes(&result).iter().any(|c| c.starts_with("parse-")),
        "{:?}",
        codes(&result)
    );

    let result = compile_src("void f() { u8 x = ; }");
    assert!(result.has_errors());

    let result = compile_src("void f(");
    assert!(result.has_errors());
    assert!(
        codes(&result).contains(&"parse-incomplete") || codes(&result).contains(&"parse-expected"),
        "{:?}",
        codes(&result)
    );
}

#[test]
fn recovery_reaches_later_function() {
    let src = r#"
void broken() { u8 x =
void ok() { return; }
"#;
    let result = compile_src(src);
    assert!(result.has_errors());
    let ok = function_named(&result, "ok");
    assert_eq!(ok.name.name, "ok");
    assert!(matches!(ok.body.stmts.first(), Some(Stmt::Return(_))));
    assert!(
        result
            .diagnostics
            .iter()
            .all(|d| d.span.end >= d.span.start),
        "spans should be located"
    );
}

#[test]
fn in_memory_compile_is_deterministic() {
    let result = compile_src("pub void main() { return; }");
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let main = first_function(&result);
    assert!(main.is_pub);
    assert_eq!(main.return_ty.kind, TypeKind::Void);
    assert_eq!(main.name.name, "main");
    assert_eq!(
        main.id.0,
        first_function(&compile_src("pub void main() { return; }"))
            .id
            .0
    );
}

#[test]
fn lexer_keeps_comment_bytes_out_of_tokens() {
    let file = SourceFile::new(FileId(0), "t.c80", "u8 x; // comment\n");
    let mut diags = Vec::new();
    let lexed = lex(&file, &mut diags);
    assert!(diags.is_empty(), "{diags:?}");
    let kinds: Vec<_> = lexed.tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::U8,
            TokenKind::Ident,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn ptr_str_array_and_deref_parse() {
    let result = compile_src(
        r#"
u8 positions[16];
pub str enemy_name = "hi";
u8 f(ptr<u8> p) { return *p + p[1]; }
"#,
    );
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let positions = result
        .units
        .iter()
        .flat_map(|u| u.items.iter())
        .find_map(|item| match item {
            Item::Decl(d) if d.name.name == "positions" => Some(d),
            _ => None,
        })
        .unwrap();
    assert!(positions.array_len.is_some());
    let f = function_named(&result, "f");
    assert!(matches!(f.params[0].ty.kind, TypeKind::Ptr(_)));
}

#[test]
fn attributes_before_and_after_pub() {
    let a = compile_src("@stackcall pub u16 add(u16 a, u16 b) { return a + b; }");
    let b = compile_src("pub @stackcall u16 add(u16 a, u16 b) { return a + b; }");
    let c = compile_src("@fastcall u8 id(u8 x) { return x; }");
    assert!(!a.has_errors(), "{:?}", a.diagnostics);
    assert!(!b.has_errors(), "{:?}", b.diagnostics);
    assert!(!c.has_errors(), "{:?}", c.diagnostics);
    assert_eq!(first_function(&a).conv, CallConv::Stack);
    assert_eq!(first_function(&b).conv, CallConv::Stack);
    assert_eq!(first_function(&c).conv, CallConv::Register);
}

#[test]
fn attribute_errors_are_located() {
    let unknown = compile_src("@nope u8 f() { return 1; }");
    assert!(
        codes(&unknown).contains(&"parse-expected"),
        "{:?}",
        codes(&unknown)
    );
    let dup = compile_src("@stackcall @stackcall u8 f() { return 1; }");
    assert!(codes(&dup).contains(&"parse-expected"), "{:?}", codes(&dup));
    let conflict = compile_src("@stackcall @fastcall u8 f() { return 1; }");
    assert!(
        codes(&conflict).contains(&"parse-expected"),
        "{:?}",
        codes(&conflict)
    );
    let on_data = compile_src("@stackcall u8 x;");
    assert!(
        codes(&on_data).contains(&"parse-expected"),
        "{:?}",
        codes(&on_data)
    );
}
