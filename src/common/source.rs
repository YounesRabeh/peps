//! Source spans used to attach byte and line/column locations to compiler data.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    /// Creates a new source span.
    ///
    /// # Arguments
    ///
    /// * `start` - Byte offset where the span starts.
    /// * `end` - Byte offset where the span ends.
    /// * `line` - Source line where the span starts.
    /// * `column` - Source column where the span starts.
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// Returns a span covering both this span and another span.
    ///
    /// The merged span uses the smallest start offset and the largest end offset.
    /// It keeps the line and column position of `self`.
    pub fn merge(self, other: Span) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
            column: self.column,
        }
    }
}
