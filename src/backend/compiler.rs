//! Bytecode compiler for semantically checked Peps programs.

use crate::{
    ast::{BinaryOp, Expr, ForSource, Stmt, UnaryOp},
    bytecode::{Instruction, Value},
    diagnostic::Diagnostic,
    semantic::CheckedProgram,
};

pub fn compile(checked: CheckedProgram) -> Result<Vec<Instruction>, Vec<Diagnostic>> {
    let mut compiler = Compiler {
        instructions: Vec::new(),
        emoji_literals: checked.emoji_literals,
        loop_counter: 0,
        loop_stack: Vec::new(),
    };

    for statement in &checked.program.statements {
        compiler.compile_statement(statement);
    }

    Ok(compiler.instructions)
}

struct Compiler {
    instructions: Vec<Instruction>,
    emoji_literals: std::collections::HashSet<(usize, usize)>,
    loop_counter: usize,
    loop_stack: Vec<LoopContext>,
}

#[derive(Default)]
struct LoopContext {
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
}

impl Compiler {
    fn compile_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign { name, expr, .. } => {
                self.compile_expr(expr);
                self.instructions.push(Instruction::StoreVar(name.clone()));
            }
            Stmt::Append { name, expr, .. } => {
                self.instructions.push(Instruction::LoadVar(name.clone()));
                self.compile_expr(expr);
                self.instructions.push(Instruction::ListAppend);
                self.instructions.push(Instruction::StoreVar(name.clone()));
            }
            Stmt::Print { expr, .. } => {
                self.compile_expr(expr);
                self.instructions.push(Instruction::Print);
            }
            Stmt::Break { .. } => self.emit_loop_break(),
            Stmt::Continue { .. } => self.emit_loop_continue(),
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
                self.begin_loop();

                for statement in body {
                    self.compile_statement(statement);
                }

                self.instructions.push(Instruction::Jump(loop_start));
                let after_loop = self.instructions.len();
                self.patch_jump(jump_if_false, after_loop);
                self.end_loop(loop_start, after_loop);
            }
            Stmt::For {
                variable,
                source,
                body,
                ..
            } => self.compile_for(variable, source, body),
        }
    }

    fn compile_for(&mut self, variable: &str, source: &ForSource, body: &[Stmt]) {
        match source {
            ForSource::List { expr, .. } => self.compile_for_list(variable, expr, body),
            ForSource::Range { start, end, .. } => self.compile_for_range(variable, start, end, body),
        }
    }

    fn compile_for_list(&mut self, variable: &str, source: &Expr, body: &[Stmt]) {
        let id = self.next_loop_id();
        let list_name = format!("__peps_for_{}_list", id);
        let index_name = format!("__peps_for_{}_index", id);
        let len_name = format!("__peps_for_{}_len", id);

        self.compile_expr(source);
        self.instructions.push(Instruction::StoreVar(list_name.clone()));
        self.instructions.push(Instruction::LoadConst(Value::Num(0)));
        self.instructions
            .push(Instruction::StoreVar(index_name.clone()));
        self.instructions.push(Instruction::LoadVar(list_name.clone()));
        self.instructions.push(Instruction::ListLen);
        self.instructions.push(Instruction::StoreVar(len_name.clone()));

        let loop_start = self.instructions.len();
        self.instructions.push(Instruction::LoadVar(index_name.clone()));
        self.instructions.push(Instruction::LoadVar(len_name));
        self.instructions.push(Instruction::Lt);
        let jump_if_false = self.emit_placeholder_jump_if_false();

        self.instructions.push(Instruction::LoadVar(list_name));
        self.instructions.push(Instruction::LoadVar(index_name.clone()));
        self.instructions.push(Instruction::ListGet);
        self.instructions
            .push(Instruction::StoreVar(variable.to_string()));
        self.begin_loop();

        for statement in body {
            self.compile_statement(statement);
        }

        let continue_target = self.instructions.len();
        self.instructions.push(Instruction::LoadVar(index_name.clone()));
        self.instructions.push(Instruction::LoadConst(Value::Num(1)));
        self.instructions.push(Instruction::Add);
        self.instructions.push(Instruction::StoreVar(index_name));
        self.instructions.push(Instruction::Jump(loop_start));

        let after_loop = self.instructions.len();
        self.patch_jump(jump_if_false, after_loop);
        self.end_loop(continue_target, after_loop);
    }

    fn compile_for_range(&mut self, variable: &str, start: &Expr, end: &Expr, body: &[Stmt]) {
        let id = self.next_loop_id();
        let index_name = format!("__peps_for_{}_index", id);
        let end_name = format!("__peps_for_{}_end", id);

        self.compile_expr(start);
        self.instructions
            .push(Instruction::StoreVar(index_name.clone()));
        self.compile_expr(end);
        self.instructions.push(Instruction::StoreVar(end_name.clone()));

        let loop_start = self.instructions.len();
        self.instructions.push(Instruction::LoadVar(index_name.clone()));
        self.instructions.push(Instruction::LoadVar(end_name));
        self.instructions.push(Instruction::Lt);
        let jump_if_false = self.emit_placeholder_jump_if_false();

        self.instructions.push(Instruction::LoadVar(index_name.clone()));
        self.instructions
            .push(Instruction::StoreVar(variable.to_string()));
        self.begin_loop();

        for statement in body {
            self.compile_statement(statement);
        }

        let continue_target = self.instructions.len();
        self.instructions.push(Instruction::LoadVar(index_name.clone()));
        self.instructions.push(Instruction::LoadConst(Value::Num(1)));
        self.instructions.push(Instruction::Add);
        self.instructions.push(Instruction::StoreVar(index_name));
        self.instructions.push(Instruction::Jump(loop_start));

        let after_loop = self.instructions.len();
        self.patch_jump(jump_if_false, after_loop);
        self.end_loop(continue_target, after_loop);
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
            Expr::Unary {
                op: UnaryOp::Len,
                expr,
                ..
            } => {
                self.compile_expr(expr);
                self.instructions.push(Instruction::ListLen);
            }
            Expr::Unary {
                op: UnaryOp::Not,
                expr,
                ..
            } => self.compile_unary_not(expr),
            Expr::Binary {
                left, op, right, ..
            } if matches!(op, BinaryOp::And) => self.compile_logical_and(left, right),
            Expr::Binary {
                left, op, right, ..
            } if matches!(op, BinaryOp::Or) => self.compile_logical_or(left, right),
            Expr::Binary {
                left, op, right, ..
            } => {
                self.compile_expr(left);
                self.compile_expr(right);
                self.instructions.push(match op {
                    BinaryOp::Append => Instruction::ListAppend,
                    BinaryOp::Add => Instruction::Add,
                    BinaryOp::Sub => Instruction::Sub,
                    BinaryOp::Mul => Instruction::Mul,
                    BinaryOp::Div => Instruction::Div,
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                    BinaryOp::Index => Instruction::ListGet,
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

    fn compile_unary_not(&mut self, expr: &Expr) {
        self.compile_expr(expr);
        let jump_if_false = self.emit_placeholder_jump_if_false();
        self.instructions.push(Instruction::LoadConst(Value::Bool(false)));
        let jump_end = self.emit_placeholder_jump();
        let true_branch = self.instructions.len();
        self.patch_jump(jump_if_false, true_branch);
        self.instructions.push(Instruction::LoadConst(Value::Bool(true)));
        let after = self.instructions.len();
        self.patch_jump(jump_end, after);
    }

    fn compile_logical_and(&mut self, left: &Expr, right: &Expr) {
        self.compile_expr(left);
        let left_false = self.emit_placeholder_jump_if_false();
        self.compile_expr(right);
        let right_false = self.emit_placeholder_jump_if_false();
        self.instructions.push(Instruction::LoadConst(Value::Bool(true)));
        let jump_end = self.emit_placeholder_jump();
        let false_branch = self.instructions.len();
        self.patch_jump(left_false, false_branch);
        self.patch_jump(right_false, false_branch);
        self.instructions.push(Instruction::LoadConst(Value::Bool(false)));
        let after = self.instructions.len();
        self.patch_jump(jump_end, after);
    }

    fn compile_logical_or(&mut self, left: &Expr, right: &Expr) {
        self.compile_expr(left);
        let left_false = self.emit_placeholder_jump_if_false();
        self.instructions.push(Instruction::LoadConst(Value::Bool(true)));
        let jump_end_left = self.emit_placeholder_jump();
        let eval_right = self.instructions.len();
        self.patch_jump(left_false, eval_right);

        self.compile_expr(right);
        let right_false = self.emit_placeholder_jump_if_false();
        self.instructions.push(Instruction::LoadConst(Value::Bool(true)));
        let jump_end_right = self.emit_placeholder_jump();
        let false_branch = self.instructions.len();
        self.patch_jump(right_false, false_branch);
        self.instructions.push(Instruction::LoadConst(Value::Bool(false)));
        let after = self.instructions.len();
        self.patch_jump(jump_end_left, after);
        self.patch_jump(jump_end_right, after);
    }

    fn emit_placeholder_jump_if_false(&mut self) -> usize {
        let index = self.instructions.len();
        self.instructions.push(Instruction::JumpIfFalse(usize::MAX));
        index
    }

    fn begin_loop(&mut self) {
        self.loop_stack.push(LoopContext::default());
    }

    fn end_loop(&mut self, continue_target: usize, break_target: usize) {
        let context = self
            .loop_stack
            .pop()
            .expect("compiler loop stack underflow");
        for jump in context.continue_jumps {
            self.patch_jump(jump, continue_target);
        }
        for jump in context.break_jumps {
            self.patch_jump(jump, break_target);
        }
    }

    fn emit_loop_break(&mut self) {
        let jump = self.emit_placeholder_jump();
        let context = self
            .loop_stack
            .last_mut()
            .expect("break used outside loop during compilation");
        context.break_jumps.push(jump);
    }

    fn emit_loop_continue(&mut self) {
        let jump = self.emit_placeholder_jump();
        let context = self
            .loop_stack
            .last_mut()
            .expect("continue used outside loop during compilation");
        context.continue_jumps.push(jump);
    }

    fn next_loop_id(&mut self) -> usize {
        let id = self.loop_counter;
        self.loop_counter += 1;
        id
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
