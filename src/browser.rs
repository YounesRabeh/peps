//! Shared response model for browser-hosted Peps execution.

use serde::Serialize;

use crate::{diagnostic::Diagnostic, run_source_with_step_limit, vm::IDE_STEP_LIMIT};

/// JSON-compatible result returned to the browser IDE.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RunResponse {
    /// Whether compilation and execution completed successfully.
    pub ok: bool,
    /// Lines printed by the program before success or failure.
    pub output: Vec<String>,
    /// Compiler or runtime diagnostics formatted for the IDE.
    pub diagnostics: Vec<IdeDiagnostic>,
}

/// Diagnostic shape consumed by the browser IDE.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct IdeDiagnostic {
    /// Human-readable diagnostic message.
    pub message: String,
    /// One-based source line, when a source span is available.
    pub line: Option<usize>,
    /// One-based source column, when a source span is available.
    pub column: Option<usize>,
    /// Byte offset where the diagnostic span starts.
    pub start: Option<usize>,
    /// Byte offset where the diagnostic span ends.
    pub end: Option<usize>,
}

/// Compile and run browser-submitted source with the IDE safety limit.
pub fn run_source_for_browser(source: &str) -> RunResponse {
    match run_source_with_step_limit(source, IDE_STEP_LIMIT) {
        Ok(output) => RunResponse {
            ok: true,
            output,
            diagnostics: Vec::new(),
        },
        Err(error) => RunResponse {
            ok: false,
            output: error.output,
            diagnostics: error.diagnostics.iter().map(IdeDiagnostic::from).collect(),
        },
    }
}

impl From<&Diagnostic> for IdeDiagnostic {
    fn from(diagnostic: &Diagnostic) -> Self {
        Self {
            message: diagnostic.message.clone(),
            line: diagnostic.span.map(|span| span.line),
            column: diagnostic.span.map(|span| span.column),
            start: diagnostic.span.map(|span| span.start),
            end: diagnostic.span.map(|span| span.end),
        }
    }
}
