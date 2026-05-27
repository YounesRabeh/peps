use peps::{
    ast::{BinaryOp, Expr, ForSource, Stmt},
    lexer, parser,
};

fn parse(source: &str) -> peps::Program {
    parser::parse(lexer::lex(source).expect("lexing should succeed")).expect("parsing should succeed")
}

#[test]
fn parses_assignment() {
    let program = parse("🐶 🟰 5️⃣ 🔚");
    assert!(matches!(program.statements[0], Stmt::Assign { .. }));
}

#[test]
fn parses_ascii_assignment() {
    let program = parse("counter 🟰 5️⃣ 🔚");
    assert!(matches!(
        &program.statements[0],
        Stmt::Assign { name, .. } if name == "counter"
    ));
}

#[test]
fn parses_print() {
    let program = parse("📢 🐶 🔚");
    assert!(matches!(program.statements[0], Stmt::Print { .. }));
}

#[test]
fn parses_arithmetic_precedence() {
    let program = parse("🐶 🟰 1️⃣ ➕ 2️⃣ ✖️ 3️⃣ 🔚");
    let Stmt::Assign { expr, .. } = &program.statements[0] else {
        panic!("expected assignment");
    };
    let Expr::Binary { op, right, .. } = expr else {
        panic!("expected binary expression");
    };
    assert_eq!(*op, BinaryOp::Add);
    assert!(matches!(right.as_ref(), Expr::Binary { op: BinaryOp::Mul, .. }));
}

#[test]
fn parses_if_block() {
    let program = parse("🤔 ✅ 🔓 📢 1️⃣ 🔚 🔒");
    assert!(matches!(program.statements[0], Stmt::If { else_branch: None, .. }));
}

#[test]
fn parses_if_else_block() {
    let program = parse("🤔 ✅ 🔓 📢 1️⃣ 🔚 🔒 😐 🔓 📢 2️⃣ 🔚 🔒");
    assert!(matches!(
        program.statements[0],
        Stmt::If {
            else_branch: Some(_),
            ..
        }
    ));
}

#[test]
fn parses_while_block() {
    let program = parse("🔁 ✅ 🔓 📢 1️⃣ 🔚 🔒");
    assert!(matches!(program.statements[0], Stmt::While { .. }));
}

#[test]
fn parses_for_each_list_block() {
    let program = parse("🔁 🐾 🧭 🍎 🔓 📢 🐾 🔚 🔒");
    assert!(matches!(
        &program.statements[0],
        Stmt::For {
            variable,
            source: ForSource::List { .. },
            ..
        } if variable == "🐾"
    ));
}

#[test]
fn parses_for_range_block() {
    let program = parse("🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓 📢 🐾 🔚 🔒");
    assert!(matches!(
        &program.statements[0],
        Stmt::For {
            variable,
            source: ForSource::Range { .. },
            ..
        } if variable == "🐾"
    ));
}

#[test]
fn parses_for_range_block_with_ascii_variable() {
    let program = parse("🔁 idx 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓 📢 idx 🔚 🔒");
    assert!(matches!(
        &program.statements[0],
        Stmt::For {
            variable,
            source: ForSource::Range { .. },
            ..
        } if variable == "idx"
    ));
}

#[test]
fn errors_on_missing_range_arrow() {
    let tokens = lexer::lex("🔁 🐾 🧭 🔢 0️⃣ 3️⃣ 🔓 📢 🐾 🔚 🔒").expect("lexing should succeed");
    let diagnostics = parser::parse(tokens).expect_err("missing range arrow should fail");
    assert!(diagnostics[0].message.contains("range end arrow"));
}

#[test]
fn errors_on_malformed_for_loop() {
    let tokens = lexer::lex("🔁 🐾 🍎 🔓 📢 🐾 🔚 🔒").expect("lexing should succeed");
    let diagnostics = parser::parse(tokens).expect_err("malformed for loop should fail");
    assert!(!diagnostics[0].message.is_empty());
}

#[test]
fn parses_list() {
    let program = parse("🐶 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚 🔚");
    let Stmt::Assign { expr, .. } = &program.statements[0] else {
        panic!("expected assignment");
    };
    assert!(matches!(expr, Expr::List { elements, .. } if elements.len() == 3));
}

#[test]
fn errors_on_missing_statement_end() {
    let tokens = lexer::lex("🐶 🟰 5️⃣").expect("lexing should succeed");
    let diagnostics = parser::parse(tokens).expect_err("missing statement end should fail");
    assert!(diagnostics[0].message.contains("missing statement end"));
}

#[test]
fn errors_on_missing_block_end() {
    let tokens = lexer::lex("🤔 ✅ 🔓 📢 1️⃣ 🔚").expect("lexing should succeed");
    let diagnostics = parser::parse(tokens).expect_err("missing block end should fail");
    assert!(diagnostics[0].message.contains("missing block end"));
}

#[test]
fn errors_on_multi_emoji_variable() {
    let tokens = lexer::lex("🐶🐱 🟰 5️⃣ 🔚").expect("lexing should succeed");
    let diagnostics = parser::parse(tokens).expect_err("multi emoji variable should fail");
    assert!(diagnostics[0].message.contains("exactly one emoji"));
}

#[test]
fn errors_on_multi_emoji_variable_reference() {
    let tokens = lexer::lex("📢 🐶🐱 🔚").expect("lexing should succeed");
    let diagnostics = parser::parse(tokens).expect_err("multi emoji variable should fail");
    assert!(diagnostics[0].message.contains("exactly one emoji"));
}
