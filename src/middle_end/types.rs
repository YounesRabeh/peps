//! Static type definitions used by semantic analysis.

/// Static types supported by the Peps v0 semantic checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// A value whose concrete type is only known when a function runs.
    Unknown,
    /// Arbitrary-precision integer values.
    Num,
    /// Floating-point values.
    Float,
    /// Text values.
    Str,
    /// Boolean values.
    Bool,
    /// Emoji literal values.
    Emoji,
    /// Homogeneous list values.
    List(Box<Type>),
    /// Ordered text-keyed map with homogeneous values.
    Map(Box<Type>),
}
