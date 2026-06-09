use peps::{compile_source, Instruction, Value};

#[test]
fn compiles_assignment() {
    assert_eq!(
        compile_source("🐶 🟰 5️⃣ 🔚").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(5)),
            Instruction::StoreVar("🐶".to_string()),
        ]
    );
}

#[test]
fn compiles_emoji_literal_assignment() {
    assert_eq!(
        compile_source("📦 🟰 🥊 🔚").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Emoji("🥊".to_string())),
            Instruction::StoreVar("📦".to_string()),
        ]
    );
}

#[test]
fn compiles_print() {
    assert_eq!(
        compile_source("🐶 🟰 5️⃣ 🔚 📢 🐶 🔚").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(5)),
            Instruction::StoreVar("🐶".to_string()),
            Instruction::LoadVar("🐶".to_string()),
            Instruction::Print,
        ]
    );
}

#[test]
fn compiles_newline_separated_statements_without_statement_end_token() {
    assert_eq!(
        compile_source("🐶 🟰 5️⃣\n📢 🐶").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(5)),
            Instruction::StoreVar("🐶".to_string()),
            Instruction::LoadVar("🐶".to_string()),
            Instruction::Print,
        ]
    );
}

#[test]
fn compiles_print_undeclared_emoji_literal() {
    assert_eq!(
        compile_source("📢 ✈️ 🔚").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Emoji("✈️".to_string())),
            Instruction::Print,
        ]
    );
}

#[test]
fn compiles_arithmetic_precedence() {
    assert_eq!(
        compile_source("🐶 🟰 1️⃣ ➕ 2️⃣ ✖️ 3️⃣ 🔚").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(1)),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::LoadConst(Value::Num(3)),
            Instruction::Mul,
            Instruction::Add,
            Instruction::StoreVar("🐶".to_string()),
        ]
    );
}

#[test]
fn compiles_string_concatenation() {
    assert_eq!(
        compile_source("📝 🟰 💬 hello 💬 ➕ 💬 world 💬 🔚").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Str(" hello ".to_string())),
            Instruction::LoadConst(Value::Str(" world ".to_string())),
            Instruction::Add,
            Instruction::StoreVar("📝".to_string()),
        ]
    );
}

#[test]
fn compiles_comparison() {
    assert_eq!(
        compile_source("🐶 🟰 5️⃣ ▶️ 3️⃣ 🔚").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(5)),
            Instruction::LoadConst(Value::Num(3)),
            Instruction::Gt,
            Instruction::StoreVar("🐶".to_string()),
        ]
    );
}

#[test]
fn compiles_logical_not() {
    assert_eq!(
        compile_source("🐶 🟰 🚫 ✅ 🔚").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Bool(true)),
            Instruction::JumpIfFalse(4),
            Instruction::LoadConst(Value::Bool(false)),
            Instruction::Jump(5),
            Instruction::LoadConst(Value::Bool(true)),
            Instruction::StoreVar("🐶".to_string()),
        ]
    );
}

#[test]
fn compiles_list_length() {
    assert_eq!(
        compile_source("🐶 🟰 📏 📚 1️⃣ 2️⃣ 📚 🔚").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(1)),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::MakeList(2),
            Instruction::ListLen,
            Instruction::StoreVar("🐶".to_string()),
        ]
    );
}

#[test]
fn compiles_list_index_and_append() {
    assert_eq!(
        compile_source("🐶 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚 🔎 1️⃣ 🔚 🐱 🟰 📚 1️⃣ 2️⃣ 📚 📥 3️⃣ 🔚")
            .expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(1)),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::LoadConst(Value::Num(3)),
            Instruction::MakeList(3),
            Instruction::LoadConst(Value::Num(1)),
            Instruction::ListGet,
            Instruction::StoreVar("🐶".to_string()),
            Instruction::LoadConst(Value::Num(1)),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::MakeList(2),
            Instruction::LoadConst(Value::Num(3)),
            Instruction::ListAppend,
            Instruction::StoreVar("🐱".to_string()),
        ]
    );
}

#[test]
fn compiles_append_statement() {
    assert_eq!(
        compile_source("🍎 🟰 📚 1️⃣ 2️⃣ 📚 🔚 🍎 📥 6️⃣3️⃣ 1️⃣ 2️⃣ 🔚")
            .expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(1)),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::MakeList(2),
            Instruction::StoreVar("🍎".to_string()),
            Instruction::LoadVar("🍎".to_string()),
            Instruction::LoadConst(Value::Num(63)),
            Instruction::LoadConst(Value::Num(1)),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::MakeList(3),
            Instruction::ListAppend,
            Instruction::StoreVar("🍎".to_string()),
        ]
    );
}

#[test]
fn compiles_append_expression_with_implicit_list_payload() {
    assert_eq!(
        compile_source("🍎 🟰 📚 1️⃣ 2️⃣ 📚 🔚 🥝 🟰 🍎 📥 6️⃣3️⃣ 1️⃣ 2️⃣ 🔚")
            .expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(1)),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::MakeList(2),
            Instruction::StoreVar("🍎".to_string()),
            Instruction::LoadVar("🍎".to_string()),
            Instruction::LoadConst(Value::Num(63)),
            Instruction::LoadConst(Value::Num(1)),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::MakeList(3),
            Instruction::ListAppend,
            Instruction::StoreVar("🥝".to_string()),
        ]
    );
}

#[test]
fn compiles_list_construction() {
    assert_eq!(
        compile_source("🐶 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚 🔚").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(1)),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::LoadConst(Value::Num(3)),
            Instruction::MakeList(3),
            Instruction::StoreVar("🐶".to_string()),
        ]
    );
}

#[test]
fn compiles_if() {
    assert_eq!(
        compile_source("🐶 🟰 ✅ 🔚 🤔 🐶 🔓 📢 1️⃣ 🔚 🔒").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Bool(true)),
            Instruction::StoreVar("🐶".to_string()),
            Instruction::LoadVar("🐶".to_string()),
            Instruction::JumpIfFalse(6),
            Instruction::LoadConst(Value::Num(1)),
            Instruction::Print,
        ]
    );
}

#[test]
fn compiles_if_else() {
    assert_eq!(
        compile_source("🐶 🟰 ✅ 🔚 🤔 🐶 🔓 📢 1️⃣ 🔚 🔒 😐 🔓 📢 2️⃣ 🔚 🔒")
            .expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Bool(true)),
            Instruction::StoreVar("🐶".to_string()),
            Instruction::LoadVar("🐶".to_string()),
            Instruction::JumpIfFalse(7),
            Instruction::LoadConst(Value::Num(1)),
            Instruction::Print,
            Instruction::Jump(9),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::Print,
        ]
    );
}

#[test]
fn compiles_while() {
    assert_eq!(
        compile_source("🐶 🟰 ✅ 🔚 🔁 🐶 🔓 📢 🐶 🔚 🔒").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Bool(true)),
            Instruction::StoreVar("🐶".to_string()),
            Instruction::LoadVar("🐶".to_string()),
            Instruction::JumpIfFalse(7),
            Instruction::LoadVar("🐶".to_string()),
            Instruction::Print,
            Instruction::Jump(2),
        ]
    );
}

#[test]
fn compiles_while_with_break() {
    assert_eq!(
        compile_source("🔁 ✅ 🔓 🛑 🔚 🔒").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Bool(true)),
            Instruction::JumpIfFalse(4),
            Instruction::Jump(4),
            Instruction::Jump(0),
        ]
    );
}

#[test]
fn compiles_while_with_continue() {
    assert_eq!(
        compile_source("🔁 ✅ 🔓 ⏭️ 🔚 📢 1️⃣ 🔚 🔒").expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Bool(true)),
            Instruction::JumpIfFalse(6),
            Instruction::Jump(0),
            Instruction::LoadConst(Value::Num(1)),
            Instruction::Print,
            Instruction::Jump(0),
        ]
    );
}

#[test]
fn compiles_for_each_list() {
    assert_eq!(
        compile_source("🍎 🟰 📚 1️⃣ 2️⃣ 📚 🔚 🔁 🐾 🧭 🍎 🔓 📢 🐾 🔚 🔒")
            .expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(1)),
            Instruction::LoadConst(Value::Num(2)),
            Instruction::MakeList(2),
            Instruction::StoreVar("🍎".to_string()),
            Instruction::LoadVar("🍎".to_string()),
            Instruction::StoreVar("__peps_for_0_list".to_string()),
            Instruction::LoadConst(Value::Num(0)),
            Instruction::StoreVar("__peps_for_0_index".to_string()),
            Instruction::LoadVar("__peps_for_0_list".to_string()),
            Instruction::ListLen,
            Instruction::StoreVar("__peps_for_0_len".to_string()),
            Instruction::LoadVar("__peps_for_0_index".to_string()),
            Instruction::LoadVar("__peps_for_0_len".to_string()),
            Instruction::Lt,
            Instruction::JumpIfFalse(26),
            Instruction::LoadVar("__peps_for_0_list".to_string()),
            Instruction::LoadVar("__peps_for_0_index".to_string()),
            Instruction::ListGet,
            Instruction::StoreVar("🐾".to_string()),
            Instruction::LoadVar("🐾".to_string()),
            Instruction::Print,
            Instruction::LoadVar("__peps_for_0_index".to_string()),
            Instruction::LoadConst(Value::Num(1)),
            Instruction::Add,
            Instruction::StoreVar("__peps_for_0_index".to_string()),
            Instruction::Jump(11),
        ]
    );
}

#[test]
fn compiles_for_range() {
    assert_eq!(
        compile_source("🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓 📢 🐾 🔚 🔒")
            .expect("source should compile"),
        vec![
            Instruction::LoadConst(Value::Num(0)),
            Instruction::StoreVar("__peps_for_0_index".to_string()),
            Instruction::LoadConst(Value::Num(3)),
            Instruction::StoreVar("__peps_for_0_end".to_string()),
            Instruction::LoadVar("__peps_for_0_index".to_string()),
            Instruction::LoadVar("__peps_for_0_end".to_string()),
            Instruction::Lt,
            Instruction::JumpIfFalse(17),
            Instruction::LoadVar("__peps_for_0_index".to_string()),
            Instruction::StoreVar("🐾".to_string()),
            Instruction::LoadVar("🐾".to_string()),
            Instruction::Print,
            Instruction::LoadVar("__peps_for_0_index".to_string()),
            Instruction::LoadConst(Value::Num(1)),
            Instruction::Add,
            Instruction::StoreVar("__peps_for_0_index".to_string()),
            Instruction::Jump(4),
        ]
    );
}
