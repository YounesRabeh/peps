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
fn rejects_ascii_variable_declaration() {
    assert!(first_error("count 🟰 5️⃣ 🔚").contains("exactly one emoji"));
}

#[test]
fn infers_emoji_literal_assignment() {
    let checked = check("📦 🟰 🥊 🔚").expect("source should check");
    assert_eq!(checked.symbols.get("📦"), Some(&Type::Emoji));
}

#[test]
fn allows_newline_separated_statements_without_statement_end_token() {
    check("🐶 🟰 5️⃣\n📢 🐶").expect("source should check");
}

#[test]
fn allows_logical_ops() {
    check("🐶 🟰 🚫 ✅ 🤝 ❌ 🔀 ✅ 🔚").expect("source should check");
}

#[test]
fn allows_list_ops() {
    let checked = check(
        "🍎 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚 🔚 🐶 🟰 📏 🍎 🔚 🐱 🟰 🍎 🔎 1️⃣ 🔚 🦊 🟰 🍎 📥 4️⃣ 🔚",
    )
    .expect("source should check");
    assert_eq!(checked.symbols.get("🐶"), Some(&Type::Num));
    assert_eq!(checked.symbols.get("🐱"), Some(&Type::Num));
    assert_eq!(
        checked.symbols.get("🦊"),
        Some(&Type::List(Box::new(Type::Num)))
    );
}

#[test]
fn treats_undeclared_emoji_reference_as_literal() {
    check("📢 🐶 🔚").expect("undeclared emoji references should be treated as literals");
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
fn rejects_logical_type_error() {
    assert!(first_error("🐶 🟰 🚫 5️⃣ 🔚").contains("logical not"));
    assert!(first_error("🐶 🟰 5️⃣ 🤝 ✅ 🔚").contains("logical operator"));
}

#[test]
fn rejects_list_op_type_errors() {
    assert!(first_error("🐶 🟰 📏 5️⃣ 🔚").contains("list length"));
    assert!(first_error("🐶 🟰 5️⃣ 🔎 0️⃣ 🔚").contains("list value on the left"));
    assert!(first_error("🍎 🟰 📚 1️⃣ 2️⃣ 📚 🔚 🐶 🟰 🍎 🔎 ✅ 🔚").contains("num index"));
    assert!(first_error("🍎 🟰 📚 1️⃣ 2️⃣ 📚 🔚 🐶 🟰 🍎 📥 ✅ 🔚").contains("element type"));
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
fn allows_for_each_over_list() {
    check("🍎 🟰 📚 1️⃣ 2️⃣ 📚 🔚 🔁 🐾 🧭 🍎 🔓 📢 🐾 🔚 🔒")
        .expect("source should check");
}

#[test]
fn allows_for_range() {
    check("🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓 📢 🐾 🔚 🔒").expect("source should check");
}

#[test]
fn allows_break_and_continue_inside_loops() {
    check("🔁 ✅ 🔓 ⏭️ 🔚 🔒 🔁 ✅ 🔓 🛑 🔚 🔒").expect("source should check");
}

#[test]
fn rejects_for_each_over_non_list() {
    assert!(first_error("🐶 🟰 5️⃣ 🔚 🔁 🐾 🧭 🐶 🔓 📢 🐾 🔚 🔒").contains("source must be a list"));
}

#[test]
fn rejects_break_outside_loop() {
    assert!(first_error("🛑 🔚").contains("inside loops"));
}

#[test]
fn rejects_continue_outside_loop() {
    assert!(first_error("⏭️ 🔚").contains("inside loops"));
}

#[test]
fn rejects_non_num_range_bounds() {
    assert!(first_error("🔁 🐾 🧭 🔢 ✅ ➡️ 3️⃣ 🔓 📢 🐾 🔚 🔒").contains("range bounds"));
}

#[test]
fn treats_loop_variable_reference_after_loop_as_literal() {
    check("🔁 🐾 🧭 🔢 0️⃣ ➡️ 1️⃣ 🔓 📢 🐾 🔚 🔒 📢 🐾 🔚")
        .expect("loop variable outside loop should be treated as emoji literal");
}

#[test]
fn rejects_loop_variable_name_conflict() {
    assert!(first_error("🐾 🟰 1️⃣ 🔚 🔁 🐾 🧭 🔢 0️⃣ ➡️ 1️⃣ 🔓 📢 🐾 🔚 🔒")
        .contains("already declared"));
}

#[test]
fn allows_string_list_assignment() {
    let checked = check("🐶 🟰 📚 💬 one 💬 💬 two 💬 📚 🔚").expect("source should check");
    assert_eq!(
        checked.symbols.get("🐶"),
        Some(&Type::List(Box::new(Type::Str)))
    );
}
