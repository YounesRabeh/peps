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
fn lexes_integer_larger_than_i64() {
    assert_eq!(
        kinds("9️⃣2️⃣2️⃣3️⃣3️⃣7️⃣2️⃣0️⃣3️⃣6️⃣8️⃣5️⃣4️⃣7️⃣7️⃣5️⃣8️⃣0️⃣8️⃣")[0],
        TokenKind::Number(
            BigInt::parse_bytes(b"9223372036854775808", 10).expect("valid integer")
        )
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
