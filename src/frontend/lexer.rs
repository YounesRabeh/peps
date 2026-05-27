//! Emoji-aware lexer that tokenizes Peps source by Unicode grapheme cluster.

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
// normalize to resolve the invidsibale character issue in some emojis, e.g. "1️⃣" can be represented as "1\u{FE0F}\u{20E3}" 

//pub fn normalize_peps_source(source: &str) -> String {
//    source.replace('\u{FE0F}', "")
//}

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
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            let grapheme = self.peek().clone();

            if is_whitespace(&grapheme.text) {
                self.advance();
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
    return matches!(text, " " | "\t" | "\n" | "\r" | "\r\n");
}

fn single_token_kind(text: &str) -> Option<TokenKind> {
    match text {
        "📢" => Some(TokenKind::Print),
        "🤔" => Some(TokenKind::If),
        "😐" => Some(TokenKind::Else),
        "🔁" => Some(TokenKind::While),
        "🧭" => Some(TokenKind::In),
        "🔢" => Some(TokenKind::Range),
        "✅" => Some(TokenKind::Bool(true)),
        "❌" => Some(TokenKind::Bool(false)),
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

fn is_reserved(text: &str) -> bool {
    return single_token_kind(text).is_some()
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
