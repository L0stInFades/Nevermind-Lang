//! Expression parser using Pratt parsing

use nevermind_common::Span;

use nevermind_lexer::{Token, TokenType};
use nevermind_lexer::token::{Keyword, Operator, Delimiter, LiteralType};

use nevermind_ast::{Expr, Stmt, Parameter};
use nevermind_ast::expr::Literal;

use super::error::{ParseError, ParseResult};
use super::Parser;

/// Expression parser using Pratt parsing
pub struct ExprParser<'a> {
    /// Reference to the parent parser
    parser: &'a mut Parser,
}

impl<'a> ExprParser<'a> {
    /// Create a new expression parser
    pub fn new(parser: &'a mut Parser) -> Self {
        Self { parser }
    }

    /// Parse an expression with minimum binding power
    pub fn parse_expression_bp(&mut self, min_bp: u8) -> ParseResult<Expr> {
        let start = self.parser.peek_span();

        // Parse the left-hand side
        let mut lhs = self.parse_prefix()?;

        // Parse infix operators
        loop {
            let op_token = if let Some(token) = &self.parser.current {
                token.clone()
            } else {
                break;
            };

            let (left_bp, right_bp) = match self.get_binding_power(&op_token) {
                Some(bp) => bp,
                None => break,
            };

            if left_bp < min_bp {
                break;
            }

            self.parser.advance();

            lhs = self.parse_infix(lhs, op_token, right_bp, start)?;
        }

        Ok(lhs)
    }

    /// Parse a prefix expression
    fn parse_prefix(&mut self) -> ParseResult<Expr> {
        let start = self.parser.peek_span();

        let expr = match self.parser.peek_token_type() {
            TokenType::Literal(lit_type) => {
                let token = self.parser.advance().unwrap();
                let span = token.span.clone();
                let literal = match lit_type {
                    LiteralType::Integer => {
                        let value = token.text.parse::<i64>().unwrap_or(0);
                        Literal::Integer(value, span)
                    }
                    LiteralType::Float => {
                        let value = token.text.parse::<f64>().unwrap_or(0.0);
                        Literal::Float(value, span)
                    }
                    LiteralType::String => {
                        Literal::String(token.text, span)
                    }
                    LiteralType::Char => {
                        let c = token.text.chars().next().unwrap_or('\0');
                        Literal::Char(c, span)
                    }
                };

                Expr::Literal(literal)
            }

            TokenType::Identifier => {
                let token = self.parser.advance().unwrap();
                Expr::Variable {
                    id: nevermind_ast::new_node_id(),
                    name: token.text,
                    span: token.span,
                }
            }

            TokenType::Keyword(Keyword::True) => {
                let token = self.parser.advance().unwrap();
                Expr::Literal(Literal::Boolean(true, token.span))
            }

            TokenType::Keyword(Keyword::False) => {
                let token = self.parser.advance().unwrap();
                Expr::Literal(Literal::Boolean(false, token.span))
            }

            TokenType::Keyword(Keyword::Null) => {
                let token = self.parser.advance().unwrap();
                Expr::Literal(Literal::Null(token.span))
            }

            TokenType::Delimiter(Delimiter::LParen) => {
                self.parser.advance();
                let expr = self.parse_expression_bp(0)?;
                self.parser.consume_delimiter(Delimiter::RParen, "expected ')' after expression")?;
                expr
            }

            TokenType::Delimiter(Delimiter::LBracket) => {
                self.parser.advance();
                self.parse_list()?
            }

            TokenType::Delimiter(Delimiter::LBrace) => {
                self.parser.advance();
                self.parse_map()?
            }

            TokenType::Operator(Operator::Not) | TokenType::Operator(Operator::BitNot) | TokenType::Operator(Operator::Sub) => {
                let token = self.parser.advance().unwrap();
                let op = match token.kind {
                    TokenType::Operator(Operator::Not) => UnaryOp::Not,
                    TokenType::Operator(Operator::BitNot) => UnaryOp::BitNot,
                    TokenType::Operator(Operator::Sub) => UnaryOp::Neg,
                    _ => unreachable!(),
                };

                let expr = self.parse_expression_bp(14)?;  // Unary operators have high precedence

                Expr::Unary {
                    id: nevermind_ast::new_node_id(),
                    op,
                    expr: Box::new(expr),
                    span: Span::new(start, self.parser.previous_span()),
                }
            }

            TokenType::Delimiter(Delimiter::Pipe) => {
                // Lambda expression: |param1, param2| -> expr
                self.parser.advance();
                self.parse_lambda()?
            }

            TokenType::Keyword(Keyword::If) => {
                self.parser.advance();
                self.parse_if_expression()?
            }

            TokenType::Keyword(Keyword::Do) => {
                self.parser.advance();
                self.parse_block()?
            }

            TokenType::Keyword(Keyword::Match) => {
                self.parser.advance();
                self.parse_match_expression()?
            }

            _ => {
                return Err(ParseError::new(
                    format!("unexpected token in expression: {:?}", self.parser.peek_token_type()),
                    start,
                ))
            }
        };

        Ok(expr)
    }

    /// Parse an infix expression
    fn parse_infix(
        &mut self,
        lhs: Expr,
        op_token: Token,
        right_bp: u8,
        start: Span,
    ) -> ParseResult<Expr> {
        let expr = match op_token.kind {
            TokenType::Operator(op) => {
                match op {
                    Operator::Add | Operator::Sub | Operator::Mul | Operator::Div |
                    Operator::Mod | Operator::Pow | Operator::Concat => {
                        let rhs = self.parse_expression_bp(right_bp)?;
                        let bin_op = match op {
                            Operator::Add => BinaryOp::Add,
                            Operator::Sub => BinaryOp::Sub,
                            Operator::Mul => BinaryOp::Mul,
                            Operator::Div => BinaryOp::Div,
                            Operator::Mod => BinaryOp::Mod,
                            Operator::Pow => BinaryOp::Pow,
                            Operator::Concat => BinaryOp::Concat,
                            _ => unreachable!(),
                        };

                        Expr::Binary {
                            id: nevermind_ast::new_node_id(),
                            left: Box::new(lhs),
                            op: bin_op,
                            right: Box::new(rhs),
                            span: Span::new(start, self.parser.previous_span()),
                        }
                    }

                    Operator::Eq | Operator::Ne | Operator::Lt | Operator::Gt |
                    Operator::Le | Operator::Ge => {
                        let rhs = self.parse_expression_bp(right_bp)?;
                        let cmp_op = match op {
                            Operator::Eq => ComparisonOp::Eq,
                            Operator::Ne => ComparisonOp::Ne,
                            Operator::Lt => ComparisonOp::Lt,
                            Operator::Gt => ComparisonOp::Gt,
                            Operator::Le => ComparisonOp::Le,
                            Operator::Ge => ComparisonOp::Ge,
                            _ => unreachable!(),
                        };

                        Expr::Comparison {
                            id: nevermind_ast::new_node_id(),
                            left: Box::new(lhs),
                            op: cmp_op,
                            right: Box::new(rhs),
                            span: Span::new(start, self.parser.previous_span()),
                        }
                    }

                    Operator::And | Operator::Or => {
                        let rhs = self.parse_expression_bp(right_bp)?;
                        let log_op = match op {
                            Operator::And => LogicalOp::And,
                            Operator::Or => LogicalOp::Or,
                            _ => unreachable!(),
                        };

                        Expr::Logical {
                            id: nevermind_ast::new_node_id(),
                            left: Box::new(lhs),
                            op: log_op,
                            right: Box::new(rhs),
                            span: Span::new(start, self.parser.previous_span()),
                        }
                    }

                    Operator::Pipe => {
                        // Pipeline operator
                        self.parser.consume_delimiter(Delimiter::Pipe, "expected '|' after first expression")?;

                        let mut stages = vec![lhs];
                        stages.push(self.parse_expression_bp(right_bp)?);

                        while self.parser.match_operator(Operator::Pipe) {
                            stages.push(self.parse_expression_bp(right_bp)?);
                        }

                        Expr::Pipeline {
                            id: nevermind_ast::new_node_id(),
                            stages,
                            span: Span::new(start, self.parser.previous_span()),
                        }
                    }

                    Operator::Dot => {
                        // Field access or method call
                        let field = self.parser.consume_identifier("expected field name after '.'")?;

                        Expr::Variable {
                            id: nevermind_ast::new_node_id(),
                            name: field,
                            span: Span::new(start, self.parser.previous_span()),
                        }  // TODO: Implement proper member access
                    }

                    _ => {
                        return Err(ParseError::new(
                            format!("unexpected operator: {:?}", op),
                            op_token.span,
                        ))
                    }
                }
            }

            TokenType::Delimiter(Delimiter::LParen) => {
                // Function call
                let mut args = Vec::new();

                self.parser.advance();  // consume '('

                while !self.parser.check_delimiter(Delimiter::RParen) && !self.parser.is_at_end() {
                    args.push(self.parse_expression_bp(0)?);

                    if !self.parser.match_delimiter(Delimiter::Comma) {
                        break;
                    }
                }

                self.parser.consume_delimiter(Delimiter::RParen, "expected ')' after arguments")?;

                Expr::Call {
                    id: nevermind_ast::new_node_id(),
                    callee: Box::new(lhs),
                    args,
                    span: Span::new(start, self.parser.previous_span()),
                }
            }

            TokenType::Delimiter(Delimiter::LBracket) => {
                // Index or slice
                self.parser.advance();
                let index = self.parse_expression_bp(0)?;
                self.parser.consume_delimiter(Delimiter::RBracket, "expected ']' after index")?;

                Expr::Variable {
                    id: nevermind_ast::new_node_id(),
                    name: format!("{:?}[{:?}]", lhs, index),
                    span: Span::new(start, self.parser.previous_span()),
                }  // TODO: Implement proper indexing
            }

            _ => {
                return Err(ParseError::new(
                    format!("unexpected token in infix position: {:?}", op_token.kind),
                    op_token.span,
                ))
            }
        };

        Ok(expr)
    }

    /// Parse a list literal
    fn parse_list(&mut self) -> ParseResult<Expr> {
        let start = self.parser.peek_span();

        let mut elements = Vec::new();

        while !self.parser.check_delimiter(Delimiter::RBracket) && !self.parser.is_at_end() {
            elements.push(self.parse_expression_bp(0)?);

            if !self.parser.match_delimiter(Delimiter::Comma) {
                break;
            }
        }

        self.parser.consume_delimiter(Delimiter::RBracket, "expected ']' after list elements")?;

        Ok(Expr::List {
            id: nevermind_ast::new_node_id(),
            elements,
            span: Span::new(start, self.parser.previous_span()),
        })
    }

    /// Parse a map literal
    fn parse_map(&mut self) -> ParseResult<Expr> {
        let start = self.parser.peek_span();

        let mut entries = Vec::new();

        while !self.parser.check_delimiter(Delimiter::RBrace) && !self.parser.is_at_end() {
            let key = self.parse_expression_bp(0)?;
            self.parser.consume_delimiter(Delimiter::Colon, "expected ':' after map key")?;
            let value = self.parse_expression_bp(0)?;

            entries.push((key, value));

            if !self.parser.match_delimiter(Delimiter::Comma) {
                break;
            }
        }

        self.parser.consume_delimiter(Delimiter::RBrace, "expected '}' after map entries")?;

        Ok(Expr::Map {
            id: nevermind_ast::new_node_id(),
            entries,
            span: Span::new(start, self.parser.previous_span()),
        })
    }

    /// Parse a lambda expression
    fn parse_lambda(&mut self) -> ParseResult<Expr> {
        let start = self.parser.peek_span();

        // Parse parameters
        let mut params = Vec::new();

        while !self.parser.check_delimiter(Delimiter::Pipe) && !self.parser.is_at_end() {
            let name = self.parser.consume_identifier("expected parameter name")?;

            let type_annotation = if self.parser.match_delimiter(Delimiter::Colon) {
                Some(self.parser.parse_type_annotation()?)
            } else {
                None
            };

            params.push(Parameter {
                id: nevermind_ast::new_node_id(),
                name,
                type_annotation,
                default_value: None,
            });

            if !self.parser.match_delimiter(Delimiter::Comma) {
                break;
            }
        }

        self.parser.consume_delimiter(Delimiter::Pipe, "expected '|' to end lambda parameters")?;

        // Parse body (expression or block)
        let body = if self.parser.check_delimiter(Delimiter::Pipe) {
            // Block body
            self.parse_block()?
        } else {
            // Expression body
            self.parse_expression_bp(0)?
        };

        Ok(Expr::Lambda {
            id: nevermind_ast::new_node_id(),
            params,
            body: Box::new(body),
            span: Span::new(start, self.parser.previous_span()),
        })
    }

    /// Parse an if expression
    fn parse_if_expression(&mut self) -> ParseResult<Expr> {
        let start = self.parser.peek_span();

        let condition = self.parse_expression_bp(0)?;

        self.parser.consume_keyword(Keyword::Then, "expected 'then' in if expression")?;

        let then_branch = self.parse_expression_bp(0)?;

        self.parser.consume_keyword(Keyword::Else, "expected 'else' in if expression")?;

        let else_branch = self.parse_expression_bp(0)?;

        Ok(Expr::If {
            id: nevermind_ast::new_node_id(),
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
            span: Span::new(start, self.parser.previous_span()),
        })
    }

    /// Parse a block expression
    fn parse_block(&mut self) -> ParseResult<Expr> {
        let start = self.parser.peek_span();

        let mut statements = Vec::new();

        while !self.parser.check_keyword(Keyword::End) && !self.parser.is_at_end() {
            if let Some(stmt) = self.parser.parse_statement()? {
                statements.push(stmt);
            }
        }

        self.parser.consume_keyword(Keyword::End, "expected 'end' to close block")?;

        Ok(Expr::Block {
            id: nevermind_ast::new_node_id(),
            statements,
            span: Span::new(start, self.parser.previous_span()),
        })
    }

    /// Parse a match expression
    fn parse_match_expression(&mut self) -> ParseResult<Expr> {
        let start = self.parser.peek_span();

        let scrutinee = self.parse_expression_bp(0)?;

        self.parser.consume_delimiter(Delimiter::LBrace, "expected '{' to start match arms")?;

        let mut arms = Vec::new();

        while !self.parser.check_delimiter(Delimiter::RBrace) && !self.parser.is_at_end() {
            let pattern = self.parser.parse_pattern()?;

            let guard = if self.parser.match_delimiter(Delimiter::Colon) {
                Some(Box::new(self.parse_expression_bp(0)?))
            } else {
                None
            };

            self.parser.consume_operator(Operator::FatArrow, "expected '=>' after match pattern")?;

            let body = self.parse_expression_bp(0)?;

            arms.push(super::MatchArm {
                pattern,
                guard,
                body,
            });

            self.parser.match_delimiter(Delimiter::Comma);
        }

        self.parser.consume_delimiter(Delimiter::RBrace, "expected '}' to end match expression")?;

        Ok(Expr::Match {
            id: nevermind_ast::new_node_id(),
            scrutinee: Box::new(scrutinee),
            arms,
            span: Span::new(start, self.parser.previous_span()),
        })
    }

    /// Get the binding power (precedence) of an operator
    fn get_binding_power(&self, token: &Token) -> Option<(u8, u8)> {
        match &token.kind {
            TokenType::Operator(op) => {
                let bp = match op {
                    Operator::Assign => (2, 1),
                    Operator::Or => (4, 5),
                    Operator::And => (5, 6),
                    Operator::Eq | Operator::Ne | Operator::Lt | Operator::Gt | Operator::Le | Operator::Ge => (8, 9),
                    Operator::Add | Operator::Sub => (10, 11),
                    Operator::Mul | Operator::Div | Operator::Mod => (12, 13),
                    Operator::Pow => (15, 14),  // Right-associative
                    Operator::Concat => (11, 11),
                    Operator::Pipe => (6, 7),
                    _ => return None,
                };
                Some(bp)
            }

            TokenType::Delimiter(Delimiter::LParen) => {
                // Function call has very high precedence
                Some((20, 19))
            }

            TokenType::Delimiter(Delimiter::LBracket) => {
                // Indexing has high precedence
                Some((21, 20))
            }

            TokenType::Operator(Operator::Dot) => {
                // Member access has highest precedence
                Some((22, 21))
            }

            _ => None,
        }
    }
}
