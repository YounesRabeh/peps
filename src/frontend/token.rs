//! Token types emitted by the emoji-aware lexer.

use crate::source::Span;

/// A lexical token with its original source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The token category and any literal payload.
    pub kind: TokenKind,
    /// Source span covered by this token.
    pub span: Span,
}

impl Token {
    /// Create a token from its kind and source span.
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Token categories recognized by the Peps lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// One user-visible identifier grapheme.
    Identifier(String),
    /// Numeric literal.
    Number(i64),
    /// Text literal.
    StringLiteral(String),
    /// Boolean literal.
    Bool(bool),

    /// Print statement keyword.
    Print,
    /// If statement keyword.
    If,
    /// Else branch keyword.
    Else,
    /// While loop keyword.
    While,
    /// Break statement keyword.
    Break,
    /// Continue statement keyword.
    Continue,
    /// For-each membership keyword.
    In,
    /// Numeric range keyword.
    Range,
    /// Logical and operator.
    And,
    /// Logical or operator.
    Or,
    /// Logical not operator.
    Not,
    /// List length operator.
    ListLen,
    /// List indexing operator.
    ListIndex,
    /// List append operator.
    ListAppend,

    /// Assignment operator.
    Assign,
    /// Addition operator.
    Plus,
    /// Subtraction or numeric negation operator.
    Minus,
    /// Multiplication operator.
    Star,
    /// Division operator.
    Slash,

    /// Equality comparison operator.
    Eq,
    /// Inequality comparison operator.
    NotEq,
    /// Less-than comparison operator.
    Lt,
    /// Greater-than comparison operator.
    Gt,
    /// Less-than-or-equal comparison operator.
    LtEq,
    /// Greater-than-or-equal comparison operator.
    GtEq,
    /// For-loop arrow separator.
    Arrow,

    /// Block opening delimiter.
    BlockStart,
    /// Block closing delimiter.
    BlockEnd,
    /// Statement separator, produced from explicit separators or newlines.
    StatementEnd,
    /// List element delimiter.
    ListDelimiter,

    /// End-of-file marker appended by the lexer.
    Eof,
}
