//! Parser for Peps token streams.
//!
//! Statements are parsed with recursive descent, while expressions use
//! precedence climbing so binary operators associate correctly without a large
//! expression grammar. The parser owns language-shape checks that depend on
//! token context, such as rejecting ASCII or multi-emoji identifiers and
//! allowing `🛑` / `⏭️` only inside loop bodies.
//!
//! Blocks are delimited with `🔓` and `🔒`. Statement separators may be explicit
//! `🔚` tokens or newlines that the lexer has already normalized into
//! [`TokenKind::StatementEnd`].

use crate::{
    ast::{BinaryOp, Expr, ForSource, Program, Stmt, UnaryOp},
    diagnostic::Diagnostic,
    source::Span,
    token::{Token, TokenKind},
};
use unicode_segmentation::UnicodeSegmentation;

/// Build an abstract syntax tree from lexer tokens.
///
/// The input must include the trailing [`TokenKind::Eof`] token produced by the
/// lexer. Parsing currently stops on the first syntax error and wraps it in a
/// diagnostic vector to match the rest of the compiler pipeline.
pub fn parse(tokens: Vec<Token>) -> Result<Program, Vec<Diagnostic>> {
    Parser::new(tokens).parse_program().map_err(|err| vec![err])
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
    /// Nesting count used to validate `break` and `continue` placement.
    loop_depth: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            loop_depth: 0,
        }
    }

    fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let start = self.peek().span;
        let mut statements = Vec::new();

        while !self.is_at_end() {
            self.skip_statement_separators();
            if self.is_at_end() {
                break;
            }
            if matches!(self.peek().kind, TokenKind::BlockEnd) {
                return Err(Diagnostic::at("unexpected block end 🔒", self.peek().span));
            }
            statements.push(self.parse_statement()?);
        }

        let end = self.peek().span;
        Ok(Program {
            statements,
            span: start.merge(end),
        })
    }

    fn parse_statement(&mut self) -> Result<Stmt, Diagnostic> {
        match &self.peek().kind {
            TokenKind::Identifier(_) => self.parse_assignment(),
            TokenKind::Print => self.parse_print(),
            TokenKind::Break => self.parse_break(),
            TokenKind::Continue => self.parse_continue(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_loop(),
            TokenKind::Else => Err(Diagnostic::at("else 😐 without matching if", self.peek().span)),
            TokenKind::Eof => Err(Diagnostic::at("unexpected end of file", self.peek().span)),
            _ => Err(Diagnostic::at("expected statement", self.peek().span)),
        }
    }

    fn parse_assignment(&mut self) -> Result<Stmt, Diagnostic> {
        let name_token = self.advance().clone();
        let TokenKind::Identifier(name) = name_token.kind else {
            unreachable!("parse_assignment called only for identifiers");
        };

        if !is_single_emoji_identifier(&name) || matches!(self.peek().kind, TokenKind::Identifier(_)) {
            return Err(Diagnostic::at(
                "variable identifiers must be exactly one emoji long",
                name_token.span.merge(self.peek().span),
            ));
        }

        self.expect_assign()?;
        let expr = self.parse_expression(0)?;
        let end = self.expect_statement_end()?;

        Ok(Stmt::Assign {
            name,
            span: name_token.span.merge(end),
            expr,
        })
    }

    fn parse_print(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.advance().span;
        let expr = self.parse_expression(0)?;
        let end = self.expect_statement_end()?;

        Ok(Stmt::Print {
            expr,
            span: start.merge(end),
        })
    }

    fn parse_break(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.advance().span;
        if self.loop_depth == 0 {
            return Err(Diagnostic::at("break can only be used inside loops", start));
        }
        let end = self.expect_statement_end()?;
        Ok(Stmt::Break {
            span: start.merge(end),
        })
    }

    fn parse_continue(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.advance().span;
        if self.loop_depth == 0 {
            return Err(Diagnostic::at("continue can only be used inside loops", start));
        }
        let end = self.expect_statement_end()?;
        Ok(Stmt::Continue {
            span: start.merge(end),
        })
    }

    fn parse_if(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.advance().span;
        let condition = self.parse_expression(0)?;
        let then_branch = self.parse_block()?;
        let mut span = start.merge(self.previous().span);
        self.skip_statement_separators();

        let else_branch = if matches!(self.peek().kind, TokenKind::Else) {
            self.advance();
            self.expect_block_start()?;
            let branch = self.parse_block_body()?;
            span = span.merge(self.previous().span);
            Some(branch)
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    fn parse_loop(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.advance().span;

        // `🔁 <identifier> 🧭 ...` is a for loop; every other `🔁` starts a
        // while loop whose condition begins immediately after the loop token.
        if matches!(self.peek().kind, TokenKind::Identifier(_))
            && matches!(self.peek_next_kind(), Some(TokenKind::In))
        {
            return self.parse_for(start);
        }

        let condition = self.parse_expression(0)?;
        let body = self.parse_loop_block()?;
        let span = start.merge(self.previous().span);

        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn parse_for(&mut self, start: Span) -> Result<Stmt, Diagnostic> {
        let variable_token = self.advance().clone();
        let TokenKind::Identifier(variable) = variable_token.kind else {
            unreachable!("parse_for called only after seeing an identifier");
        };
        if !is_single_emoji_identifier(&variable) {
            return Err(Diagnostic::at(
                "variable identifiers must be exactly one emoji long",
                variable_token.span,
            ));
        }

        self.expect_in()?;
        let source = self.parse_for_source()?;
        let body = self.parse_loop_block()?;
        let span = start.merge(self.previous().span);

        Ok(Stmt::For {
            variable,
            source,
            body,
            span,
        })
    }

    fn parse_for_source(&mut self) -> Result<ForSource, Diagnostic> {
        // A `🔢` marker selects range syntax. Otherwise the source is a normal
        // expression, usually a list variable or list literal.
        if matches!(self.peek().kind, TokenKind::Range) {
            let start_token = self.advance().span;
            let start = self.parse_expression(0)?;
            self.expect_arrow()?;
            let end = self.parse_expression(0)?;
            let span = start_token.merge(end.span());
            Ok(ForSource::Range { start, end, span })
        } else {
            let expr = self.parse_expression(0)?;
            let span = expr.span();
            Ok(ForSource::List { expr, span })
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        self.expect_block_start()?;
        self.parse_block_body()
    }

    fn parse_loop_block(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        // Increment before parsing the required block so nested loops and loop
        // control statements are validated against the current context.
        self.loop_depth += 1;
        let result = self.parse_block();
        self.loop_depth -= 1;
        result
    }

    fn parse_block_body(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        let mut statements = Vec::new();

        loop {
            self.skip_statement_separators();
            if matches!(self.peek().kind, TokenKind::BlockEnd | TokenKind::Eof) {
                break;
            }
            statements.push(self.parse_statement()?);
        }

        if matches!(self.peek().kind, TokenKind::Eof) {
            return Err(Diagnostic::at("missing block end 🔒", self.peek().span));
        }

        self.advance();
        Ok(statements)
    }

    fn parse_expression(&mut self, min_precedence: u8) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_unary()?;

        while let Some((op, precedence)) = self.current_binary_op() {
            if precedence < min_precedence {
                break;
            }

            self.advance();
            let right = self.parse_expression(precedence + 1)?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, Diagnostic> {
        if matches!(self.peek().kind, TokenKind::Minus) {
            let op_span = self.advance().span;
            let expr = self.parse_unary()?;
            let span = op_span.merge(expr.span());
            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(expr),
                span,
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Number(value) => Ok(Expr::Number {
                value,
                span: token.span,
            }),
            TokenKind::StringLiteral(value) => Ok(Expr::String {
                value,
                span: token.span,
            }),
            TokenKind::Bool(value) => Ok(Expr::Bool {
                value,
                span: token.span,
            }),
            TokenKind::Identifier(name) => {
                if !is_single_emoji_identifier(&name) {
                    Err(Diagnostic::at(
                        "variable identifiers must be exactly one emoji long",
                        token.span,
                    ))
                } else {
                    Ok(Expr::Variable {
                        name,
                        span: token.span,
                    })
                }
            }
            TokenKind::ListDelimiter => self.parse_list(token.span),
            _ => Err(Diagnostic::at("expected expression", token.span)),
        }
    }

    fn parse_list(&mut self, start: Span) -> Result<Expr, Diagnostic> {
        let mut elements = Vec::new();

        while !matches!(self.peek().kind, TokenKind::Eof) {
            self.skip_statement_separators();
            // `📚` closes the list unless it is the first delimiter in a nested
            // list expression like `📚 📚 1️⃣ 📚 📚`.
            if matches!(self.peek().kind, TokenKind::ListDelimiter)
                && !(elements.is_empty() && self.next_token_starts_expression())
            {
                break;
            }
            elements.push(self.parse_expression(0)?);
        }

        if matches!(self.peek().kind, TokenKind::Eof) {
            return Err(Diagnostic::at("missing closing list delimiter 📚", self.peek().span));
        }

        let end = self.advance().span;
        Ok(Expr::List {
            elements,
            span: start.merge(end),
        })
    }

    fn current_binary_op(&self) -> Option<(BinaryOp, u8)> {
        // Higher numbers bind tighter. Equal-precedence operators are
        // left-associative because the recursive call uses `precedence + 1`.
        match self.peek().kind {
            TokenKind::Eq => Some((BinaryOp::Eq, 1)),
            TokenKind::NotEq => Some((BinaryOp::NotEq, 1)),
            TokenKind::Lt => Some((BinaryOp::Lt, 1)),
            TokenKind::Gt => Some((BinaryOp::Gt, 1)),
            TokenKind::LtEq => Some((BinaryOp::LtEq, 1)),
            TokenKind::GtEq => Some((BinaryOp::GtEq, 1)),
            TokenKind::Plus => Some((BinaryOp::Add, 2)),
            TokenKind::Minus => Some((BinaryOp::Sub, 2)),
            TokenKind::Star => Some((BinaryOp::Mul, 3)),
            TokenKind::Slash => Some((BinaryOp::Div, 3)),
            _ => None,
        }
    }

    fn peek_next_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.current + 1).map(|token| &token.kind)
    }

    fn next_token_starts_expression(&self) -> bool {
        self.tokens
            .get(self.current + 1)
            .map(|token| {
                matches!(
                    token.kind,
                    TokenKind::Identifier(_)
                        | TokenKind::Number(_)
                        | TokenKind::StringLiteral(_)
                        | TokenKind::Bool(_)
                        | TokenKind::Minus
                        | TokenKind::ListDelimiter
                )
            })
            .unwrap_or(false)
    }

    fn expect_assign(&mut self) -> Result<Span, Diagnostic> {
        if matches!(self.peek().kind, TokenKind::Assign) {
            Ok(self.advance().span)
        } else {
            Err(Diagnostic::at("expected assignment operator 🟰", self.peek().span))
        }
    }

    fn expect_in(&mut self) -> Result<Span, Diagnostic> {
        if matches!(self.peek().kind, TokenKind::In) {
            Ok(self.advance().span)
        } else {
            Err(Diagnostic::at("expected loop in marker 🧭", self.peek().span))
        }
    }

    fn expect_arrow(&mut self) -> Result<Span, Diagnostic> {
        if matches!(self.peek().kind, TokenKind::Arrow) {
            Ok(self.advance().span)
        } else {
            Err(Diagnostic::at("expected range end arrow ➡️", self.peek().span))
        }
    }

    fn expect_statement_end(&mut self) -> Result<Span, Diagnostic> {
        if matches!(self.peek().kind, TokenKind::Identifier(_)) {
            return Err(Diagnostic::at(
                "variable identifiers must be exactly one emoji long",
                self.peek().span,
            ));
        }

        if matches!(self.peek().kind, TokenKind::StatementEnd) {
            let end = self.advance().span;
            self.skip_statement_separators();
            Ok(end)
        } else if matches!(self.peek().kind, TokenKind::BlockEnd | TokenKind::Eof | TokenKind::Else) {
            Ok(self.previous().span)
        } else {
            Err(Diagnostic::at(
                "missing statement separator (newline or 🔚)",
                self.peek().span,
            ))
        }
    }

    fn skip_statement_separators(&mut self) {
        while matches!(self.peek().kind, TokenKind::StatementEnd) {
            self.advance();
        }
    }

    fn expect_block_start(&mut self) -> Result<Span, Diagnostic> {
        if matches!(self.peek().kind, TokenKind::BlockStart) {
            Ok(self.advance().span)
        } else {
            Err(Diagnostic::at("expected block start 🔓", self.peek().span))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
            self.previous()
        } else {
            self.peek()
        }
    }
}

fn is_single_emoji_identifier(name: &str) -> bool {
    if name.chars().any(|ch| ch.is_ascii()) {
        return false;
    }
    name.graphemes(true).count() == 1
}
