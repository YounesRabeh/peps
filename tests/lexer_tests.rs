use num_bigint::BigInt;

use peps::{lexer, TokenKind};

fn kinds(source: &str) -> Vec<TokenKind> {
    lexer::lex(source)
        .expect("source should lex")
        .into_iter()
        .map(|token| token.kind)
        .collect()
}

#[test]
fn lexes_emoji_number() {
    assert_eq!(
        kinds("🐶 🟰 5️⃣ 🔚"),
        vec![
            TokenKind::Identifier("🐶".to_string()),
            TokenKind::Assign,
            TokenKind::Number(BigInt::from(5)),
            TokenKind::StatementEnd,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_multi_digit_emoji_number() {
    assert_eq!(
        kinds("🐶 🟰 1️⃣2️⃣3️⃣ 🔚")[2],
        TokenKind::Number(BigInt::from(123))
    );
}

#[test]
fn lexes_float_with_emoji_digits() {
    assert_eq!(
        kinds("🐶 🟰 1️⃣2️⃣.3️⃣4️⃣ 🔚"),
        vec![
            TokenKind::Identifier("🐶".to_string()),
            TokenKind::Assign,
            TokenKind::Float(12.34),
            TokenKind::StatementEnd,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn rejects_ascii_digits_in_float_literals() {
    let diagnostics = lexer::lex("1.5").expect_err("ASCII digits should fail");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("ASCII digits")));
}

#[test]
fn lexes_integer_larger_than_i64() {
    assert_eq!(
        kinds("9️⃣2️⃣2️⃣3️⃣3️⃣7️⃣2️⃣0️⃣3️⃣6️⃣8️⃣5️⃣4️⃣7️⃣7️⃣5️⃣8️⃣0️⃣8️⃣")[0],
        TokenKind::Number(BigInt::parse_bytes(b"9223372036854775808", 10).expect("valid integer"))
    );
}

#[test]
fn lexes_minus_as_operator() {
    assert_eq!(kinds("🐶 🟰 ➖5️⃣ 🔚")[2], TokenKind::Minus);
}

#[test]
fn lexes_string_literal() {
    assert_eq!(
        kinds("🐶 🟰 💬 hello world 💬 🔚")[2],
        TokenKind::StringLiteral(" hello world ".to_string())
    );
}

#[test]
fn rejects_ascii_digit_outside_string() {
    let diagnostics = lexer::lex("🐶 🟰 5 🔚").expect_err("ASCII digit should fail");
    assert!(diagnostics[0].message.contains("ASCII digits"));
}

#[test]
fn rejects_ascii_operator() {
    let diagnostics = lexer::lex("🐶 = 5️⃣ 🔚").expect_err("ASCII operator should fail");
    assert!(diagnostics[0].message.contains("invalid character"));
}

#[test]
fn rejects_normal_quote() {
    let diagnostics = lexer::lex("🐶 🟰 \"hello\" 🔚").expect_err("quote should fail");
    assert!(diagnostics[0].message.contains("invalid character"));
}

#[test]
fn lexes_longest_match_operators() {
    assert_eq!(
        kinds("🐶 🟰🟰 5️⃣ ❌🟰 4️⃣ ◀️🟰 6️⃣ ▶️🟰 3️⃣ 🔚"),
        vec![
            TokenKind::Identifier("🐶".to_string()),
            TokenKind::Eq,
            TokenKind::Number(BigInt::from(5)),
            TokenKind::NotEq,
            TokenKind::Number(BigInt::from(4)),
            TokenKind::LtEq,
            TokenKind::Number(BigInt::from(6)),
            TokenKind::GtEq,
            TokenKind::Number(BigInt::from(3)),
            TokenKind::StatementEnd,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_list_operators() {
    assert_eq!(
        kinds("📏 🍎 🔎 1️⃣ 📥 2️⃣"),
        vec![
            TokenKind::ListLen,
            TokenKind::Identifier("🍎".to_string()),
            TokenKind::ListIndex,
            TokenKind::Number(BigInt::from(1)),
            TokenKind::ListAppend,
            TokenKind::Number(BigInt::from(2)),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_for_loop_tokens() {
    assert_eq!(
        kinds("🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓 🔒"),
        vec![
            TokenKind::While,
            TokenKind::Identifier("🐾".to_string()),
            TokenKind::In,
            TokenKind::Range,
            TokenKind::Number(BigInt::from(0)),
            TokenKind::Arrow,
            TokenKind::Number(BigInt::from(3)),
            TokenKind::BlockStart,
            TokenKind::BlockEnd,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_function_tokens() {
    let tokens = kinds("🧩 🧮 📚 📚 🔓 ↩️ 📞 🧮 📚 📚 🔚 🔒");
    assert!(tokens.contains(&TokenKind::Function));
    assert!(tokens.contains(&TokenKind::Call));
    assert!(tokens.contains(&TokenKind::Return));
}

#[test]
fn lexes_typed_input_tokens() {
    assert_eq!(
        kinds("⌨️ 🔤 ⌨️ 🔢 ⌨️ 🔣 ⌨️ ☑️"),
        vec![
            TokenKind::Input,
            TokenKind::InputText,
            TokenKind::Input,
            TokenKind::Range,
            TokenKind::Input,
            TokenKind::InputFloat,
            TokenKind::Input,
            TokenKind::InputBool,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_explicit_conversion_tokens() {
    assert_eq!(
        kinds("🔄 🔢 📝 🔄 🔣 🐶"),
        vec![
            TokenKind::Convert,
            TokenKind::Range,
            TokenKind::Identifier("📝".to_string()),
            TokenKind::Convert,
            TokenKind::InputFloat,
            TokenKind::Identifier("🐶".to_string()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_map_tokens() {
    assert_eq!(
        kinds("🗺️ 💬name💬 ➡️ 💬Peps💬 🗺️"),
        vec![
            TokenKind::MapDelimiter,
            TokenKind::StringLiteral("name".to_string()),
            TokenKind::Arrow,
            TokenKind::StringLiteral("Peps".to_string()),
            TokenKind::MapDelimiter,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_map_key_existence_operator() {
    assert_eq!(
        kinds("🔑 📖 💬name💬"),
        vec![
            TokenKind::MapHas,
            TokenKind::Identifier("📖".to_string()),
            TokenKind::StringLiteral("name".to_string()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_constant_declaration() {
    assert_eq!(
        kinds("🔐 🐶 🟰 4️⃣2️⃣"),
        vec![
            TokenKind::Const,
            TokenKind::Identifier("🐶".to_string()),
            TokenKind::Assign,
            TokenKind::Number(BigInt::from(42)),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_emoji_variables() {
    assert_eq!(
        kinds("🚀 🟰 🌙 🔚"),
        vec![
            TokenKind::Identifier("🚀".to_string()),
            TokenKind::Assign,
            TokenKind::Identifier("🌙".to_string()),
            TokenKind::StatementEnd,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_ascii_variables() {
    assert_eq!(
        kinds("test_name 🟰 value2 🔚"),
        vec![
            TokenKind::Identifier("test_name".to_string()),
            TokenKind::Assign,
            TokenKind::Identifier("value2".to_string()),
            TokenKind::StatementEnd,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_break_and_continue_tokens() {
    assert_eq!(
        kinds("🛑 🔚 ⏭️ 🔚"),
        vec![
            TokenKind::Break,
            TokenKind::StatementEnd,
            TokenKind::Continue,
            TokenKind::StatementEnd,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_logical_operators() {
    assert_eq!(
        kinds("🤝 🔀 🚫"),
        vec![
            TokenKind::And,
            TokenKind::Or,
            TokenKind::Not,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_newline_as_statement_separator() {
    assert_eq!(
        kinds("🐶 🟰 1️⃣\n📢 🐶"),
        vec![
            TokenKind::Identifier("🐶".to_string()),
            TokenKind::Assign,
            TokenKind::Number(BigInt::from(1)),
            TokenKind::StatementEnd,
            TokenKind::Print,
            TokenKind::Identifier("🐶".to_string()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn skips_line_comments() {
    assert_eq!(
        kinds("🐶 🟰 1️⃣ // ignored\n📢 🐶"),
        vec![
            TokenKind::Identifier("🐶".to_string()),
            TokenKind::Assign,
            TokenKind::Number(BigInt::from(1)),
            TokenKind::StatementEnd,
            TokenKind::Print,
            TokenKind::Identifier("🐶".to_string()),
            TokenKind::Eof,
        ]
    );
}
