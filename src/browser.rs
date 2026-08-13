//! Shared response model for browser-hosted Peps execution.

use serde::Serialize;

use crate::{
    diagnostic::Diagnostic,
    run_source_with_inputs_and_step_limit,
    vm::{IDE_STEP_LIMIT, INPUT_REQUIRED_PREFIX},
};

/// JSON-compatible result returned to the browser IDE.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RunResponse {
    /// Whether compilation and execution completed successfully.
    pub ok: bool,
    /// Lines printed by the program before success or failure.
    pub output: Vec<String>,
    /// Compiler or runtime diagnostics formatted for the IDE.
    pub diagnostics: Vec<IdeDiagnostic>,
    /// Input type requested when the queued terminal input has been exhausted.
    #[serde(rename = "inputRequest")]
    pub input_request: Option<String>,
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
pub fn run_source_for_browser(source: &str, inputs: &[String]) -> RunResponse {
    match run_source_with_inputs_and_step_limit(source, inputs.iter().cloned(), IDE_STEP_LIMIT) {
        Ok(output) => RunResponse {
            ok: true,
            output,
            diagnostics: Vec::new(),
            input_request: None,
        },
        Err(error) => {
            let input_request = error
                .diagnostics
                .first()
                .and_then(|diagnostic| diagnostic.message.strip_prefix(INPUT_REQUIRED_PREFIX))
                .map(str::to_string);
            RunResponse {
                ok: false,
                output: error.output,
                diagnostics: if input_request.is_some() {
                    Vec::new()
                } else {
                    error.diagnostics.iter().map(IdeDiagnostic::from).collect()
                },
                input_request,
            }
        }
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
