use num_bigint::BigInt;

use peps::{
    bytecode::{Instruction, Value},
    run_source, run_source_with_inputs, vm, RunError,
};

#[test]
fn runs_printed_values() {
    let output =
        run_source("🐶 🟰 5️⃣ 🔚 🐱 🟰 ✅ 🔚 📢 🐶 🔚 📢 🐱 🔚").expect("source should run");

    assert_eq!(output, vec!["5".to_string(), "✅".to_string()]);
}

#[test]
fn runs_reassignment_and_updates_outer_bindings_from_blocks() {
    let output = run_source("🐶 🟰 1️⃣ 🔚 🤔 ✅ 🔓 🐶 🟰 🐶 ➕ 1️⃣ 🔚 🔒 📢 🐶 🔚")
        .expect("source should run");
    assert_eq!(output, vec!["2".to_string()]);
}

#[test]
fn runs_reassignment_with_a_different_type() {
    let output = run_source("🐶 🟰 2️⃣ 🔚 🐶 🟰 ✅ 🔚 📢 🐶 🔚").expect("source should run");
    assert_eq!(output, vec!["✅".to_string()]);
}

#[test]
fn block_reassignment_can_change_an_outer_binding_type() {
    let output = run_source("🐶 🟰 2️⃣ 🔚 🤔 ✅ 🔓 🐶 🟰 💬 yes 💬 🔚 🔒 📢 🐶 🔚")
        .expect("source should run");
    assert_eq!(output, vec![" yes ".to_string()]);
}

#[test]
fn keeps_block_locals_visible_only_inside_their_scope() {
    let output =
        run_source("🤔 ✅ 🔓 🐶 🟰 5️⃣ 🔚 📢 🐶 🔚 🔒 📢 🐶 🔚").expect("source should run");
    assert_eq!(output, vec!["5".to_string(), "🐶".to_string()]);
}

#[test]
fn evaluates_a_fresh_declaration_rhs_before_binding_its_name() {
    let output = run_source("🤔 ✅ 🔓 🐶 🟰 🐶 🔚 📢 🐶 🔚 🔒").expect("source should run");
    assert_eq!(output, vec!["🐶".to_string()]);
}

#[test]
fn keeps_sibling_branch_locals_independent() {
    let output = run_source("🤔 ❌ 🔓 🐶 🟰 1️⃣ 🔚 🔒 😐 🔓 🐶 🟰 ✅ 🔚 📢 🐶 🔚 🔒 📢 🐶 🔚")
        .expect("source should run");
    assert_eq!(output, vec!["✅".to_string(), "🐶".to_string()]);
}

#[test]
fn runs_newline_separated_statements_without_statement_end_token() {
    let output = run_source("🐶 🟰 5️⃣\n📢 🐶").expect("source should run");
    assert_eq!(output, vec!["5".to_string()]);
}

#[test]
fn runs_with_line_comments() {
    let output = run_source("🐶 🟰 5️⃣ // keep this value\n📢 🐶").expect("source should run");
    assert_eq!(output, vec!["5".to_string()]);
}

#[test]
fn runs_logical_ops_with_short_circuit() {
    let output = run_source("📢 🚫 ✅ 🔚 📢 ❌ 🤝 1️⃣ ➗ 0️⃣ ▶️ 0️⃣ 🔚 📢 ✅ 🔀 1️⃣ ➗ 0️⃣ ▶️ 0️⃣ 🔚")
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
fn runs_float_and_mixed_numeric_operations() {
    let output = run_source(
        "📢 1️⃣.5️⃣ ➕ 2️⃣ 🔚 📢 5️⃣ ➖ 2️⃣.5️⃣ 🔚 📢 1️⃣.5️⃣ ✖️ 2️⃣ 🔚 📢 1️⃣.0️⃣ ➗ 4️⃣ 🔚 📢 2️⃣.5️⃣ ▶️ 2️⃣ 🔚 📢 2️⃣ 🟰🟰 2️⃣.0️⃣ 🔚",
    )
    .expect("float operations should run");
    assert_eq!(output, vec!["3.5", "2.5", "3", "0.25", "✅", "✅"]);
}

#[test]
fn runs_negative_values_in_expressions_lists_and_ranges() {
    let output = run_source(
        "🐶 🟰 2️⃣ 🔚 📢 ➖5️⃣ 🔚 📢 ➖1️⃣.5️⃣ 🔚 📢 ➖🐶 🔚 🐱 🟰 ➖2️⃣ 🔚 🐼 🟰 ➖1️⃣ 🔚 🍎 🟰 📚 🐱 🐼 0️⃣ 📚 🔚 📢 🍎 🔚 🔁 🐾 🧭 🔢 ➖2️⃣ ➡️ 2️⃣ 🔓 📢 🐾 🔚 🔒",
    )
    .expect("negative values should run");
    assert_eq!(
        output,
        vec!["-5", "-1.5", "-2", "📚 -2 -1 0 📚", "-2", "-1", "0", "1"]
    );
}

#[test]
fn reads_text_integer_float_and_boolean_input() {
    let output = run_source_with_inputs(
        "📝 🟰 ⌨️ 🔤 🔚 🐶 🟰 ⌨️ 🔢 🔚 🦊 🟰 ⌨️ 🔣 🔚 🐱 🟰 ⌨️ ☑️ 🔚 📢 📝 🔚 📢 🐶 ➕ 1️⃣ 🔚 📢 🦊 ➕ 0️⃣.5️⃣ 🔚 📢 🐱 🔚",
        ["hello Peps", "41", "2.5", "✅"],
    )
    .expect("typed input should run");
    assert_eq!(output, vec!["hello Peps", "42", "3", "✅"]);
}

#[test]
fn reports_exhausted_and_invalid_input() {
    let exhausted = run_source("🐶 🟰 ⌨️ 🔢 🔚").expect_err("input should be required");
    assert_eq!(exhausted.diagnostics[0].message, "input required: integer");

    let invalid = run_source_with_inputs("🐶 🟰 ⌨️ 🔢 🔚", ["not-a-number"])
        .expect_err("invalid integer should fail");
    assert!(invalid.diagnostics[0].message.contains("valid integer"));
}

#[test]
fn runs_explicit_text_and_integer_conversions() {
    let output = run_source(
        "🐶 🟰 🔄 🔢 💬42💬 🔚 🦊 🟰 🔄 🔣 🐶 🔚 🐱 🟰 🔄 🔣 💬3.5💬 🔚 📢 🐶 ➕ 1️⃣ 🔚 📢 🦊 🔚 📢 🐱 ➕ 0️⃣.5️⃣ 🔚",
    )
    .expect("explicit conversions should run");
    assert_eq!(output, vec!["43", "42", "4"]);
}

#[test]
fn explicit_integer_to_float_conversion_allows_rounding() {
    let output = run_source("📢 🔄 🔣 9️⃣0️⃣0️⃣7️⃣1️⃣9️⃣9️⃣2️⃣5️⃣4️⃣7️⃣4️⃣0️⃣9️⃣9️⃣3️⃣ 🔚")
        .expect("explicit conversion should permit float rounding");
    assert_eq!(output, vec!["9007199254740992"]);
}

#[test]
fn reports_invalid_text_conversions() {
    let integer_error =
        run_source("📢 🔄 🔢 💬hello💬 🔚").expect_err("invalid integer text should fail");
    assert!(integer_error.diagnostics[0]
        .message
        .contains("not a valid integer"));

    let float_error =
        run_source("📢 🔄 🔣 💬not-a-float💬 🔚").expect_err("invalid float text should fail");
    assert!(float_error.diagnostics[0]
        .message
        .contains("not a valid float"));

    let non_finite_error =
        run_source("📢 🔄 🔣 💬infinity💬 🔚").expect_err("non-finite float text should fail");
    assert!(non_finite_error.diagnostics[0].message.contains("finite"));
}

#[test]
fn validates_dynamic_function_conversions_at_runtime() {
    let output = run_source("🧩 🧪 📚 🐾 📚 🔓 ↩️ 🔄 🔢 🐾 🔒 📝 🟰 💬7💬 🔚 📢 📞 🧪 📚 📝 📚 🔚")
        .expect("text parameter should convert at runtime");
    assert_eq!(output, vec!["7"]);

    let error = run_source("🧩 🧪 📚 🐾 📚 🔓 ↩️ 🔄 🔢 🐾 🔒 🐱 🟰 ✅ 🔚 📢 📞 🧪 📚 🐱 📚 🔚")
        .expect_err("boolean parameter should fail conversion at runtime");
    assert!(error.diagnostics[0]
        .message
        .contains("integer conversion requires text"));
}

#[test]
fn reports_float_division_by_zero() {
    let error = run_source("📢 1️⃣.0️⃣ ➗ 0️⃣.0️⃣ 🔚").expect_err("float division by zero should fail");
    assert!(error.diagnostics[0].message.contains("division by zero"));
}

#[test]
fn rejects_lossy_integer_to_float_promotion() {
    let error = run_source("📢 9️⃣0️⃣0️⃣7️⃣1️⃣9️⃣9️⃣2️⃣5️⃣4️⃣7️⃣4️⃣0️⃣9️⃣9️⃣3️⃣ ➕ 0️⃣.5️⃣ 🔚")
        .expect_err("lossy promotion should fail");
    assert!(error.diagnostics[0]
        .message
        .contains("cannot represent this integer exactly as a float"));
}

#[test]
fn runs_arithmetic_beyond_i64_limits() {
    let output = run_source("📢 9️⃣2️⃣2️⃣3️⃣3️⃣7️⃣2️⃣0️⃣3️⃣6️⃣8️⃣5️⃣4️⃣7️⃣7️⃣5️⃣8️⃣0️⃣8️⃣ ➕ 1️⃣ 🔚")
        .expect("source should run");
    assert_eq!(output, vec!["9223372036854775809".to_string()]);
}

#[test]
fn run_source_is_not_stopped_by_the_ide_step_limit() {
    let output = run_source("🐶 🟰 0️⃣ 🔚 🔁 🐶 ◀️ 2️⃣0️⃣0️⃣0️⃣0️⃣ 🔓 🐶 🟰 🐶 ➕ 1️⃣ 🔚 🔒 📢 🐶 🔚")
        .expect("core execution should be unlimited");
    assert_eq!(output, vec!["20000".to_string()]);
}

#[test]
fn runs_string_assignment_print() {
    let output = run_source("🐶 🟰 💬 hello 💬 🔚 📢 🐶 🔚").expect("source should run");
    assert_eq!(output, vec![" hello ".to_string()]);
}

#[test]
fn runs_string_concatenation() {
    let output = run_source("📢 💬 hello 💬 ➕ 💬 world 💬 🔚").expect("source should run");
    assert_eq!(output, vec![" hello  world ".to_string()]);
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
fn runs_list_ops() {
    let output =
        run_source("🍎 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚 🔚 📢 📏 🍎 🔚 📢 🍎 🔎 1️⃣ 🔚 🐶 🟰 🍎 📥 4️⃣ 🔚 📢 🐶 🔚")
            .expect("source should run");
    assert_eq!(
        output,
        vec![
            "3".to_string(),
            "2".to_string(),
            "📚 1 2 3 4 📚".to_string()
        ]
    );
}

#[test]
fn runs_append_statement_and_list_extend() {
    let output = run_source(
        "🍎 🟰 📚 1️⃣ 2️⃣ 📚 🔚 🍎 📥 3️⃣ 🔚 🍎 📥 📚 4️⃣ 5️⃣ 📚 🔚 🍎 📥 6️⃣3️⃣ 1️⃣ 2️⃣ 🔚 📢 🍎 🔚",
    )
    .expect("source should run");
    assert_eq!(output, vec!["📚 1 2 3 4 5 63 1 2 📚".to_string()]);
}

#[test]
fn runs_append_expression_with_implicit_list_payload() {
    let output = run_source("🍎 🟰 📚 1️⃣ 2️⃣ 📚 🔚 🥝 🟰 🍎 📥 6️⃣3️⃣ 1️⃣ 2️⃣ 🔚 📢 🥝 🔚")
        .expect("source should run");
    assert_eq!(output, vec!["📚 1 2 63 1 2 📚".to_string()]);
}

#[test]
fn runs_for_each_list_loop() {
    let output = run_source("🍎 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚 🔚 🔁 🐾 🧭 🍎 🔓 📢 🐾 🔚 🔒")
        .expect("source should run");
    assert_eq!(
        output,
        vec!["1".to_string(), "2".to_string(), "3".to_string()]
    );
}

#[test]
fn runs_range_loop() {
    let output = run_source("🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓 📢 🐾 🔚 🔒").expect("source should run");
    assert_eq!(
        output,
        vec!["0".to_string(), "1".to_string(), "2".to_string()]
    );
}

#[test]
fn recreates_loop_local_values_on_each_iteration_without_leaking() {
    let output = run_source("🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓 🐶 🟰 🐾 🔚 📢 🐶 🔚 🔒 📢 🐶 🔚")
        .expect("source should run");
    assert_eq!(
        output,
        vec![
            "0".to_string(),
            "1".to_string(),
            "2".to_string(),
            "🐶".to_string()
        ]
    );
}

#[test]
fn while_blocks_can_update_their_condition_binding() {
    let output = run_source("🐶 🟰 0️⃣ 🔚 🔁 🐶 ◀️ 3️⃣ 🔓 🐶 🟰 🐶 ➕ 1️⃣ 🔚 🔒 📢 🐶 🔚")
        .expect("source should run");
    assert_eq!(output, vec!["3".to_string()]);
}

#[test]
fn block_list_updates_apply_to_the_visible_outer_binding() {
    let output = run_source("🍎 🟰 📚 1️⃣ 2️⃣ 📚 🔚 🤔 ✅ 🔓 🍎 📥 3️⃣ 🔚 🔒 📢 🍎 🔚")
        .expect("source should run");
    assert_eq!(output, vec!["📚 1 2 3 📚".to_string()]);
}

#[test]
fn scoped_locals_work_with_break_and_continue() {
    let output = run_source(
        "🔁 ✅ 🔓 🐶 🟰 1️⃣ 🔚 🛑 🔚 🔒 📢 🐶 🔚 🔁 🐾 🧭 🔢 0️⃣ ➡️ 2️⃣ 🔓 🐱 🟰 🐾 🔚 ⏭️ 🔚 🔒 📢 🐱 🔚",
    )
    .expect("source should run");
    assert_eq!(output, vec!["🐶".to_string(), "🐱".to_string()]);
}

#[test]
fn runs_break_in_loop() {
    let output = run_source("🔁 ✅ 🔓 🛑 🔚 📢 1️⃣ 🔚 🔒").expect("source should run");
    assert!(output.is_empty());
}

#[test]
fn runs_continue_in_loop() {
    let output =
        run_source("🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓 ⏭️ 🔚 📢 🐾 🔚 🔒").expect("source should run");
    assert!(output.is_empty());
}

#[test]
fn descending_range_is_empty() {
    let output = run_source("🔁 🐾 🧭 🔢 3️⃣ ➡️ 0️⃣ 🔓 📢 🐾 🔚 🔒").expect("source should run");
    assert!(output.is_empty());
}

#[test]
fn reports_division_by_zero() {
    let error =
        run_source("🐶 🟰 1️⃣ ➗ 0️⃣ 🔚 📢 🐶 🔚").expect_err("source should fail at runtime");
    assert!(error.diagnostics[0].message.contains("division by zero"));
}

#[test]
fn reports_list_index_out_of_bounds() {
    let error = run_source("🍎 🟰 📚 1️⃣ 2️⃣ 📚 🔚 📢 🍎 🔎 2️⃣ 🔚")
        .expect_err("source should fail at runtime");
    assert!(error.diagnostics[0].message.contains("out of bounds"));
}

#[test]
fn stops_non_terminating_while_loop() {
    let bytecode =
        peps::compile_source("🐶 🟰 ✅ 🔚 🔁 🐶 🔓 📢 🐶 🔚 🔒").expect("source should compile");
    let error =
        vm::execute_with_step_limit(&bytecode, 12).expect_err("source should stop at runtime");

    assert!(!error.output.is_empty());
    assert!(error.diagnostics[0].message.contains("step limit"));
}

#[test]
fn enforces_step_limit_as_a_backup() {
    let bytecode = vec![Instruction::LoadConst(Value::Num(BigInt::from(1))); 2];
    let error: RunError =
        vm::execute_with_step_limit(&bytecode, 1).expect_err("step limit should stop execution");

    assert!(error.diagnostics[0].message.contains("step limit"));
}

#[test]
fn runs_functions_nested_calls_and_discards_results() {
    let output = run_source(
        "🧩 🧮 📚 🐶 🐱 📚 🔓 ↩️ 🐶 ➕ 🐱 🔚 🔒 📞 🧮 📚 3️⃣ 4️⃣ 📚 🔚 📢 📞 🧮 📚 1️⃣ 📞 🧮 📚 2️⃣ 3️⃣ 📚 📚 🔚",
    )
    .expect("calls should run");
    assert_eq!(output, vec!["6".to_string()]);
}

#[test]
fn runs_recursive_function_with_isolated_frames() {
    let output = run_source(
        "🧩 🌀 📚 🐶 📚 🔓 🤔 🐶 ◀️🟰 1️⃣ 🔓 ↩️ 1️⃣ 🔚 🔒 😐 🔓 🐱 🟰 🐶 ➖ 1️⃣ 🔚 ↩️ 🐶 ✖️ 📞 🌀 📚 🐱 📚 🔚 🔒 🔒 📢 📞 🌀 📚 5️⃣ 📚 🔚",
    )
    .expect("recursion should run");
    assert_eq!(output, vec!["120".to_string()]);
}

#[test]
fn functions_mutate_globals_without_reading_caller_locals() {
    let output = run_source(
        "🐶 🟰 1️⃣ 🔚 🧩 🧮 📚 📚 🔓 🐶 🟰 🐶 ➕ 1️⃣ 🔚 ↩️ 🦊 🔚 🔒 🤔 ✅ 🔓 🦊 🟰 9️⃣ 🔚 📞 🧮 📚 📚 🔚 🔒 📢 🐶 🔚",
    )
    .expect("global mutation should run");
    assert_eq!(output, vec!["2".to_string()]);
}

#[test]
fn dynamic_parameter_type_errors_are_reported_at_runtime() {
    let error = run_source("🧩 🧮 📚 🐶 📚 🔓 ↩️ 🐶 ➕ 1️⃣ 🔚 🔒 📞 🧮 📚 ✅ 📚 🔚")
        .expect_err("dynamic type mismatch should fail at runtime");
    assert!(error.diagnostics[0].message.contains("add requires"));
}
