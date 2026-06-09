use peps::{
    bytecode::{Instruction, Value},
    vm, run_source, RunError,
};

#[test]
fn runs_printed_values() {
    let output = run_source("🐶 🟰 5️⃣ 🔚 🐱 🟰 ✅ 🔚 📢 🐶 🔚 📢 🐱 🔚")
        .expect("source should run");

    assert_eq!(output, vec!["5".to_string(), "✅".to_string()]);
}

#[test]
fn runs_newline_separated_statements_without_statement_end_token() {
    let output = run_source("🐶 🟰 5️⃣\n📢 🐶").expect("source should run");
    assert_eq!(output, vec!["5".to_string()]);
}

#[test]
fn runs_with_line_comments() {
    let output = run_source("🐶 🟰 5️⃣ // keep this value\n📢 🐶")
        .expect("source should run");
    assert_eq!(output, vec!["5".to_string()]);
}

#[test]
fn runs_logical_ops_with_short_circuit() {
    let output = run_source(
        "📢 🚫 ✅ 🔚 📢 ❌ 🤝 1️⃣ ➗ 0️⃣ ▶️ 0️⃣ 🔚 📢 ✅ 🔀 1️⃣ ➗ 0️⃣ ▶️ 0️⃣ 🔚",
    )
    .expect("source should run");

    assert_eq!(
        output,
        vec!["❌".to_string(), "❌".to_string(), "✅".to_string()]
    );
}

#[test]
fn runs_arithmetic_and_comparison() {
    let output = run_source("🐶 🟰 1️⃣ ➕ 2️⃣ ✖️ 3️⃣ 🔚 🐱 🟰 🐶 ▶️ 6️⃣ 🔚 📢 🐶 🔚 📢 🐱 🔚")
        .expect("source should run");

    assert_eq!(output, vec!["7".to_string(), "✅".to_string()]);
}

#[test]
fn runs_string_assignment_print() {
    let output = run_source("🐶 🟰 💬 hello 💬 🔚 📢 🐶 🔚").expect("source should run");
    assert_eq!(output, vec![" hello ".to_string()]);
}

#[test]
fn runs_emoji_literal_assignment_print() {
    let output = run_source("📦 🟰 🥊 🔚 📢 📦 🔚").expect("source should run");
    assert_eq!(output, vec!["🥊".to_string()]);
}

#[test]
fn runs_undeclared_emoji_reference_as_literal_print() {
    let output = run_source("📢 ✈️ 🔚").expect("source should run");
    assert_eq!(output, vec!["✈️".to_string()]);
}

#[test]
fn runs_list_print() {
    let output = run_source("🐶 🟰 📚 1️⃣ 2️⃣ 📚 🔚 📢 🐶 🔚").expect("source should run");
    assert_eq!(output, vec!["📚 1 2 📚".to_string()]);
}

#[test]
fn runs_for_each_list_loop() {
    let output =
        run_source("🍎 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚 🔚 🔁 🐾 🧭 🍎 🔓 📢 🐾 🔚 🔒")
            .expect("source should run");
    assert_eq!(output, vec!["1".to_string(), "2".to_string(), "3".to_string()]);
}

#[test]
fn runs_range_loop() {
    let output = run_source("🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓 📢 🐾 🔚 🔒")
        .expect("source should run");
    assert_eq!(output, vec!["0".to_string(), "1".to_string(), "2".to_string()]);
}

#[test]
fn runs_break_in_loop() {
    let output = run_source("🔁 ✅ 🔓 🛑 🔚 📢 1️⃣ 🔚 🔒").expect("source should run");
    assert!(output.is_empty());
}

#[test]
fn runs_continue_in_loop() {
    let output = run_source("🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓 ⏭️ 🔚 📢 🐾 🔚 🔒")
        .expect("source should run");
    assert!(output.is_empty());
}

#[test]
fn descending_range_is_empty() {
    let output = run_source("🔁 🐾 🧭 🔢 3️⃣ ➡️ 0️⃣ 🔓 📢 🐾 🔚 🔒")
        .expect("source should run");
    assert!(output.is_empty());
}

#[test]
fn reports_division_by_zero() {
    let error = run_source("🐶 🟰 1️⃣ ➗ 0️⃣ 🔚 📢 🐶 🔚").expect_err("source should fail at runtime");
    assert!(error.diagnostics[0].message.contains("division by zero"));
}

#[test]
fn stops_non_terminating_while_loop() {
    let bytecode = peps::compile_source("🐶 🟰 ✅ 🔚 🔁 🐶 🔓 📢 🐶 🔚 🔒")
        .expect("source should compile");
    let error = vm::execute_with_step_limit(&bytecode, 12)
        .expect_err("source should stop at runtime");

    assert!(!error.output.is_empty());
    assert!(error.diagnostics[0].message.contains("step limit"));
}

#[test]
fn enforces_step_limit_as_a_backup() {
    let bytecode = vec![Instruction::LoadConst(Value::Num(1)); 2];
    let error: RunError =
        vm::execute_with_step_limit(&bytecode, 1).expect_err("step limit should stop execution");

    assert!(error.diagnostics[0].message.contains("step limit"));
}
