//! Static type definitions used by semantic analysis.

/// Static types supported by the Peps v0 semantic checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Numeric values.
    Num,
    /// Text values.
    Str,
    /// Boolean values.
    Bool,
    /// Emoji literal values.
    Emoji,
    /// Homogeneous list values.
    List(Box<Type>),
}
