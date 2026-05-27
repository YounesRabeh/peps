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
fn runs_list_print() {
    let output = run_source("🐶 🟰 📚 1️⃣ 2️⃣ 📚 🔚 📢 🐶 🔚").expect("source should run");
    assert_eq!(output, vec!["📚 1 2 📚".to_string()]);
}

#[test]
fn reports_division_by_zero() {
    let error = run_source("🐶 🟰 1️⃣ ➗ 0️⃣ 🔚 📢 🐶 🔚").expect_err("source should fail at runtime");
    assert!(error.diagnostics[0].message.contains("division by zero"));
}

#[test]
fn stops_non_terminating_while_loop() {
    let error = run_source("🐶 🟰 ✅ 🔚 🔁 🐶 🔓 📢 🐶 🔚 🔒")
        .expect_err("source should stop at runtime");

    assert_eq!(error.output, vec!["✅".to_string()]);
    assert!(error.diagnostics[0].message.contains("while loop did not terminate"));
}

#[test]
fn enforces_step_limit_as_a_backup() {
    let bytecode = vec![Instruction::LoadConst(Value::Num(1)); 2];
    let error: RunError =
        vm::execute_with_step_limit(&bytecode, 1).expect_err("step limit should stop execution");

    assert!(error.diagnostics[0].message.contains("step limit"));
}
