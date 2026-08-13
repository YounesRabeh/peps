//! Bytecode compiler for semantically checked Peps programs.

use std::collections::{HashMap, HashSet};

use num_bigint::BigInt;

use crate::{
    ast::{BinaryOp, Expr, ForSource, Stmt, UnaryOp},
    bytecode::{Instruction, Value},
    diagnostic::Diagnostic,
    semantic::CheckedProgram,
};

/// Compile a semantically checked Peps program into bytecode instructions.
pub fn compile(checked: CheckedProgram) -> Result<Vec<Instruction>, Vec<Diagnostic>> {
    let global_scope = checked
        .symbols
        .iter()
        .map(|(name, _)| (name.clone(), Binding::Global(name.clone())))
        .collect();
    let mut compiler = Compiler {
        instructions: Vec::new(),
        emoji_literals: checked.emoji_literals,
        loop_counter: 0,
        local_counter: 0,
        loop_stack: Vec::new(),
        scopes: vec![global_scope],
        functions: HashMap::new(),
        pending_calls: Vec::new(),
    };

    for statement in &checked.program.statements {
        if !matches!(statement, Stmt::Function { .. }) {
            compiler.compile_statement(statement);
        }
    }

    let function_statements = checked
        .program
        .statements
        .iter()
        .filter(|statement| matches!(statement, Stmt::Function { .. }))
        .collect::<Vec<_>>();
    if !function_statements.is_empty() {
        let skip_functions = compiler.emit_placeholder_jump();
        for statement in function_statements {
            compiler.compile_function(statement);
        }
        let end = compiler.instructions.len();
        compiler.patch_jump(skip_functions, end);
    }
    for (index, name) in compiler.pending_calls {
        let target = compiler.functions[&name];
        match &mut compiler.instructions[index] {
            Instruction::Call { target: slot, .. } => *slot = target,
            _ => unreachable!("pending call did not point at call bytecode"),
        }
    }

    Ok(compiler.instructions)
}

struct Compiler {
    instructions: Vec<Instruction>,
    emoji_literals: HashSet<(usize, usize)>,
    loop_counter: usize,
    local_counter: usize,
    loop_stack: Vec<LoopContext>,
    /// Source names mapped to their bytecode storage names in each lexical scope.
    scopes: Vec<HashMap<String, Binding>>,
    functions: HashMap<String, usize>,
    pending_calls: Vec<(usize, String)>,
}

#[derive(Debug, Clone)]
enum Binding {
    Global(String),
    Local(String),
}

#[derive(Default)]
struct LoopContext {
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
}

impl Compiler {
    /// Compile one statement and append its bytecode to the instruction stream.
    fn compile_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign { name, expr, .. } => {
                self.compile_expr(expr);
                let binding = self
                    .resolve_name(name)
                    .unwrap_or_else(|| self.declare_name(name));
                self.emit_store(&binding);
            }
            Stmt::Append { name, expr, .. } => {
                let binding = self
                    .resolve_name(name)
                    .expect("semantic checker accepted an unresolved append target");
                self.emit_load(&binding);
                self.compile_expr(expr);
                self.instructions.push(Instruction::ListAppend);
                self.emit_store(&binding);
            }
            Stmt::Print { expr, .. } => {
                self.compile_expr(expr);
                self.instructions.push(Instruction::Print);
            }
            Stmt::Break { .. } => self.emit_loop_break(),
            Stmt::Continue { .. } => self.emit_loop_continue(),
            Stmt::Function { .. } => {}
            Stmt::Return { expr, .. } => {
                self.compile_expr(expr);
                self.instructions.push(Instruction::Return);
            }
            Stmt::Call { expr, .. } => {
                self.compile_expr(expr);
                self.instructions.push(Instruction::Pop);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.compile_expr(condition);
                let jump_if_false = self.emit_placeholder_jump_if_false();

                self.push_scope();
                for statement in then_branch {
                    self.compile_statement(statement);
                }
                self.pop_scope();

                if let Some(else_branch) = else_branch {
                    let jump_after_else = self.emit_placeholder_jump();
                    let else_start = self.instructions.len();
                    self.patch_jump(jump_if_false, else_start);

                    self.push_scope();
                    for statement in else_branch {
                        self.compile_statement(statement);
                    }
                    self.pop_scope();

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

                self.push_scope();
                for statement in body {
                    self.compile_statement(statement);
                }
                self.pop_scope();

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

    fn compile_function(&mut self, statement: &Stmt) {
        let Stmt::Function {
            name,
            parameters,
            body,
            ..
        } = statement
        else {
            unreachable!()
        };

        let entry = self.instructions.len();
        self.functions.insert(name.clone(), entry);
        self.scopes.truncate(1);
        self.push_scope();
        self.loop_stack.clear();

        let parameter_bindings = parameters
            .iter()
            .map(|parameter| self.declare_local_name(parameter))
            .collect::<Vec<_>>();
        for binding in parameter_bindings.iter().rev() {
            self.emit_store(binding);
        }
        for statement in body {
            self.compile_statement(statement);
        }
        self.pop_scope();
    }

    /// Compile a `for` statement according to the kind of source it iterates.
    fn compile_for(&mut self, variable: &str, source: &ForSource, body: &[Stmt]) {
        match source {
            ForSource::List { expr, .. } => self.compile_for_list(variable, expr, body),
            ForSource::Range { start, end, .. } => {
                self.compile_for_range(variable, start, end, body)
            }
        }
    }

    /// Compile a `for` loop over a list using generated list, index, and length variables.
    fn compile_for_list(&mut self, variable: &str, source: &Expr, body: &[Stmt]) {
        let id = self.next_loop_id();
        let list_name = format!("__peps_for_{}_list", id);
        let index_name = format!("__peps_for_{}_index", id);
        let len_name = format!("__peps_for_{}_len", id);

        self.compile_expr(source);
        self.push_scope();
        let variable_name = self.declare_name(variable);
        self.instructions
            .push(Instruction::StoreLocal(list_name.clone()));
        self.instructions
            .push(Instruction::LoadConst(Value::Num(BigInt::from(0_u8))));
        self.instructions
            .push(Instruction::StoreLocal(index_name.clone()));
        self.instructions
            .push(Instruction::LoadLocal(list_name.clone()));
        self.instructions.push(Instruction::ListLen);
        self.instructions
            .push(Instruction::StoreLocal(len_name.clone()));

        let loop_start = self.instructions.len();
        self.instructions
            .push(Instruction::LoadLocal(index_name.clone()));
        self.instructions.push(Instruction::LoadLocal(len_name));
        self.instructions.push(Instruction::Lt);
        let jump_if_false = self.emit_placeholder_jump_if_false();

        self.instructions.push(Instruction::LoadLocal(list_name));
        self.instructions
            .push(Instruction::LoadLocal(index_name.clone()));
        self.instructions.push(Instruction::ListGet);
        self.emit_store(&variable_name);
        self.begin_loop();

        for statement in body {
            self.compile_statement(statement);
        }

        let continue_target = self.instructions.len();
        self.instructions
            .push(Instruction::LoadLocal(index_name.clone()));
        self.instructions
            .push(Instruction::LoadConst(Value::Num(BigInt::from(1_u8))));
        self.instructions.push(Instruction::Add);
        self.instructions.push(Instruction::StoreLocal(index_name));
        self.instructions.push(Instruction::Jump(loop_start));

        let after_loop = self.instructions.len();
        self.patch_jump(jump_if_false, after_loop);
        self.end_loop(continue_target, after_loop);
        self.pop_scope();
    }

    /// Compile a numeric range loop with an exclusive upper bound.
    fn compile_for_range(&mut self, variable: &str, start: &Expr, end: &Expr, body: &[Stmt]) {
        let id = self.next_loop_id();
        let index_name = format!("__peps_for_{}_index", id);
        let end_name = format!("__peps_for_{}_end", id);

        self.compile_expr(start);
        self.instructions
            .push(Instruction::StoreLocal(index_name.clone()));
        self.compile_expr(end);
        self.instructions
            .push(Instruction::StoreLocal(end_name.clone()));
        self.push_scope();
        let variable_name = self.declare_name(variable);

        let loop_start = self.instructions.len();
        self.instructions
            .push(Instruction::LoadLocal(index_name.clone()));
        self.instructions.push(Instruction::LoadLocal(end_name));
        self.instructions.push(Instruction::Lt);
        let jump_if_false = self.emit_placeholder_jump_if_false();

        self.instructions
            .push(Instruction::LoadLocal(index_name.clone()));
        self.emit_store(&variable_name);
        self.begin_loop();

        for statement in body {
            self.compile_statement(statement);
        }

        let continue_target = self.instructions.len();
        self.instructions
            .push(Instruction::LoadLocal(index_name.clone()));
        self.instructions
            .push(Instruction::LoadConst(Value::Num(BigInt::from(1_u8))));
        self.instructions.push(Instruction::Add);
        self.instructions.push(Instruction::StoreLocal(index_name));
        self.instructions.push(Instruction::Jump(loop_start));

        let after_loop = self.instructions.len();
        self.patch_jump(jump_if_false, after_loop);
        self.end_loop(continue_target, after_loop);
        self.pop_scope();
    }

    /// Compile an expression so its value is left on the VM stack.
    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number { value, .. } => self
                .instructions
                .push(Instruction::LoadConst(Value::Num(value.clone()))),
            Expr::Float { value, .. } => self
                .instructions
                .push(Instruction::LoadConst(Value::Float(*value))),
            Expr::Input { kind, .. } => self.instructions.push(Instruction::Input(*kind)),
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
                    let binding = self
                        .resolve_name(name)
                        .expect("semantic checker accepted an unresolved variable");
                    self.emit_load(&binding);
                }
            }
            Expr::List { elements, .. } => {
                for element in elements {
                    self.compile_expr(element);
                }
                self.instructions
                    .push(Instruction::MakeList(elements.len()));
            }
            Expr::Unary {
                op: UnaryOp::Negate,
                expr,
                ..
            } => {
                if let Expr::Number { value, .. } = expr.as_ref() {
                    self.instructions
                        .push(Instruction::LoadConst(Value::Num(-value.clone())));
                } else if let Expr::Float { value, .. } = expr.as_ref() {
                    self.instructions
                        .push(Instruction::LoadConst(Value::Float(-value)));
                } else {
                    self.instructions
                        .push(Instruction::LoadConst(Value::Num(BigInt::from(0_u8))));
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
                left,
                op: BinaryOp::And,
                right,
                ..
            } => self.compile_logical_and(left, right),
            Expr::Binary {
                left,
                op: BinaryOp::Or,
                right,
                ..
            } => self.compile_logical_or(left, right),
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
            Expr::Call {
                name, arguments, ..
            } => {
                for argument in arguments {
                    self.compile_expr(argument);
                }
                let index = self.instructions.len();
                self.instructions.push(Instruction::Call {
                    target: usize::MAX,
                    arity: arguments.len(),
                });
                self.pending_calls.push((index, name.clone()));
            }
        }
    }

    /// Compile logical not by branching to explicit boolean constants.
    fn compile_unary_not(&mut self, expr: &Expr) {
        self.compile_expr(expr);
        let jump_if_false = self.emit_placeholder_jump_if_false();
        self.instructions
            .push(Instruction::LoadConst(Value::Bool(false)));
        let jump_end = self.emit_placeholder_jump();
        let true_branch = self.instructions.len();
        self.patch_jump(jump_if_false, true_branch);
        self.instructions
            .push(Instruction::LoadConst(Value::Bool(true)));
        let after = self.instructions.len();
        self.patch_jump(jump_end, after);
    }

    /// Compile short-circuiting logical `and`.
    fn compile_logical_and(&mut self, left: &Expr, right: &Expr) {
        self.compile_expr(left);
        let left_false = self.emit_placeholder_jump_if_false();
        self.compile_expr(right);
        let right_false = self.emit_placeholder_jump_if_false();
        self.instructions
            .push(Instruction::LoadConst(Value::Bool(true)));
        let jump_end = self.emit_placeholder_jump();
        let false_branch = self.instructions.len();
        self.patch_jump(left_false, false_branch);
        self.patch_jump(right_false, false_branch);
        self.instructions
            .push(Instruction::LoadConst(Value::Bool(false)));
        let after = self.instructions.len();
        self.patch_jump(jump_end, after);
    }

    /// Compile short-circuiting logical `or`.
    fn compile_logical_or(&mut self, left: &Expr, right: &Expr) {
        self.compile_expr(left);
        let left_false = self.emit_placeholder_jump_if_false();
        self.instructions
            .push(Instruction::LoadConst(Value::Bool(true)));
        let jump_end_left = self.emit_placeholder_jump();
        let eval_right = self.instructions.len();
        self.patch_jump(left_false, eval_right);

        self.compile_expr(right);
        let right_false = self.emit_placeholder_jump_if_false();
        self.instructions
            .push(Instruction::LoadConst(Value::Bool(true)));
        let jump_end_right = self.emit_placeholder_jump();
        let false_branch = self.instructions.len();
        self.patch_jump(right_false, false_branch);
        self.instructions
            .push(Instruction::LoadConst(Value::Bool(false)));
        let after = self.instructions.len();
        self.patch_jump(jump_end_left, after);
        self.patch_jump(jump_end_right, after);
    }

    /// Emit a conditional jump with a placeholder target and return its index.
    fn emit_placeholder_jump_if_false(&mut self) -> usize {
        let index = self.instructions.len();
        self.instructions.push(Instruction::JumpIfFalse(usize::MAX));
        index
    }

    /// Start collecting `break` and `continue` jumps for a loop body.
    fn begin_loop(&mut self) {
        self.loop_stack.push(LoopContext::default());
    }

    /// Patch loop `continue` and `break` jumps, then leave the current loop context.
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

    /// Emit a placeholder jump that will later target the end of the current loop.
    fn emit_loop_break(&mut self) {
        let jump = self.emit_placeholder_jump();
        let context = self
            .loop_stack
            .last_mut()
            .expect("break used outside loop during compilation");
        context.break_jumps.push(jump);
    }

    /// Emit a placeholder jump that will later target the current loop increment.
    fn emit_loop_continue(&mut self) {
        let jump = self.emit_placeholder_jump();
        let context = self
            .loop_stack
            .last_mut()
            .expect("continue used outside loop during compilation");
        context.continue_jumps.push(jump);
    }

    /// Return a unique numeric suffix for generated loop variables.
    fn next_loop_id(&mut self) -> usize {
        let id = self.loop_counter;
        self.loop_counter += 1;
        id
    }

    /// Look up the bytecode storage name for the nearest visible binding.
    fn resolve_name(&self, name: &str) -> Option<Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    /// Declare a source name in the current scope and return its storage name.
    fn declare_name(&mut self, name: &str) -> Binding {
        let binding = if self.scopes.len() == 1 {
            Binding::Global(name.to_string())
        } else {
            self.new_local_binding()
        };
        self.scopes
            .last_mut()
            .expect("compiler scope stack underflow")
            .insert(name.to_string(), binding.clone());
        binding
    }

    fn declare_local_name(&mut self, name: &str) -> Binding {
        let binding = self.new_local_binding();
        self.scopes
            .last_mut()
            .expect("compiler scope stack underflow")
            .insert(name.to_string(), binding.clone());
        binding
    }

    fn new_local_binding(&mut self) -> Binding {
        let id = self.local_counter;
        self.local_counter += 1;
        Binding::Local(format!("__peps_local_{}", id))
    }

    fn emit_load(&mut self, binding: &Binding) {
        self.instructions.push(match binding {
            Binding::Global(name) => Instruction::LoadVar(name.clone()),
            Binding::Local(name) => Instruction::LoadLocal(name.clone()),
        });
    }

    fn emit_store(&mut self, binding: &Binding) {
        self.instructions.push(match binding {
            Binding::Global(name) => Instruction::StoreVar(name.clone()),
            Binding::Local(name) => Instruction::StoreLocal(name.clone()),
        });
    }

    /// Start a lexical block scope.
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// End the innermost lexical block scope.
    fn pop_scope(&mut self) {
        assert!(
            self.scopes.len() > 1,
            "cannot pop the compiler's top-level scope"
        );
        self.scopes.pop();
    }

    /// Emit an unconditional jump with a placeholder target and return its index.
    fn emit_placeholder_jump(&mut self) -> usize {
        let index = self.instructions.len();
        self.instructions.push(Instruction::Jump(usize::MAX));
        index
    }

    /// Replace a placeholder jump target with its final instruction index.
    fn patch_jump(&mut self, index: usize, target: usize) {
        match &mut self.instructions[index] {
            Instruction::Jump(destination) | Instruction::JumpIfFalse(destination) => {
                *destination = target;
            }
            _ => unreachable!("attempted to patch a non-jump instruction"),
        }
    }
}
