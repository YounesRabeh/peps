//! Public library API for the Peps compiler and bytecode runner.
//!
//! The crate is organized around compiler layers:
//! frontend parsing, middle-end semantic analysis, backend bytecode generation,
//! and runtime bytecode execution.

pub mod backend;
pub mod common;
pub mod frontend;
#[cfg(not(target_arch = "wasm32"))]
pub mod ide;
pub mod middle_end;
pub mod runtime;

mod browser;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use backend::{bytecode, compiler};
pub use common::{diagnostic, source};
pub use frontend::{ast, lexer, parser, token};
pub use middle_end::{semantic, symbol_table, types};
pub use runtime::vm;

pub use ast::{BinaryOp, ConversionKind, Expr, ForSource, InputKind, Program, Stmt, UnaryOp};
pub use bytecode::{Instruction, Value};
pub use diagnostic::Diagnostic;
pub use source::Span;
pub use token::{Token, TokenKind};
pub use types::Type;
pub use vm::{ExecutionLimit, RunError, RuntimeValue, IDE_STEP_LIMIT};

/// Compile Peps source text into bytecode instructions.
pub fn compile_source(source: &str) -> Result<Vec<Instruction>, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    let checked_program = semantic::check(program)?;
    compiler::compile(checked_program)
}

/// Compile and run Peps source text, returning the printed output lines.
pub fn run_source(source: &str) -> Result<Vec<String>, RunError> {
    let bytecode = compile_source(source).map_err(|diagnostics| RunError {
        output: Vec::new(),
        diagnostics,
    })?;
    vm::execute(&bytecode)
}

/// Compile and run Peps source using input lines supplied in source order.
pub fn run_source_with_inputs<I, S>(source: &str, inputs: I) -> Result<Vec<String>, RunError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let bytecode = compile_source(source).map_err(|diagnostics| RunError {
        output: Vec::new(),
        diagnostics,
    })?;
    vm::execute_with_inputs(&bytecode, inputs)
}

/// Compile and run Peps source with a caller-provided instruction step limit.
pub fn run_source_with_step_limit(
    source: &str,
    step_limit: usize,
) -> Result<Vec<String>, RunError> {
    let bytecode = compile_source(source).map_err(|diagnostics| RunError {
        output: Vec::new(),
        diagnostics,
    })?;
    vm::execute_with_step_limit(&bytecode, step_limit)
}

/// Compile and run Peps source with queued inputs and a step limit.
pub fn run_source_with_inputs_and_step_limit<I, S>(
    source: &str,
    inputs: I,
    step_limit: usize,
) -> Result<Vec<String>, RunError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let bytecode = compile_source(source).map_err(|diagnostics| RunError {
        output: Vec::new(),
        diagnostics,
    })?;
    vm::execute_with_inputs_and_limit(&bytecode, inputs, ExecutionLimit::Steps(step_limit))
}
