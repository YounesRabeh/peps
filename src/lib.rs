//! Public library API for the Peps compiler and bytecode runner.
//!
//! The crate is organized around compiler layers:
//! frontend parsing, middle-end semantic analysis, backend bytecode generation,
//! and runtime bytecode execution.

pub mod backend;
pub mod common;
pub mod frontend;
pub mod middle_end;
pub mod runtime;

pub use backend::{bytecode, compiler};
pub use common::{diagnostic, source};
pub use frontend::{ast, lexer, parser, token};
pub use middle_end::{semantic, symbol_table, types};
pub use runtime::vm;

pub use ast::{BinaryOp, Expr, ForSource, Program, Stmt, UnaryOp};
pub use bytecode::{Instruction, Value};
pub use diagnostic::Diagnostic;
pub use source::Span;
pub use token::{Token, TokenKind};
pub use types::Type;
pub use vm::{RunError, RuntimeValue};

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
