//! Semantic checker for declaration rules, type inference, and v1 loop scopes.

use std::collections::{HashMap, HashSet};

use crate::{
    ast::{BinaryOp, Expr, ForSource, Program, Stmt, UnaryOp},
    diagnostic::Diagnostic,
    symbol_table::SymbolTable,
    types::Type,
};

/// A successfully checked program and the semantic metadata needed downstream.
#[derive(Debug, Clone)]
pub struct CheckedProgram {
    /// The original parsed program.
    pub program: Program,
    /// Top-level declarations discovered during semantic analysis.
    pub symbols: SymbolTable,
    /// Byte spans of unresolved identifiers that should be treated as emoji
    /// literals instead of variables.
    pub emoji_literals: HashSet<(usize, usize)>,
}

/// Validate declarations and infer static types for a parsed Peps program.
///
/// On success this returns the original program with a populated symbol table
/// and the unresolved identifier spans that the compiler should lower as emoji
/// literals. On failure it returns every diagnostic collected while walking the
/// tree.
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
    /// Top-level variables declared with assignment statements.
    symbols: SymbolTable,
    /// Block-local bindings, currently used for loop variables.
    local_scopes: Vec<HashMap<String, Type>>,
    /// Semantic errors collected during the pass.
    diagnostics: Vec<Diagnostic>,
    /// Unresolved identifier spans that are valid emoji literal expressions.
    emoji_literals: HashSet<(usize, usize)>,
}

impl Checker {
    /// Check one statement and recursively walk any nested block it owns.
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
            Stmt::Append { name, expr, span } => {
                self.check_append_statement(name, expr, *span);
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

    /// Reject loop variables that would shadow visible bindings.
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

    /// Infer the item type produced by a `for` source.
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

    /// Ensure a control-flow condition is boolean.
    fn check_condition(&mut self, condition: &Expr, kind: &str, span: crate::source::Span) {
        if let Some(ty) = self.infer_expr(condition) {
            if ty != Type::Bool {
                self.diagnostics
                    .push(Diagnostic::at(format!("{} condition must be bool", kind), span));
            }
        }
    }

    /// Validate append-assignment syntax against an existing list variable.
    fn check_append_statement(&mut self, name: &str, expr: &Expr, span: crate::source::Span) {
        let Some(target_ty) = self.lookup(name).cloned() else {
            self.diagnostics.push(Diagnostic::at(
                format!("list append target {} is not declared", name),
                span,
            ));
            return;
        };

        let Type::List(element_type) = target_ty else {
            self.diagnostics.push(Diagnostic::at(
                "list append requires a list variable on the left",
                span,
            ));
            return;
        };

        self.check_append_rhs(expr, &element_type, span);
    }

    /// Infer an assignment right-hand side using declaration-only literal rules.
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
            _ => self.infer_expr(expr),
        }
    }

    /// Infer an expression type under normal expression rules.
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
                match op {
                    UnaryOp::Negate => {
                        let ty = self.infer_expr(expr)?;
                        if ty == Type::Num {
                            Some(Type::Num)
                        } else {
                            self.diagnostics.push(Diagnostic::at(
                                "numeric negation requires a num operand",
                                *span,
                            ));
                            None
                        }
                    }
                    UnaryOp::Not => {
                        let ty = self.infer_expr(expr)?;
                        if ty == Type::Bool {
                            Some(Type::Bool)
                        } else {
                            self.diagnostics.push(Diagnostic::at(
                                "logical not requires a bool operand",
                                *span,
                            ));
                            None
                        }
                    }
                    UnaryOp::Len => {
                        let ty = self.infer_expr(expr)?;
                        if matches!(ty, Type::List(_)) {
                            Some(Type::Num)
                        } else {
                            self.diagnostics.push(Diagnostic::at(
                                "list length requires a list operand",
                                *span,
                            ));
                            None
                        }
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
        match op {
            BinaryOp::Index => {
                let left_ty = self.infer_expr(left)?;
                let right_ty = self.infer_expr(right)?;
                match (left_ty, right_ty) {
                    (Type::List(element_type), Type::Num) => Some(*element_type),
                    (Type::List(_), _) => {
                        self.diagnostics.push(Diagnostic::at(
                            "list index requires a num index",
                            span,
                        ));
                        None
                    }
                    _ => {
                        self.diagnostics.push(Diagnostic::at(
                            "list index requires a list value on the left",
                            span,
                        ));
                        None
                    }
                }
            }
            BinaryOp::Append => {
                let left_ty = self.infer_expr(left)?;
                let Type::List(element_type) = left_ty else {
                    self.diagnostics.push(Diagnostic::at(
                        "list append requires a list value on the left",
                        span,
                    ));
                    return None;
                };
                let expected_type = (*element_type).clone();
                self.check_append_rhs(right, &expected_type, span)?;
                Some(Type::List(Box::new(expected_type)))
            }
            BinaryOp::And | BinaryOp::Or => {
                let left_ty = self.infer_expr(left)?;
                let right_ty = self.infer_expr(right)?;
                if left_ty == Type::Bool && right_ty == Type::Bool {
                    Some(Type::Bool)
                } else {
                    self.diagnostics.push(Diagnostic::at(
                        format!("logical operator {} requires bool operands", op_symbol(op)),
                        span,
                    ));
                    None
                }
            }
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                let left_ty = if op == BinaryOp::Add {
                    self.infer_expr_allow_raw_strings(left)?
                } else {
                    self.infer_expr(left)?
                };
                let right_ty = if op == BinaryOp::Add {
                    self.infer_expr_allow_raw_strings(right)?
                } else {
                    self.infer_expr(right)?
                };
                if left_ty == Type::Num && right_ty == Type::Num {
                    Some(Type::Num)
                } else if op == BinaryOp::Add && left_ty == Type::Str && right_ty == Type::Str {
                    Some(Type::Str)
                } else if op == BinaryOp::Add
                    && (left_ty == Type::Str || right_ty == Type::Str)
                {
                    self.diagnostics.push(Diagnostic::at(
                        "string concatenation requires both operands to be text",
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
                let left_ty = self.infer_expr(left)?;
                let right_ty = self.infer_expr(right)?;
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
                let left_ty = self.infer_expr(left)?;
                let right_ty = self.infer_expr(right)?;
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

    /// Infer a homogeneous list type and reject unsupported list shapes.
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

    /// Infer an expression type in a context where raw string literals are legal.
    fn infer_expr_allow_raw_strings(&mut self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::String { .. } => Some(Type::Str),
            Expr::Number { .. } => Some(Type::Num),
            Expr::Bool { .. } => Some(Type::Bool),
            Expr::Emoji { .. } => Some(Type::Emoji),
            Expr::Variable { name, span } => match self.lookup(name) {
                Some(ty) => Some(ty.clone()),
                None => {
                    self.emoji_literals.insert((span.start, span.end));
                    Some(Type::Emoji)
                }
            },
            Expr::List { elements, span } => self.infer_list(elements, *span, true, false),
            Expr::Unary { op, expr, span } => match op {
                UnaryOp::Negate => {
                    let ty = self.infer_expr_allow_raw_strings(expr)?;
                    if ty == Type::Num {
                        Some(Type::Num)
                    } else {
                        self.diagnostics.push(Diagnostic::at(
                            "numeric negation requires a num operand",
                            *span,
                        ));
                        None
                    }
                }
                UnaryOp::Not => {
                    let ty = self.infer_expr_allow_raw_strings(expr)?;
                    if ty == Type::Bool {
                        Some(Type::Bool)
                    } else {
                        self.diagnostics.push(Diagnostic::at(
                            "logical not requires a bool operand",
                            *span,
                        ));
                        None
                    }
                }
                UnaryOp::Len => {
                    let ty = self.infer_expr_allow_raw_strings(expr)?;
                    if matches!(ty, Type::List(_)) {
                        Some(Type::Num)
                    } else {
                        self.diagnostics.push(Diagnostic::at(
                            "list length requires a list operand",
                            *span,
                        ));
                        None
                    }
                }
            },
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => self.infer_binary(left, *op, right, *span),
        }
    }

    fn check_append_rhs(
        &mut self,
        expr: &Expr,
        expected_type: &Type,
        span: crate::source::Span,
    ) -> Option<()> {
        let right_ty = self.infer_append_rhs_type(expr, expected_type)?;
        let expected_list_type = Type::List(Box::new(expected_type.clone()));

        if right_ty == *expected_type || right_ty == expected_list_type {
            Some(())
        } else {
            self.diagnostics.push(Diagnostic::at(
                "list append requires a value or list matching the list element type",
                span,
            ));
            None
        }
    }

    /// Infer append RHS type, applying string-list literal exceptions when needed.
    fn infer_append_rhs_type(&mut self, expr: &Expr, expected_type: &Type) -> Option<Type> {
        match (expected_type, expr) {
            (Type::Str, Expr::String { .. }) => Some(Type::Str),
            (
                Type::Str,
                Expr::List {
                    elements,
                    span,
                },
            ) => self.infer_list(elements, *span, true, true),
            _ => self.infer_expr(expr),
        }
    }

    /// Look up a name in local scopes first, then top-level declarations.
    fn lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.local_scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        self.symbols.get(name)
    }

    /// Start a new block-local scope.
    fn push_scope(&mut self) {
        self.local_scopes.push(HashMap::new());
    }

    /// End the innermost block-local scope.
    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    /// Insert a binding into the innermost local scope.
    fn insert_local(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(name, ty);
        }
    }
}

/// Return the display symbol used in diagnostics for a binary operator.
fn op_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "➕",
        BinaryOp::Sub => "➖",
        BinaryOp::Mul => "✖️",
        BinaryOp::Div => "➗",
        BinaryOp::Append => "📥",
        BinaryOp::And => "🤝",
        BinaryOp::Or => "🔀",
        BinaryOp::Index => "🔎",
        BinaryOp::Eq => "🟰🟰",
        BinaryOp::NotEq => "❌🟰",
        BinaryOp::Lt => "◀️",
        BinaryOp::Gt => "▶️",
        BinaryOp::LtEq => "◀️🟰",
        BinaryOp::GtEq => "▶️🟰",
    }
}
