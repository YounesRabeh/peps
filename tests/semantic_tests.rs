use peps::{lexer, parser, semantic, types::Type};

fn check(source: &str) -> Result<peps::semantic::CheckedProgram, Vec<peps::Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    semantic::check(program)
}

fn first_error(source: &str) -> String {
    check(source).expect_err("source should fail")[0].message.clone()
}

#[test]
fn infers_variable_declaration() {
    let checked = check("🐶 🟰 5️⃣ 🔚").expect("source should check");
    assert_eq!(checked.symbols.get("🐶"), Some(&Type::Num));
}

#[test]
fn infers_emoji_literal_assignment() {
    let checked = check("📦 🟰 🥊 🔚").expect("source should check");
    assert_eq!(checked.symbols.get("📦"), Some(&Type::Emoji));
}

#[test]
fn rejects_undeclared_variable() {
    assert!(first_error("📢 🐶 🔚").contains("not declared"));
}

#[test]
fn rejects_reassignment() {
    assert!(first_error("🐶 🟰 1️⃣ 🔚 🐶 🟰 2️⃣ 🔚").contains("already assigned"));
}

#[test]
fn rejects_arithmetic_type_error() {
    assert!(first_error("🐶 🟰 ✅ ➕ 1️⃣ 🔚").contains("requires num operands"));
}

#[test]
fn rejects_comparison_type_error() {
    assert!(first_error("🐶 🟰 ✅ ▶️ ❌ 🔚").contains("ordering comparison"));
}

#[test]
fn rejects_if_condition_type_error() {
    assert!(first_error("🤔 5️⃣ 🔓 📢 1️⃣ 🔚 🔒").contains("if condition must be bool"));
}

#[test]
fn rejects_while_condition_type_error() {
    assert!(first_error("🔁 5️⃣ 🔓 📢 1️⃣ 🔚 🔒").contains("while condition must be bool"));
}

#[test]
fn rejects_raw_string_in_print() {
    assert!(first_error("📢 💬 hello 💬 🔚").contains("Raw string literals"));
}

#[test]
fn rejects_raw_string_list_in_print() {
    assert!(first_error("📢 📚 💬 hello 💬 📚 🔚").contains("Raw string literals"));
}

#[test]
fn rejects_mixed_list_types() {
    assert!(first_error("🐶 🟰 📚 1️⃣ ✅ 📚 🔚").contains("same type"));
}

#[test]
fn rejects_empty_lists() {
    assert!(first_error("🐶 🟰 📚 📚 🔚").contains("empty lists"));
}

#[test]
fn rejects_nested_lists() {
    assert!(first_error("🐶 🟰 📚 📚 1️⃣ 📚 📚 🔚").contains("nested lists"));
}

#[test]
fn rejects_declaration_inside_block() {
    assert!(
        first_error("🤔 ✅ 🔓 🐶 🟰 5️⃣ 🔚 🔒").contains("inside blocks")
    );
}

#[test]
fn allows_string_list_assignment() {
    let checked = check("🐶 🟰 📚 💬 one 💬 💬 two 💬 📚 🔚").expect("source should check");
    assert_eq!(
        checked.symbols.get("🐶"),
        Some(&Type::List(Box::new(Type::Str)))
    );
}
