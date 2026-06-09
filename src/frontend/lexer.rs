//! Emoji-aware lexer for Peps source text.
//!
//! Peps syntax is built from emoji tokens, including emoji that are encoded as
//! multiple Unicode scalar values. The lexer therefore walks the input by
//! Unicode grapheme cluster instead of by byte or scalar value. Each emitted
//! [`Token`] carries a [`Span`] with byte offsets and human-oriented line and
//! column positions.
//!
//! Newlines and explicit `🔚` tokens both become [`TokenKind::StatementEnd`].
//! Line comments starting with `//` are skipped until the end of the line.
//! Consecutive separators are collapsed by newline handling so the parser can
//! accept blank lines without seeing empty statements.

use unicode_segmentation::UnicodeSegmentation;

use crate::{
    diagnostic::Diagnostic,
    source::Span,
    token::{Token, TokenKind},
};

#[derive(Debug, Clone)]
struct Grapheme {
    text: String,
    span: Span,
}

/// Tokenize source text into a stream ending with [`TokenKind::Eof`].
///
/// The lexer reports all invalid characters it sees in one pass. ASCII
/// identifiers are accepted here so the parser can issue the language-specific
/// "exactly one emoji" diagnostic at identifier positions.
pub fn lex(source: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let graphemes = collect_graphemes(source);
    let mut lexer = Lexer {
        graphemes,
        current: 0,
        diagnostics: Vec::new(),
        source_len: source.len(),
        eof_line: 1,
        eof_column: 1,
    };
    lexer.set_eof_position();
    lexer.lex_tokens()
}

/// Collect source text into grapheme clusters with source positions.
///
/// A single Peps-visible character may contain variation selectors or combining
/// marks, so column numbers advance by grapheme cluster rather than byte length.
fn collect_graphemes(source: &str) -> Vec<Grapheme> {
    let mut line = 1;
    let mut column = 1;
    let mut out = Vec::new();

    for (start, text) in source.grapheme_indices(true) {
        let end = start + text.len();
        out.push(Grapheme {
            text: text.to_string(),
            span: Span::new(start, end, line, column),
        });

        if text == "\n" || text == "\r" || text == "\r\n" {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    out
}

struct Lexer {
    graphemes: Vec<Grapheme>,
    current: usize,
    diagnostics: Vec<Diagnostic>,
    source_len: usize,
    eof_line: usize,
    eof_column: usize,
}

impl Lexer {
    /// Record the line and column where the synthetic EOF token should point.
    fn set_eof_position(&mut self) {
        if let Some(last) = self.graphemes.last() {
            if last.text == "\n" || last.text == "\r" || last.text == "\r\n" {
                self.eof_line = last.span.line + 1;
                self.eof_column = 1;
            } else {
                self.eof_line = last.span.line;
                self.eof_column = last.span.column + 1;
            }
        }
    }

    fn lex_tokens(&mut self) -> Result<Vec<Token>, Vec<Diagnostic>> {
        let mut tokens: Vec<Token> = Vec::new();

        while !self.is_at_end() {
            let grapheme = self.peek().clone();

            if is_newline(&grapheme.text) {
                let span = grapheme.span;
                self.advance();
                if should_emit_statement_separator(tokens.last().map(|token| &token.kind)) {
                    tokens.push(Token::new(TokenKind::StatementEnd, span));
                }
                continue;
            }

            if is_whitespace(&grapheme.text) {
                self.advance();
                continue;
            }

            if self.is_line_comment_start() {
                self.skip_line_comment();
                continue;
            }

            if grapheme.text == "💬" {
                if let Some(token) = self.lex_string() {
                    tokens.push(token);
                }
                continue;
            }

            if let Some(token) = self.lex_long_operator() {
                tokens.push(token);
                continue;
            }

            if is_emoji_digit(&grapheme.text) {
                tokens.push(self.lex_number());
                continue;
            }

            if is_ascii_identifier_start(&grapheme.text) {
                tokens.push(self.lex_ascii_identifier());
                continue;
            }

            if let Some(kind) = single_token_kind(&grapheme.text) {
                let span = grapheme.span;
                self.advance();
                tokens.push(Token::new(kind, span));
                continue;
            }

            if is_identifier_emoji(&grapheme.text) {
                let span = grapheme.span;
                let name = grapheme.text;
                self.advance();
                tokens.push(Token::new(TokenKind::Identifier(name), span));
                continue;
            }

            self.report_invalid_character(&grapheme);
            self.advance();
        }

        let eof_span = Span::new(self.source_len, self.source_len, self.eof_line, self.eof_column);
        tokens.push(Token::new(TokenKind::Eof, eof_span));

        if self.diagnostics.is_empty() {
            Ok(tokens)
        } else {
            Err(std::mem::take(&mut self.diagnostics))
        }
    }

    fn lex_string(&mut self) -> Option<Token> {
        let start = self.advance().clone();
        let mut content = String::new();
        let mut end_span = start.span;

        while !self.is_at_end() {
            let grapheme = self.advance().clone();
            if grapheme.text == "💬" {
                end_span = grapheme.span;
                let span = start.span.merge(end_span);
                return Some(Token::new(TokenKind::StringLiteral(content), span));
            }
            content.push_str(&grapheme.text);
            end_span = grapheme.span;
        }

        let span = start.span.merge(end_span);
        self.diagnostics
            .push(Diagnostic::at("unterminated string literal", span));
        None
    }

    fn lex_long_operator(&mut self) -> Option<Token> {
        let first = self.peek();
        let second = self.peek_next()?;
        let kind = match (first.text.as_str(), second.text.as_str()) {
            ("🟰", "🟰") => TokenKind::Eq,
            ("❌", "🟰") => TokenKind::NotEq,
            ("◀️", "🟰") => TokenKind::LtEq,
            ("▶️", "🟰") => TokenKind::GtEq,
            _ => return None,
        };

        let span = first.span.merge(second.span);
        self.advance();
        self.advance();
        Some(Token::new(kind, span))
    }

    fn lex_number(&mut self) -> Token {
        let start = self.peek().span;
        let mut value: i64 = 0;
        let mut end = start;

        while !self.is_at_end() && is_emoji_digit(&self.peek().text) {
            let grapheme = self.advance().clone();
            end = grapheme.span;
            let digit = emoji_digit_value(&grapheme.text).expect("checked by is_emoji_digit");
            value = value.saturating_mul(10).saturating_add(digit);
        }

        Token::new(TokenKind::Number(value), start.merge(end))
    }

fn lex_ascii_identifier(&mut self) -> Token {
        let start = self.peek().span;
        let mut end = start;
        let mut name = String::new();

        while !self.is_at_end() && is_ascii_identifier_continue(&self.peek().text) {
            let grapheme = self.advance().clone();
            end = grapheme.span;
            name.push_str(&grapheme.text);
        }

        let span = start.merge(end);
        if let Some(kind) = keyword_kind(&name) {
            Token::new(kind, span)
        } else {
            Token::new(TokenKind::Identifier(name), span)
        }
    }

    fn is_line_comment_start(&self) -> bool {
        matches!(
            (self.peek().text.as_str(), self.peek_next().map(|grapheme| grapheme.text.as_str())),
            ("/", Some("/"))
        )
    }

    fn skip_line_comment(&mut self) {
        self.advance();
        self.advance();

        while !self.is_at_end() && !is_newline(&self.peek().text) {
            self.advance();
        }
    }

    fn report_invalid_character(&mut self, grapheme: &Grapheme) {
        let message = if grapheme.text.chars().all(|ch| ch.is_ascii_digit()) {
            "ASCII digits are not allowed outside strings. Use emoji digits.".to_string()
        } else {
            format!(
                "invalid character outside string literal: {}",
                printable_grapheme(&grapheme.text)
            )
        };
        self.diagnostics.push(Diagnostic::at(message, grapheme.span));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.graphemes.len()
    }

    fn peek(&self) -> &Grapheme {
        &self.graphemes[self.current]
    }

    fn peek_next(&self) -> Option<&Grapheme> {
        self.graphemes.get(self.current + 1)
    }

    fn advance(&mut self) -> &Grapheme {
        let index = self.current;
        self.current += 1;
        &self.graphemes[index]
    }
}

fn is_whitespace(text: &str) -> bool {
    return matches!(text, " " | "\t");
}

fn is_newline(text: &str) -> bool {
    matches!(text, "\n" | "\r" | "\r\n")
}

fn should_emit_statement_separator(previous: Option<&TokenKind>) -> bool {
    !matches!(previous, None | Some(TokenKind::StatementEnd))
}

fn is_ascii_identifier_start(text: &str) -> bool {
    matches!(text.as_bytes(), [b'a'..=b'z'] | [b'A'..=b'Z'] | [b'_'])
}

fn is_ascii_identifier_continue(text: &str) -> bool {
    matches!(
        text.as_bytes(),
        [b'a'..=b'z'] | [b'A'..=b'Z'] | [b'0'..=b'9'] | [b'_']
    )
}

fn single_token_kind(text: &str) -> Option<TokenKind> {
    match text {
        "📢" => Some(TokenKind::Print),
        "🤔" => Some(TokenKind::If),
        "😐" => Some(TokenKind::Else),
        "🔁" => Some(TokenKind::While),
        "🛑" => Some(TokenKind::Break),
        "⏭️" | "⏭" => Some(TokenKind::Continue),
        "🧭" => Some(TokenKind::In),
        "🔢" => Some(TokenKind::Range),
        "✅" => Some(TokenKind::Bool(true)),
        "❌" => Some(TokenKind::Bool(false)),
        "🤝" => Some(TokenKind::And),
        "🔀" => Some(TokenKind::Or),
        "🚫" => Some(TokenKind::Not),
        "🟰" => Some(TokenKind::Assign),
        "➕" => Some(TokenKind::Plus),
        "➖" => Some(TokenKind::Minus),
        "✖️" => Some(TokenKind::Star),
        "➗" => Some(TokenKind::Slash),
        "◀️" => Some(TokenKind::Lt),
        "▶️" => Some(TokenKind::Gt),
        "➡️" => Some(TokenKind::Arrow),
        "🔓" => Some(TokenKind::BlockStart),
        "🔒" => Some(TokenKind::BlockEnd),
        "🔚" => Some(TokenKind::StatementEnd),
        "📚" => Some(TokenKind::ListDelimiter),
        _ => None,
    }
}

fn keyword_kind(text: &str) -> Option<TokenKind> {
    match text {
        "and" => Some(TokenKind::And),
        "or" => Some(TokenKind::Or),
        "not" => Some(TokenKind::Not),
        _ => None,
    }
}

fn is_reserved(text: &str) -> bool {
    return single_token_kind(text).is_some()
        || keyword_kind(text).is_some()
        || matches!(text, "💬")
        || is_emoji_digit(text);
}

fn is_identifier_emoji(text: &str) -> bool {
    if is_reserved(text) || text.chars().any(|ch| ch.is_ascii()) {
        return false;
    }

    return text.chars().any(is_emoji_like_scalar);
}

fn is_emoji_like_scalar(ch: char) -> bool {
    return matches!(
        ch as u32,
        0x1F000..=0x1FAFF
            | 0x2600..=0x27BF
            | 0x2300..=0x23FF
            | 0x2B00..=0x2BFF
    );
}

fn is_emoji_digit(text: &str) -> bool {
    emoji_digit_value(text).is_some()
}

fn emoji_digit_value(text: &str) -> Option<i64> {
    match text {
        "0️⃣" => Some(0),
        "1️⃣" => Some(1),
        "2️⃣" => Some(2),
        "3️⃣" => Some(3),
        "4️⃣" => Some(4),
        "5️⃣" => Some(5),
        "6️⃣" => Some(6),
        "7️⃣" => Some(7),
        "8️⃣" => Some(8),
        "9️⃣" => Some(9),
        _ => None,
    }
}

fn printable_grapheme(text: &str) -> String {
    match text {
        "\n" => "\\n".to_string(),
        "\r" => "\\r".to_string(),
        "\t" => "\\t".to_string(),
        other => other.to_string(),
    }
}
