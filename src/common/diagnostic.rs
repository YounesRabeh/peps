//! Human-readable compiler and runtime diagnostics.

use crate::source::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    /// Creates a diagnostic without a source span.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    /// Creates a diagnostic attached to a source span.
    pub fn at(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }

    /// Formats the diagnostic as a human-readable error message.
    ///
    /// If a span is available, the output includes the source path, line, and column.
    /// When no path is provided, `"<source>"` is used.
    pub fn format(&self, path: Option<&str>) -> String {
        let mut output = format!("error: {}", self.message);
        if let Some(span) = self.span {
            let file = path.unwrap_or("<source>");
            output.push_str(&format!("\n --> {}:{}:{}", file, span.line, span.column));
        }
        output
    }
}
