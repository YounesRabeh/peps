//! Human-readable compiler and runtime diagnostics.

use crate::source::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub fn at(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }

    pub fn format(&self, path: Option<&str>) -> String {
        let mut output = format!("error: {}", self.message);
        if let Some(span) = self.span {
            let file = path.unwrap_or("<source>");
            output.push_str(&format!("\n --> {}:{}:{}", file, span.line, span.column));
        }
        output
    }
}
