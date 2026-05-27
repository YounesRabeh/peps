//! Semantic checker for declaration rules, type inference, and v1 loop scopes.

use std::collections::{HashMap, HashSet};

use crate::{
    ast::{BinaryOp, Expr, ForSource, Program, Stmt, UnaryOp},
    diagnostic::Diagnostic,
    symbol_table::SymbolTable,
    types::Type,
};

#[derive(Debug, Clone)]
pub struct CheckedProgram {
    pub program: Program,
    pub symbols: SymbolTable,
    pub emoji_literals: HashSet<(usize, usize)>,
}

pub fn check(program: Program) -> Result<CheckedProgram, Vec<Diagnostic>> {
    let mut checker = Checker {
        symbols: SymbolTable::new(),
        local_scopes: Vec::new(),
        diagnostics: Vec::new(),
        emoji_literals: HashSet::new(),
    };

    for statement in &program.statements {
        checker.check_statement(statement, 0, 0);
    }

    if checker.diagnostics.is_empty() {
        Ok(CheckedProgram {
            program,
            symbols: checker.symbols,
            emoji_literals: checker.emoji_literals,
        })
    } else {
        Err(checker.diagnostics)
    }
}

struct Checker {
    symbols: SymbolTable,
    local_scopes: Vec<HashMap<String, Type>>,
    diagnostics: Vec<Diagnostic>,
    emoji_literals: HashSet<(usize, usize)>,
}

impl Checker {
    fn check_statement(&mut self, statement: &Stmt, depth: usize, loop_depth: usize) {
        match statement {
            Stmt::Assign { name, expr, span } => {
                if depth > 0 {
                    self.diagnostics.push(Diagnostic::at(
                        "variable declarations inside blocks are not supported in Peps v0",
                        *span,
                    ));
                    return;
                }

                if self.symbols.contains(name) {
                    self.diagnostics.push(Diagnostic::at(
                        format!("Variable {} is already assigned and cannot be reassigned.", name),
                        *span,
                    ));
                    return;
                }

                if let Some(ty) = self.infer_assignment_rhs(expr) {
                    self.symbols.insert(name.clone(), ty);
                }
            }
            Stmt::Print { expr, .. } => {
                self.infer_expr(expr);
            }
            Stmt::Break { span } => {
                if loop_depth == 0 {
                    self.diagnostics
                        .push(Diagnostic::at("break can only be used inside loops", *span));
                }
            }
            Stmt::Continue { span } => {
                if loop_depth == 0 {
                    self.diagnostics
                        .push(Diagnostic::at("continue can only be used inside loops", *span));
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => {
                self.check_condition(condition, "if", *span);
                for statement in then_branch {
                    self.check_statement(statement, depth + 1, loop_depth);
                }
                if let Some(else_branch) = else_branch {
                    for statement in else_branch {
                        self.check_statement(statement, depth + 1, loop_depth);
                    }
                }
            }
            Stmt::While {
                condition,
                body,
                span,
            } => {
                self.check_condition(condition, "while", *span);
                for statement in body {
                    self.check_statement(statement, depth + 1, loop_depth + 1);
                }
            }
            Stmt::For {
                variable,
                source,
                body,
                span,
            } => {
                let variable_available = self.check_loop_variable_available(variable, *span);
                let loop_type = self.infer_for_source(source);

                if variable_available {
                    if let Some(loop_type) = loop_type {
                        self.push_scope();
                        self.insert_local(variable.clone(), loop_type);
                        for statement in body {
                            self.check_statement(statement, depth + 1, loop_depth + 1);
                        }
                        self.pop_scope();
                    }
                }
            }
        }
    }

    fn check_loop_variable_available(&mut self, variable: &str, span: crate::source::Span) -> bool {
        if self.lookup(variable).is_some() {
            self.diagnostics.push(Diagnostic::at(
                format!("loop variable {} is already declared", variable),
                span,
            ));
            false
        } else {
            true
        }
    }

    fn infer_for_source(&mut self, source: &ForSource) -> Option<Type> {
        match source {
            ForSource::List { expr, span } => match self.infer_expr(expr) {
                Some(Type::List(element_type)) => Some(*element_type),
                Some(_) => {
                    self.diagnostics
                        .push(Diagnostic::at("for-each source must be a list", *span));
                    None
                }
                None => None,
            },
            ForSource::Range { start, end, span } => {
                let start_type = self.infer_expr(start);
                let end_type = self.infer_expr(end);
                if start_type == Some(Type::Num) && end_type == Some(Type::Num) {
                    Some(Type::Num)
                } else {
                    self.diagnostics
                        .push(Diagnostic::at("range bounds must be num", *span));
                    None
                }
            }
        }
    }

    fn check_condition(&mut self, condition: &Expr, kind: &str, span: crate::source::Span) {
        if let Some(ty) = self.infer_expr(condition) {
            if ty != Type::Bool {
                self.diagnostics
                    .push(Diagnostic::at(format!("{} condition must be bool", kind), span));
            }
        }
    }

    fn infer_assignment_rhs(&mut self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::String { .. } => Some(Type::Str),
            Expr::Variable { name, span } => match self.lookup(name) {
                Some(ty) => Some(ty.clone()),
                None => {
                    self.emoji_literals.insert((span.start, span.end));
                    Some(Type::Emoji)
                }
            },
            Expr::List { elements, span } => self.infer_list(elements, *span, true, true),
            Expr::Binary {
                left,
                op: BinaryOp::Add,
                right,
                span,
            } if contains_raw_string(left) || contains_raw_string(right) => {
                self.diagnostics.push(Diagnostic::at(
                    "string concatenation is not supported in Peps v0",
                    *span,
                ));
                None
            }
            _ => self.infer_expr(expr),
        }
    }

    fn infer_expr(&mut self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Number { .. } => Some(Type::Num),
            Expr::String { span, .. } => {
                self.diagnostics.push(Diagnostic::at(
                    "Raw string literals can only be assigned to variables in Peps v0.",
                    *span,
                ));
                None
            }
            Expr::Bool { .. } => Some(Type::Bool),
            Expr::Emoji { .. } => Some(Type::Emoji),
            Expr::Variable { name, span } => match self.lookup(name) {
                Some(ty) => Some(ty.clone()),
                None => {
                    self.emoji_literals.insert((span.start, span.end));
                    Some(Type::Emoji)
                }
            },
            Expr::List { elements, span } => self.infer_list(elements, *span, false, false),
            Expr::Unary { op, expr, span } => {
                let ty = self.infer_expr(expr)?;
                match op {
                    UnaryOp::Negate if ty == Type::Num => Some(Type::Num),
                    UnaryOp::Negate => {
                        self.diagnostics.push(Diagnostic::at(
                            "numeric negation requires a num operand",
                            *span,
                        ));
                        None
                    }
                }
            }
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => self.infer_binary(left, *op, right, *span),
        }
    }

    fn infer_binary(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: crate::source::Span,
    ) -> Option<Type> {
        if matches!(op, BinaryOp::Add) && (contains_raw_string(left) || contains_raw_string(right)) {
            self.diagnostics.push(Diagnostic::at(
                "string concatenation is not supported in Peps v0",
                span,
            ));
            return None;
        }

        let left_ty = self.infer_expr(left)?;
        let right_ty = self.infer_expr(right)?;

        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                if left_ty == Type::Num && right_ty == Type::Num {
                    Some(Type::Num)
                } else if op == BinaryOp::Add && (left_ty == Type::Str || right_ty == Type::Str) {
                    self.diagnostics.push(Diagnostic::at(
                        "string concatenation is not supported in Peps v0",
                        span,
                    ));
                    None
                } else {
                    self.diagnostics.push(Diagnostic::at(
                        format!("arithmetic operator {} requires num operands", op_symbol(op)),
                        span,
                    ));
                    None
                }
            }
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                if left_ty == Type::Num && right_ty == Type::Num {
                    Some(Type::Bool)
                } else {
                    self.diagnostics.push(Diagnostic::at(
                        "ordering comparison requires num operands",
                        span,
                    ));
                    None
                }
            }
            BinaryOp::Eq | BinaryOp::NotEq => {
                if matches!(left_ty, Type::List(_)) || matches!(right_ty, Type::List(_)) {
                    self.diagnostics.push(Diagnostic::at(
                        "list equality is not supported in Peps v0",
                        span,
                    ));
                    None
                } else if left_ty == right_ty {
                    Some(Type::Bool)
                } else {
                    self.diagnostics.push(Diagnostic::at(
                        "equality comparison requires operands of the same type",
                        span,
                    ));
                    None
                }
            }
        }
    }

    fn infer_list(
        &mut self,
        elements: &[Expr],
        span: crate::source::Span,
        allow_raw_strings: bool,
        allow_emoji_literals: bool,
    ) -> Option<Type> {
        if elements.is_empty() {
            self.diagnostics.push(Diagnostic::at(
                "empty lists are not supported in Peps v0 because their element type cannot be inferred",
                span,
            ));
            return None;
        }

        let mut element_type: Option<Type> = None;
        for element in elements {
            if let Expr::List { span, .. } = element {
                self.diagnostics.push(Diagnostic::at(
                    "nested lists are not supported in Peps v0",
                    *span,
                ));
                return None;
            }

            let ty = match element {
                Expr::String { .. } if allow_raw_strings => Type::Str,
                Expr::String { span, .. } => {
                    self.diagnostics.push(Diagnostic::at(
                        "Raw string literals can only be assigned to variables in Peps v0.",
                        *span,
                    ));
                    return None;
                }
                Expr::Variable { name, span }
                    if allow_emoji_literals && self.lookup(name).is_none() =>
                {
                    self.emoji_literals.insert((span.start, span.end));
                    Type::Emoji
                }
                _ => self.infer_expr(element)?,
            };

            if let Some(expected) = &element_type {
                if expected != &ty {
                    self.diagnostics.push(Diagnostic::at(
                        "list elements must all have the same type",
                        element.span(),
                    ));
                    return None;
                }
            } else {
                element_type = Some(ty);
            }
        }

        element_type.map(|ty| Type::List(Box::new(ty)))
    }

    fn lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.local_scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        self.symbols.get(name)
    }

    fn push_scope(&mut self) {
        self.local_scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    fn insert_local(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(name, ty);
        }
    }
}

fn contains_raw_string(expr: &Expr) -> bool {
    match expr {
        Expr::String { .. } => true,
        Expr::List { elements, .. } => elements.iter().any(contains_raw_string),
        Expr::Unary { expr, .. } => contains_raw_string(expr),
        Expr::Binary { left, right, .. } => contains_raw_string(left) || contains_raw_string(right),
        _ => false,
    }
}

fn op_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "➕",
        BinaryOp::Sub => "➖",
        BinaryOp::Mul => "✖️",
        BinaryOp::Div => "➗",
        BinaryOp::Eq => "🟰🟰",
        BinaryOp::NotEq => "❌🟰",
        BinaryOp::Lt => "◀️",
        BinaryOp::Gt => "▶️",
        BinaryOp::LtEq => "◀️🟰",
        BinaryOp::GtEq => "▶️🟰",
    }
}
