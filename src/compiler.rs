use crate::{
    ast::{BinaryOp, Expr, Stmt, UnaryOp},
    bytecode::{Instruction, Value},
    diagnostic::Diagnostic,
    semantic::CheckedProgram,
};

pub fn compile(checked: CheckedProgram) -> Result<Vec<Instruction>, Vec<Diagnostic>> {
    let mut compiler = Compiler {
        instructions: Vec::new(),
        emoji_literals: checked.emoji_literals,
    };

    for statement in &checked.program.statements {
        compiler.compile_statement(statement);
    }

    Ok(compiler.instructions)
}

struct Compiler {
    instructions: Vec<Instruction>,
    emoji_literals: std::collections::HashSet<(usize, usize)>,
}

impl Compiler {
    fn compile_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign { name, expr, .. } => {
                self.compile_expr(expr);
                self.instructions.push(Instruction::StoreVar(name.clone()));
            }
            Stmt::Print { expr, .. } => {
                self.compile_expr(expr);
                self.instructions.push(Instruction::Print);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.compile_expr(condition);
                let jump_if_false = self.emit_placeholder_jump_if_false();

                for statement in then_branch {
                    self.compile_statement(statement);
                }

                if let Some(else_branch) = else_branch {
                    let jump_after_else = self.emit_placeholder_jump();
                    let else_start = self.instructions.len();
                    self.patch_jump(jump_if_false, else_start);

                    for statement in else_branch {
                        self.compile_statement(statement);
                    }

                    let after_else = self.instructions.len();
                    self.patch_jump(jump_after_else, after_else);
                } else {
                    let after_then = self.instructions.len();
                    self.patch_jump(jump_if_false, after_then);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                let loop_start = self.instructions.len();
                self.compile_expr(condition);
                let jump_if_false = self.emit_placeholder_jump_if_false();

                for statement in body {
                    self.compile_statement(statement);
                }

                self.instructions.push(Instruction::Jump(loop_start));
                let after_loop = self.instructions.len();
                self.patch_jump(jump_if_false, after_loop);
            }
        }
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number { value, .. } => self
                .instructions
                .push(Instruction::LoadConst(Value::Num(*value))),
            Expr::String { value, .. } => self
                .instructions
                .push(Instruction::LoadConst(Value::Str(value.clone()))),
            Expr::Bool { value, .. } => self
                .instructions
                .push(Instruction::LoadConst(Value::Bool(*value))),
            Expr::Emoji { value, .. } => self
                .instructions
                .push(Instruction::LoadConst(Value::Emoji(value.clone()))),
            Expr::Variable { name, span } => {
                if self.emoji_literals.contains(&(span.start, span.end)) {
                    self.instructions
                        .push(Instruction::LoadConst(Value::Emoji(name.clone())));
                } else {
                    self.instructions.push(Instruction::LoadVar(name.clone()));
                }
            }
            Expr::List { elements, .. } => {
                for element in elements {
                    self.compile_expr(element);
                }
                self.instructions.push(Instruction::MakeList(elements.len()));
            }
            Expr::Unary {
                op: UnaryOp::Negate,
                expr,
                ..
            } => {
                if let Expr::Number { value, .. } = expr.as_ref() {
                    self.instructions
                        .push(Instruction::LoadConst(Value::Num(-value)));
                } else {
                    self.instructions.push(Instruction::LoadConst(Value::Num(0)));
                    self.compile_expr(expr);
                    self.instructions.push(Instruction::Sub);
                }
            }
            Expr::Binary {
                left, op, right, ..
            } => {
                self.compile_expr(left);
                self.compile_expr(right);
                self.instructions.push(match op {
                    BinaryOp::Add => Instruction::Add,
                    BinaryOp::Sub => Instruction::Sub,
                    BinaryOp::Mul => Instruction::Mul,
                    BinaryOp::Div => Instruction::Div,
                    BinaryOp::Eq => Instruction::Eq,
                    BinaryOp::NotEq => Instruction::NotEq,
                    BinaryOp::Lt => Instruction::Lt,
                    BinaryOp::Gt => Instruction::Gt,
                    BinaryOp::LtEq => Instruction::LtEq,
                    BinaryOp::GtEq => Instruction::GtEq,
                });
            }
        }
    }

    fn emit_placeholder_jump_if_false(&mut self) -> usize {
        let index = self.instructions.len();
        self.instructions.push(Instruction::JumpIfFalse(usize::MAX));
        index
    }

    fn emit_placeholder_jump(&mut self) -> usize {
        let index = self.instructions.len();
        self.instructions.push(Instruction::Jump(usize::MAX));
        index
    }

    fn patch_jump(&mut self, index: usize, target: usize) {
        match &mut self.instructions[index] {
            Instruction::Jump(destination) | Instruction::JumpIfFalse(destination) => {
                *destination = target;
            }
            _ => unreachable!("attempted to patch a non-jump instruction"),
        }
    }
}
