pub mod ast;
pub mod bytecode;
pub mod compiler;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod source;
pub mod symbol_table;
pub mod token;
pub mod types;
pub mod vm;

pub use ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
pub use bytecode::{Instruction, Value};
pub use diagnostic::Diagnostic;
pub use source::Span;
pub use token::{Token, TokenKind};
pub use types::Type;
pub use vm::{RunError, RuntimeValue};

pub fn compile_source(source: &str) -> Result<Vec<Instruction>, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    let checked_program = semantic::check(program)?;
    compiler::compile(checked_program)
}

pub fn run_source(source: &str) -> Result<Vec<String>, RunError> {
    let bytecode = compile_source(source).map_err(|diagnostics| RunError {
        output: Vec::new(),
        diagnostics,
    })?;
    vm::execute(&bytecode)
}
