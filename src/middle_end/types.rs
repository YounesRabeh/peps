//! Static type definitions used by semantic analysis.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Num,
    Str,
    Bool,
    Emoji,
    List(Box<Type>),
}
