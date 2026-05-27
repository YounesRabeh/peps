use crate::source::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Number(i64),
    StringLiteral(String),
    Bool(bool),

    Print,
    If,
    Else,
    While,
    In,
    Range,

    Assign,
    Plus,
    Minus,
    Star,
    Slash,

    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Arrow,

    BlockStart,
    BlockEnd,
    StatementEnd,
    ListDelimiter,

    Eof,
}
