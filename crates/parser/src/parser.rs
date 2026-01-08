//! The Nevermind parser (recursive descent)

use std::iter::Peekable;
use std::vec::IntoIter;

use nevermind_common::Span;

use nevermind_lexer::{Lexer, Token, TokenType};
use nevermind_lexer::token::{Keyword, Operator, Delimiter, LiteralType};

use nevermind_ast::{Expr, Stmt, Pattern, TypeAnnotation, Parameter};
use nevermind_ast::stmt::MatchArm;
use nevermind_ast::types::{Type, PrimitiveType};
use nevermind_ast::op::{BinaryOp, ComparisonOp, UnaryOp, LogicalOp};

use super::error::{ParseError, ParseResult};
use super::expr_parser::ExprParser;

/// The Nevermind parser
pub struct Parser {
    /// The tokens to parse
    tokens: Peekable<IntoIter<Token>>,

    /// Current token
    pub current: Option<Token>,

    /// Previous token
    pub previous: Option<Token>,
}

impl Parser {
    /// Create a new parser from a source string
    pub fn new(source: &str) -> ParseResult<Self> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| {
            ParseError::new(e.message, e.span)
        })?;

        Ok(Self::from_tokens(tokens))
    }

    /// Create a new parser from tokens
    pub fn from_tokens(tokens: Vec<Token>) -> Self {
        let mut tokens = tokens.into_iter().peekable();
        let current = tokens.next();

        Self {
            tokens,
            current,
            previous: None,
        }
    }

    /// Parse a source file
    pub fn parse(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmt) = self.parse_statement()? {
                statements.push(stmt);
            }
        }

        Ok(statements)
    }

    /// Parse a statement
    pub fn parse_statement(&mut self) -> ParseResult<Option<Stmt>> {
        // Skip empty lines (semicolon tokens from indentation)
        while self.match_delimiter(Delimiter::Semicolon) {
            continue;
        }

        if self.is_at_end() {
            return Ok(None);
        }

        let stmt = match self.peek_token_type() {
            TokenType::Keyword(Keyword::Let) | TokenType::Keyword(Keyword::Var) => {
                self.parse_let_statement()?
            }
            TokenType::Keyword(Keyword::Fn) => {
                self.parse_function_statement()?
            }
            TokenType::Keyword(Keyword::If) => {
                // Check if this is an if-expression (then...else...end) or if-statement (do...end)
                // We peek ahead to see what comes after the condition
                // For simplicity, we'll try both approaches
                self.parse_if_or_expr_statement()?
            }
            TokenType::Keyword(Keyword::While) => {
                self.parse_while_statement()?
            }
            TokenType::Keyword(Keyword::For) => {
                self.parse_for_statement()?
            }
            TokenType::Keyword(Keyword::Match) => {
                self.parse_match_statement()?
            }
            TokenType::Keyword(Keyword::Return) => {
                self.parse_return_statement()?
            }
            TokenType::Keyword(Keyword::Break) => {
                self.parse_break_statement()?
            }
            TokenType::Keyword(Keyword::Continue) => {
                self.parse_continue_statement()?
            }
            TokenType::Keyword(Keyword::Type) => {
                self.parse_type_alias_statement()?
            }
            TokenType::Keyword(Keyword::Use) | TokenType::Keyword(Keyword::From) => {
                self.parse_import_statement()?
            }
            TokenType::Keyword(Keyword::Class) => {
                self.parse_class_statement()?
            }
            _ => {
                // Expression statement
                let expr = self.parse_expression()?;
                Some(Stmt::ExprStmt {
                    id: nevermind_ast::new_node_id(),
                    expr,
                    span: self.previous_span(),
                })
            }
        };

        Ok(stmt)
    }

    /// Parse an if statement or if expression
    /// This handles both:
    /// - if-statement: if cond do ... end else do ... end end
    /// - if-expression: if cond then expr else expr end
    pub fn parse_if_or_expr_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        self.consume_keyword(Keyword::If, "expected 'if'")?;

        // Parse the condition
        let condition = self.parse_expression()?;

        // Check what comes next
        if self.match_keyword(Keyword::Then) {
            // This is an if-expression
            let then_branch = self.parse_expression()?;
            self.consume_keyword(Keyword::Else, "expected 'else' in if expression")?;
            let else_branch = self.parse_expression()?;
            self.consume_keyword(Keyword::End, "expected 'end' to close if expression")?;

            let span = self.span_from(start);

            Ok(Some(Stmt::ExprStmt {
                id: nevermind_ast::new_node_id(),
                expr: Expr::If {
                    id: nevermind_ast::new_node_id(),
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                    span: span.clone(),
                },
                span,
            }))
        } else if self.match_keyword(Keyword::Do) {
            // This is an if-statement
            let mut then_branch = Vec::new();
            while !self.check_keyword(Keyword::End) && !self.is_at_end() {
                if let Some(stmt) = self.parse_statement()? {
                    then_branch.push(stmt);
                }
            }
            self.consume_keyword(Keyword::End, "expected 'end' to close then block")?;

            let mut else_branch = None;
            if self.match_keyword(Keyword::Else) {
                let mut stmts = Vec::new();
                if self.match_keyword(Keyword::Do) {
                    while !self.check_keyword(Keyword::End) && !self.is_at_end() {
                        if let Some(stmt) = self.parse_statement()? {
                            stmts.push(stmt);
                        }
                    }
                    self.consume_keyword(Keyword::End, "expected 'end' to close else block")?;
                    else_branch = Some(stmts);
                } else if self.check_keyword(Keyword::If) {
                    // else if
                    if let Some(stmt) = self.parse_if_or_expr_statement()? {
                        stmts.push(stmt);
                    }
                    else_branch = Some(stmts);
                } else {
                    // Single statement else
                    if let Some(stmt) = self.parse_statement()? {
                        stmts.push(stmt);
                    }
                    else_branch = Some(stmts);
                }
            }

            self.consume_keyword(Keyword::End, "expected 'end' to close if statement")?;

            let span = self.span_from(start);

            Ok(Some(Stmt::If {
                id: nevermind_ast::new_node_id(),
                condition,
                then_branch,
                else_branch,
                span,
            }))
        } else {
            Err(ParseError::new(
                "expected 'then' or 'do' after if condition",
                self.peek_span(),
            ))
        }
    }

    /// Parse a let/var statement
    pub fn parse_let_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        let is_mutable = if self.match_keyword(Keyword::Var) {
            true
        } else if self.match_keyword(Keyword::Let) {
            false
        } else {
            return Ok(None);
        };

        let name = self.consume_identifier("expected variable name")?;

        let type_annotation = if self.match_delimiter(Delimiter::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.consume_operator(Operator::Assign, "expected '=' after let binding")?;

        let value = self.parse_expression()?;

        let span = self.span_from(start);

        Ok(Some(Stmt::Let {
            id: nevermind_ast::new_node_id(),
            is_mutable,
            name,
            type_annotation,
            value,
            span,
        }))
    }

    /// Parse a function declaration
    pub fn parse_function_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        self.consume_keyword(Keyword::Fn, "expected 'fn'")?;

        let name = self.consume_identifier("expected function name")?;

        let params = self.parse_parameters()?;

        let return_type = if self.match_delimiter(Delimiter::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let body = self.parse_expression()?;

        self.consume_keyword(Keyword::End, "expected 'end' to close function declaration")?;

        let span = self.span_from(start);

        Ok(Some(Stmt::Function {
            id: nevermind_ast::new_node_id(),
            name,
            params,
            return_type,
            body,
            span,
        }))
    }

    /// Parse function parameters
    pub fn parse_parameters(&mut self) -> ParseResult<Vec<Parameter>> {
        self.consume_delimiter(Delimiter::LParen, "expected '(' before parameters")?;

        let mut params = Vec::new();

        if !self.check_delimiter(Delimiter::RParen) {
            loop {
                let name = self.consume_identifier("expected parameter name")?;

                let type_annotation = if self.match_delimiter(Delimiter::Colon) {
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };

                let default_value = if self.match_operator(Operator::Assign) {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };

                params.push(Parameter {
                    id: nevermind_ast::new_node_id(),
                    name,
                    type_annotation,
                    default_value,
                });

                if !self.match_delimiter(Delimiter::Comma) {
                    break;
                }
            }
        }

        self.consume_delimiter(Delimiter::RParen, "expected ')' after parameters")?;

        Ok(params)
    }

    /// Parse an if statement
    pub fn parse_if_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        self.consume_keyword(Keyword::If, "expected 'if'")?;

        let condition = self.parse_expression()?;

        let then_branch = if self.check_keyword(Keyword::Do) {
            self.consume_keyword(Keyword::Do, "expected 'do'")?;
            let mut stmts = Vec::new();
            while !self.check_keyword(Keyword::End) && !self.is_at_end() {
                if let Some(stmt) = self.parse_statement()? {
                    stmts.push(stmt);
                }
            }
            self.consume_keyword(Keyword::End, "expected 'end' to close 'do' block")?;
            stmts
        } else {
            vec![Stmt::ExprStmt {
                id: nevermind_ast::new_node_id(),
                expr: self.parse_expression()?,
                span: self.previous_span(),
            }]
        };

        let mut else_branch = None;
        if self.match_keyword(Keyword::Else) {
            if self.check_keyword(Keyword::If) {
                // else if
                if let Some(stmt) = self.parse_if_statement()? {
                    else_branch = Some(vec![stmt]);
                }
            } else {
                // else block
                let mut stmts = Vec::new();
                if self.match_keyword(Keyword::Do) {
                    while !self.check_keyword(Keyword::End) && !self.is_at_end() {
                        if let Some(stmt) = self.parse_statement()? {
                            stmts.push(stmt);
                        }
                    }
                    self.consume_keyword(Keyword::End, "expected 'end' to close 'else' block")?;
                    else_branch = Some(stmts);
                } else if let Some(expr) = self.parse_expression().ok() {
                    else_branch = Some(vec![Stmt::ExprStmt {
                        id: nevermind_ast::new_node_id(),
                        expr,
                        span: self.previous_span(),
                    }]);
                }
            }
        }

        let span = self.span_from(start);

        Ok(Some(Stmt::If {
            id: nevermind_ast::new_node_id(),
            condition,
            then_branch,
            else_branch,
            span,
        }))
    }

    /// Parse a while loop
    pub fn parse_while_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        self.consume_keyword(Keyword::While, "expected 'while'")?;

        let condition = self.parse_expression()?;

        let mut body = Vec::new();
        if self.match_keyword(Keyword::Do) {
            while !self.check_keyword(Keyword::End) && !self.is_at_end() {
                if let Some(stmt) = self.parse_statement()? {
                    body.push(stmt);
                }
            }
            self.consume_keyword(Keyword::End, "expected 'end' to close 'while' block")?;
        } else {
            body.push(Stmt::ExprStmt {
                id: nevermind_ast::new_node_id(),
                expr: self.parse_expression()?,
                span: self.previous_span(),
            });
        }

        let span = self.span_from(start);

        Ok(Some(Stmt::While {
            id: nevermind_ast::new_node_id(),
            condition,
            body,
            span,
        }))
    }

    /// Parse a for loop
    pub fn parse_for_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        self.consume_keyword(Keyword::For, "expected 'for'")?;

        let variable = self.parse_pattern()?;

        self.consume_keyword(Keyword::In, "expected 'in' after for loop variable")?;

        let iter = self.parse_expression()?;

        let mut body = Vec::new();
        if self.match_keyword(Keyword::Do) {
            while !self.check_keyword(Keyword::End) && !self.is_at_end() {
                if let Some(stmt) = self.parse_statement()? {
                    body.push(stmt);
                }
            }
            self.consume_keyword(Keyword::End, "expected 'end' to close 'for' block")?;
        } else {
            body.push(Stmt::ExprStmt {
                id: nevermind_ast::new_node_id(),
                expr: self.parse_expression()?,
                span: self.previous_span(),
            });
        }

        let span = self.span_from(start);

        Ok(Some(Stmt::For {
            id: nevermind_ast::new_node_id(),
            variable,
            iter,
            body,
            span,
        }))
    }

    /// Parse a match statement
    pub fn parse_match_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        self.consume_keyword(Keyword::Match, "expected 'match'")?;

        let scrutinee = self.parse_expression()?;

        self.consume_delimiter(Delimiter::LBrace, "expected '{' to start match arms")?;

        let mut arms = Vec::new();
        while !self.check_delimiter(Delimiter::RBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;

            let guard = if self.match_delimiter(Delimiter::Colon) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume_operator(Operator::FatArrow, "expected '=>' after match pattern")?;

            let body = self.parse_expression()?;

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            self.match_delimiter(Delimiter::Comma);
        }

        self.consume_delimiter(Delimiter::RBrace, "expected '}' to end match expression")?;

        let span = self.span_from(start);

        Ok(Some(Stmt::Match {
            id: nevermind_ast::new_node_id(),
            scrutinee,
            arms,
            span,
        }))
    }

    /// Parse a return statement
    pub fn parse_return_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        self.consume_keyword(Keyword::Return, "expected 'return'")?;

        let value = if !self.is_at_end() &&
            !self.check_delimiter(Delimiter::Semicolon) &&
            !self.check_delimiter(Delimiter::RBrace)
        {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let span = self.span_from(start);

        Ok(Some(Stmt::Return {
            id: nevermind_ast::new_node_id(),
            value,
            span,
        }))
    }

    /// Parse a break statement
    pub fn parse_break_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        self.consume_keyword(Keyword::Break, "expected 'break'")?;

        let span = self.span_from(start);

        Ok(Some(Stmt::Break {
            id: nevermind_ast::new_node_id(),
            span,
        }))
    }

    /// Parse a continue statement
    pub fn parse_continue_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        self.consume_keyword(Keyword::Continue, "expected 'continue'")?;

        let span = self.span_from(start);

        Ok(Some(Stmt::Continue {
            id: nevermind_ast::new_node_id(),
            span,
        }))
    }

    /// Parse a type alias statement
    pub fn parse_type_alias_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        self.consume_keyword(Keyword::Type, "expected 'type'")?;

        let name = self.consume_identifier("expected type name")?;

        let type_params = if self.match_delimiter(Delimiter::LBracket) {
            let mut params = Vec::new();
            while !self.check_delimiter(Delimiter::RBracket) && !self.is_at_end() {
                params.push(self.consume_identifier("expected type parameter name")?);
                if !self.match_delimiter(Delimiter::Comma) {
                    break;
                }
            }
            self.consume_delimiter(Delimiter::RBracket, "expected ']' after type parameters")?;
            params
        } else {
            Vec::new()
        };

        self.consume_operator(Operator::Assign, "expected '=' after type name")?;

        let definition = self.parse_type_annotation()?;

        let span = self.span_from(start);

        Ok(Some(Stmt::TypeAlias {
            id: nevermind_ast::new_node_id(),
            name,
            type_params,
            definition,
            span,
        }))
    }

    /// Parse an import statement
    pub fn parse_import_statement(&mut self) -> ParseResult<Option<Stmt>> {
        let start = self.peek_span();

        let (module, symbols) = if self.match_keyword(Keyword::From) {
            // from "module" import symbol1, symbol2
            let module = self.consume_string_literal("expected module name after 'from'")?;
            self.consume_keyword(Keyword::Import, "expected 'import' after module name")?;

            let mut symbols = Vec::new();
            if !self.is_at_end() {
                symbols.push(self.consume_identifier("expected symbol name")?);
                while self.match_delimiter(Delimiter::Comma) {
                    symbols.push(self.consume_identifier("expected symbol name")?);
                }
            }

            (module, Some(symbols))
        } else {
            // use "module"
            self.consume_keyword(Keyword::Use, "expected 'use' or 'from'")?;
            (self.consume_string_literal("expected module name")?, None)
        };

        let span = self.span_from(start);

        Ok(Some(Stmt::Import {
            id: nevermind_ast::new_node_id(),
            module,
            symbols,
            span,
        }))
    }

    /// Parse a class statement
    pub fn parse_class_statement(&mut self) -> ParseResult<Option<Stmt>> {
        // TODO: Implement full class parsing
        let start = self.peek_span();

        self.consume_keyword(Keyword::Class, "expected 'class'")?;

        let name = self.consume_identifier("expected class name")?;

        let extends = if self.match_keyword(Keyword::Extends) {
            Some(self.consume_identifier("expected superclass name")?)
        } else {
            None
        };

        self.consume_delimiter(Delimiter::LBrace, "expected '{' to start class body")?;

        let members = Vec::new();  // TODO: Parse members

        self.consume_delimiter(Delimiter::RBrace, "expected '}' to end class body")?;

        let span = self.span_from(start);

        Ok(Some(Stmt::Class {
            id: nevermind_ast::new_node_id(),
            name,
            extends,
            members,
            span,
        }))
    }

    /// Parse a type annotation
    pub fn parse_type_annotation(&mut self) -> ParseResult<TypeAnnotation> {
        let start = self.peek_span();

        // Simple identifier type
        let name = self.consume_identifier("expected type name")?;

        let kind = match name.as_str() {
            "Int" | "UInt" | "Int64" | "UInt64" | "Int32" | "UInt32" |
            "Float" | "Float64" | "Float32" | "Bool" | "String" | "Char" | "Unit" | "Null"
            => {
                let prim = match name.as_str() {
                    "Int" => PrimitiveType::Int,
                    "Float" => PrimitiveType::Float,
                    "Bool" => PrimitiveType::Bool,
                    "String" => PrimitiveType::String,
                    "Unit" => PrimitiveType::Unit,
                    "Null" => PrimitiveType::Null,
                    _ => PrimitiveType::Int,  // TODO: Add all primitives
                };
                Type::Primitive(prim)
            }
            "List" => {
                self.consume_delimiter(Delimiter::LBracket, "expected '[' after List")?;
                let inner = self.parse_type_annotation()?;
                self.consume_delimiter(Delimiter::RBracket, "expected ']' after List type")?;
                Type::List(Box::new(inner))
            }
            "Map" => {
                self.consume_delimiter(Delimiter::LBracket, "expected '[' after Map")?;
                let key = self.parse_type_annotation()?;
                self.consume_delimiter(Delimiter::Comma, "expected ',' in Map type")?;
                let value = self.parse_type_annotation()?;
                self.consume_delimiter(Delimiter::RBracket, "expected ']' after Map type")?;
                Type::Map {
                    key: Box::new(key),
                    value: Box::new(value),
                }
            }
            "Option" => {
                self.consume_delimiter(Delimiter::LBracket, "expected '[' after Option")?;
                let inner = self.parse_type_annotation()?;
                self.consume_delimiter(Delimiter::RBracket, "expected ']' after Option type")?;
                Type::Option(Box::new(inner))
            }
            "Result" => {
                self.consume_delimiter(Delimiter::LBracket, "expected '[' after Result")?;
                let ok = self.parse_type_annotation()?;
                self.consume_delimiter(Delimiter::Comma, "expected ',' in Result type")?;
                let error = self.parse_type_annotation()?;
                self.consume_delimiter(Delimiter::RBracket, "expected ']' after Result type")?;
                Type::Result {
                    ok: Box::new(ok),
                    error: Box::new(error),
                }
            }
            _ => Type::Identifier(name),
        };

        let span = self.span_from(start);

        Ok(TypeAnnotation {
            id: nevermind_ast::new_node_id(),
            span,
            kind,
        })
    }

    /// Parse a pattern
    pub fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        let start = self.peek_span();

        // Check for wildcard
        if self.match_delimiter(Delimiter::Dollar) {
            // Note: We'd need a delimiter for wildcard, using '_' as identifier
        }

        let name = self.consume_identifier("expected pattern")?;

        let pattern = if name == "_" {
            Pattern::Wildcard {
                span: self.span_from(start),
            }
        } else {
            Pattern::Variable {
                name,
                span: self.span_from(start),
            }
        };

        Ok(pattern)
    }

    /// Parse an expression (delegate to expression parser)
    pub fn parse_expression(&mut self) -> ParseResult<Expr> {
        ExprParser::new(self).parse_expression_bp(0)
    }

    // === Helper methods ===

    /// Check if we're at the end of input
    pub fn is_at_end(&self) -> bool {
        self.current.as_ref().map_or(true, |t| t.is_eof())
    }

    /// Get the current token type
    pub fn peek_token_type(&self) -> TokenType {
        self.current.as_ref()
            .map(|t| t.kind.clone())
            .unwrap_or(TokenType::EOF)
    }

    /// Get the span of the current token
    pub fn peek_span(&self) -> Span {
        self.current.as_ref()
            .map(|t| t.span.clone())
            .unwrap_or_else(|| Span::dummy())
    }

    /// Get the span of the previous token
    pub fn previous_span(&self) -> Span {
        self.previous.as_ref()
            .map(|t| t.span.clone())
            .unwrap_or_else(|| Span::dummy())
    }

    /// Check if current token is a keyword
    pub fn check_keyword(&self, keyword: Keyword) -> bool {
        matches!(self.peek_token_type(), TokenType::Keyword(kw) if kw == keyword)
    }

    /// Match and consume a keyword
    pub fn match_keyword(&mut self, keyword: Keyword) -> bool {
        if self.check_keyword(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume a keyword or error
    pub fn consume_keyword(&mut self, keyword: Keyword, message: &str) -> ParseResult<()> {
        if self.check_keyword(keyword) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(message, self.peek_span()))
        }
    }

    /// Check if current token is a delimiter
    pub fn check_delimiter(&self, delimiter: Delimiter) -> bool {
        matches!(self.peek_token_type(), TokenType::Delimiter(del) if del == delimiter)
    }

    /// Match and consume a delimiter
    pub fn match_delimiter(&mut self, delimiter: Delimiter) -> bool {
        if self.check_delimiter(delimiter) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume a delimiter or error
    pub fn consume_delimiter(&mut self, delimiter: Delimiter, message: &str) -> ParseResult<()> {
        if self.check_delimiter(delimiter) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(message, self.peek_span()))
        }
    }

    /// Check if current token is an operator
    pub fn check_operator(&self, operator: Operator) -> bool {
        matches!(self.peek_token_type(), TokenType::Operator(op) if op == operator)
    }

    /// Match and consume an operator
    pub fn match_operator(&mut self, operator: Operator) -> bool {
        if self.check_operator(operator) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume an operator or error
    pub fn consume_operator(&mut self, operator: Operator, message: &str) -> ParseResult<()> {
        if self.check_operator(operator) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(message, self.peek_span()))
        }
    }

    /// Consume an identifier or error
    pub fn consume_identifier(&mut self, message: &str) -> ParseResult<String> {
        if matches!(self.peek_token_type(), TokenType::Identifier) {
            let token = self.advance().unwrap();
            Ok(token.text)
        } else {
            Err(ParseError::new(message, self.peek_span()))
        }
    }

    /// Consume a string literal or error
    pub fn consume_string_literal(&mut self, message: &str) -> ParseResult<String> {
        if matches!(self.peek_token_type(), TokenType::Literal(LiteralType::String)) {
            let token = self.advance().unwrap();
            Ok(token.text)
        } else {
            Err(ParseError::new(message, self.peek_span()))
        }
    }

    /// Advance to the next token
    pub fn advance(&mut self) -> Option<Token> {
        self.previous = self.current.take();
        self.current = self.tokens.next();
        self.previous.clone()
    }

    /// Create a span from start to current position
    pub fn span_from(&self, start: Span) -> Span {
        Span::new(start.start.clone(), self.peek_span().end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_let_statement() {
        let source = "let x = 42";
        let mut parser = Parser::new(source).unwrap();
        let stmts = parser.parse().unwrap();

        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Let { name, .. } => assert_eq!(name, "x"),
            _ => panic!("expected Let statement"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let source = "if x > 0 then x else 0";
        let mut parser = Parser::new(source).unwrap();
        let stmts = parser.parse().unwrap();

        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::If { .. } => {}
            _ => panic!("expected If statement"),
        }
    }
}
