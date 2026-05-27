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
