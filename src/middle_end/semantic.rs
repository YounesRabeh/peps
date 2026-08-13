//! Semantic checker for declaration rules, type inference, and lexical scopes.

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
        functions: HashMap::new(),
    };

    // Function signatures are available before any body or top-level call is
    // checked, which enables both forward calls and recursion.
    for statement in &program.statements {
        if let Stmt::Function {
            name,
            parameters,
            span,
            ..
        } = statement
        {
            if checker
                .functions
                .insert(name.clone(), parameters.len())
                .is_some()
            {
                checker.diagnostics.push(Diagnostic::at(
                    format!("function {} is already defined", name),
                    *span,
                ));
            }
        }
    }

    for statement in &program.statements {
        if !matches!(statement, Stmt::Function { .. }) {
            checker.check_statement(statement, 0);
        }
    }
    for statement in &program.statements {
        if let Stmt::Function {
            name,
            parameters,
            body,
            span,
        } = statement
        {
            checker.check_function(name, parameters, body, *span);
        }
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
    /// Bindings declared in nested lexical scopes.
    local_scopes: Vec<HashMap<String, Type>>,
    /// Semantic errors collected during the pass.
    diagnostics: Vec<Diagnostic>,
    /// Unresolved identifier spans that are valid emoji literal expressions.
    emoji_literals: HashSet<(usize, usize)>,
    /// Function arities collected before checking executable statements.
    functions: HashMap<String, usize>,
}

impl Checker {
    /// Check one statement and recursively walk any nested block it owns.
    fn check_statement(&mut self, statement: &Stmt, loop_depth: usize) {
        match statement {
            Stmt::Assign { name, expr, .. } => {
                if let Some(assigned_type) = self.infer_assignment_rhs(expr) {
                    if self.lookup(name).is_some() {
                        self.replace_visible_binding(name, assigned_type);
                    } else if self.local_scopes.is_empty() {
                        self.symbols.insert(name.clone(), assigned_type);
                    } else {
                        self.insert_local(name.clone(), assigned_type);
                    }
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
                    self.diagnostics.push(Diagnostic::at(
                        "continue can only be used inside loops",
                        *span,
                    ));
                }
            }
            Stmt::Function { span, .. } => self.diagnostics.push(Diagnostic::at(
                "function definitions are only allowed at the top level",
                *span,
            )),
            Stmt::Return { expr, .. } => {
                self.infer_expr(expr);
            }
            Stmt::Call { expr, .. } => {
                self.infer_expr(expr);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => {
                self.check_condition(condition, "if", *span);
                self.push_scope();
                for statement in then_branch {
                    self.check_statement(statement, loop_depth);
                }
                self.pop_scope();
                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    for statement in else_branch {
                        self.check_statement(statement, loop_depth);
                    }
                    self.pop_scope();
                }
            }
            Stmt::While {
                condition,
                body,
                span,
            } => {
                self.check_condition(condition, "while", *span);
                self.push_scope();
                for statement in body {
                    self.check_statement(statement, loop_depth + 1);
                }
                self.pop_scope();
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
                            self.check_statement(statement, loop_depth + 1);
                        }
                        self.pop_scope();
                    }
                }
            }
        }
    }

    fn check_function(
        &mut self,
        name: &str,
        parameters: &[String],
        body: &[Stmt],
        span: crate::source::Span,
    ) {
        // Reassignments are visible throughout this body, but checking one
        // function must not make another function's inferred global types
        // depend on source order. Runtime calls determine those mutations.
        let globals_before_body = self.symbols.clone();
        let mut seen = HashSet::new();
        self.push_scope();
        for parameter in parameters {
            if !seen.insert(parameter.clone()) {
                self.diagnostics.push(Diagnostic::at(
                    format!("function {} has duplicate parameter {}", name, parameter),
                    span,
                ));
            } else if self.symbols.get(parameter).is_some() {
                self.diagnostics.push(Diagnostic::at(
                    format!(
                        "parameter {} conflicts with a top-level variable",
                        parameter
                    ),
                    span,
                ));
            } else {
                self.insert_local(parameter.clone(), Type::Unknown);
            }
        }
        for statement in body {
            self.check_statement(statement, 0);
        }
        self.pop_scope();
        self.symbols = globals_before_body;

        if !statements_definitely_return(body) {
            self.diagnostics.push(Diagnostic::at(
                format!("function {} does not return a value on every path", name),
                span,
            ));
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
                Some(Type::Unknown) => Some(Type::Unknown),
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
                if matches!(start_type, Some(Type::Num | Type::Unknown))
                    && matches!(end_type, Some(Type::Num | Type::Unknown))
                {
                    Some(Type::Num)
                } else {
                    self.diagnostics
                        .push(Diagnostic::at("range bounds must be integers", *span));
                    None
                }
            }
        }
    }

    /// Ensure a control-flow condition is boolean.
    fn check_condition(&mut self, condition: &Expr, kind: &str, span: crate::source::Span) {
        if let Some(ty) = self.infer_expr(condition) {
            if !matches!(ty, Type::Bool | Type::Unknown) {
                self.diagnostics.push(Diagnostic::at(
                    format!("{} condition must be bool", kind),
                    span,
                ));
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

        if target_ty == Type::Unknown {
            self.infer_expr(expr);
            return;
        }
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
            Expr::Float { .. } => Some(Type::Float),
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
            Expr::Call {
                name,
                arguments,
                span,
            } => {
                for argument in arguments {
                    self.infer_expr(argument);
                }
                match self.functions.get(name) {
                    None => {
                        self.diagnostics.push(Diagnostic::at(
                            format!("function {} is not defined", name),
                            *span,
                        ));
                        None
                    }
                    Some(arity) if *arity != arguments.len() => {
                        self.diagnostics.push(Diagnostic::at(
                            format!(
                                "function {} expects {} arguments but received {}",
                                name,
                                arity,
                                arguments.len()
                            ),
                            *span,
                        ));
                        None
                    }
                    Some(_) => Some(Type::Unknown),
                }
            }
            Expr::Unary { op, expr, span } => match op {
                UnaryOp::Negate => {
                    let ty = self.infer_expr(expr)?;
                    if is_numeric(&ty) {
                        Some(ty)
                    } else {
                        self.diagnostics.push(Diagnostic::at(
                            "numeric negation requires a numeric operand",
                            *span,
                        ));
                        None
                    }
                }
                UnaryOp::Not => {
                    let ty = self.infer_expr(expr)?;
                    if matches!(ty, Type::Bool | Type::Unknown) {
                        Some(Type::Bool)
                    } else {
                        self.diagnostics
                            .push(Diagnostic::at("logical not requires a bool operand", *span));
                        None
                    }
                }
                UnaryOp::Len => {
                    let ty = self.infer_expr(expr)?;
                    if matches!(ty, Type::List(_) | Type::Unknown) {
                        Some(Type::Num)
                    } else {
                        self.diagnostics
                            .push(Diagnostic::at("list length requires a list operand", *span));
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
                    (Type::List(element_type), Type::Unknown) => Some(*element_type),
                    (Type::Unknown, Type::Num | Type::Unknown) => Some(Type::Unknown),
                    (Type::List(_), _) => {
                        self.diagnostics
                            .push(Diagnostic::at("list index requires an integer index", span));
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
                if left_ty == Type::Unknown {
                    self.infer_expr(right)?;
                    return Some(Type::Unknown);
                }
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
                if matches!(left_ty, Type::Bool | Type::Unknown)
                    && matches!(right_ty, Type::Bool | Type::Unknown)
                {
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
                if left_ty == Type::Unknown || right_ty == Type::Unknown {
                    Some(Type::Unknown)
                } else if is_numeric(&left_ty) && is_numeric(&right_ty) {
                    if left_ty == Type::Float || right_ty == Type::Float {
                        Some(Type::Float)
                    } else {
                        Some(Type::Num)
                    }
                } else if op == BinaryOp::Add && left_ty == Type::Str && right_ty == Type::Str {
                    Some(Type::Str)
                } else if op == BinaryOp::Add && (left_ty == Type::Str || right_ty == Type::Str) {
                    self.diagnostics.push(Diagnostic::at(
                        "string concatenation requires both operands to be text",
                        span,
                    ));
                    None
                } else {
                    self.diagnostics.push(Diagnostic::at(
                        format!(
                            "arithmetic operator {} requires numeric operands",
                            op_symbol(op)
                        ),
                        span,
                    ));
                    None
                }
            }
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                let left_ty = self.infer_expr(left)?;
                let right_ty = self.infer_expr(right)?;
                if is_numeric(&left_ty) && is_numeric(&right_ty) {
                    Some(Type::Bool)
                } else {
                    self.diagnostics.push(Diagnostic::at(
                        "ordering comparison requires numeric operands",
                        span,
                    ));
                    None
                }
            }
            BinaryOp::Eq | BinaryOp::NotEq => {
                let left_ty = self.infer_expr(left)?;
                let right_ty = self.infer_expr(right)?;
                if left_ty == Type::Unknown || right_ty == Type::Unknown {
                    Some(Type::Bool)
                } else if matches!(left_ty, Type::List(_)) || matches!(right_ty, Type::List(_)) {
                    self.diagnostics.push(Diagnostic::at(
                        "list equality is not supported in Peps v0",
                        span,
                    ));
                    None
                } else if left_ty == right_ty || (is_numeric(&left_ty) && is_numeric(&right_ty)) {
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
                if expected == &Type::Unknown && ty != Type::Unknown {
                    element_type = Some(ty);
                } else if ty != Type::Unknown && expected != &ty {
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
            Expr::Float { .. } => Some(Type::Float),
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
            Expr::Call { .. } => self.infer_expr(expr),
            Expr::Unary { op, expr, span } => match op {
                UnaryOp::Negate => {
                    let ty = self.infer_expr_allow_raw_strings(expr)?;
                    if is_numeric(&ty) {
                        Some(ty)
                    } else {
                        self.diagnostics.push(Diagnostic::at(
                            "numeric negation requires a numeric operand",
                            *span,
                        ));
                        None
                    }
                }
                UnaryOp::Not => {
                    let ty = self.infer_expr_allow_raw_strings(expr)?;
                    if matches!(ty, Type::Bool | Type::Unknown) {
                        Some(Type::Bool)
                    } else {
                        self.diagnostics
                            .push(Diagnostic::at("logical not requires a bool operand", *span));
                        None
                    }
                }
                UnaryOp::Len => {
                    let ty = self.infer_expr_allow_raw_strings(expr)?;
                    if matches!(ty, Type::List(_) | Type::Unknown) {
                        Some(Type::Num)
                    } else {
                        self.diagnostics
                            .push(Diagnostic::at("list length requires a list operand", *span));
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

        if *expected_type == Type::Unknown
            || right_ty == Type::Unknown
            || right_ty == *expected_type
            || right_ty == expected_list_type
        {
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
            (Type::Str, Expr::List { elements, span }) => {
                self.infer_list(elements, *span, true, true)
            }
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

    /// Replace the type of the nearest visible binding after reassignment.
    fn replace_visible_binding(&mut self, name: &str, ty: Type) {
        for scope in self.local_scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), ty);
                return;
            }
        }
        self.symbols.insert(name.to_string(), ty);
    }
}

/// Whether control flow through a sequence is guaranteed to encounter return.
fn statements_definitely_return(statements: &[Stmt]) -> bool {
    statements.iter().any(statement_definitely_returns)
}

fn statement_definitely_returns(statement: &Stmt) -> bool {
    match statement {
        Stmt::Return { .. } => true,
        Stmt::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => statements_definitely_return(then_branch) && statements_definitely_return(else_branch),
        // A loop may execute zero times (or never terminate), so it cannot by
        // itself prove that a function returns a value.
        _ => false,
    }
}

/// Whether a static type can participate in numeric operations.
fn is_numeric(ty: &Type) -> bool {
    matches!(ty, Type::Num | Type::Float | Type::Unknown)
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
