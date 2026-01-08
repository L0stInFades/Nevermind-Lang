//! Pattern parser

use nevermind_common::Span;

use nevermind_lexer::{Token, TokenType};
use nevermind_lexer::token::{Keyword, Delimiter, LiteralType};

use nevermind_ast::{Pattern, Literal};
use nevermind_ast::pattern::StructPatternField;

use super::error::{ParseError, ParseResult};
use super::Parser;

/// Pattern parser
pub struct PatternParser<'a> {
    parser: &'a mut Parser,
}

impl<'a> PatternParser<'a> {
    /// Create a new pattern parser
    pub fn new(parser: &'a mut Parser) -> Self {
        Self { parser }
    }

    /// Parse a pattern
    pub fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        self.parse_pattern_bp(0)
    }

    /// Parse a pattern with minimum binding power (for or patterns)
    fn parse_pattern_bp(&mut self, _min_bp: u8) -> ParseResult<Pattern> {
        let start = self.parser.peek_span();

        // Check for wildcard (_)
        if let TokenType::Identifier = self.parser.peek_token_type() {
            let token = self.parser.advance().unwrap();
            if token.text == "_" {
                return Ok(Pattern::Wildcard {
                    span: self.parser.span_from(start),
                });
            }
        }

        // Check for literal pattern
        if let TokenType::Literal(lit_type) = self.parser.peek_token_type() {
            return self.parse_literal_pattern(start);
        }

        // Check for tuple pattern (parentheses)
        if self.parser.match_delimiter(Delimiter::LParen) {
            return self.parse_tuple_pattern(start);
        }

        // Check for list pattern [ ... ]
        if self.parser.match_delimiter(Delimiter::LBracket) {
            return self.parse_list_pattern(start);
        }

        // Check for struct pattern { ... }
        if self.parser.match_delimiter(Delimiter::LBrace) {
            return self.parse_struct_pattern(start);
        }

        // Check for range pattern
        if let TokenType::Operator(op) = self.parser.peek_token_type() {
            // This would need to check for .. operator
            // For now, skip
        }

        // Default: variable pattern
        let name = self.parser.consume_identifier("expected pattern")?;
        Ok(Pattern::Variable {
            name,
            span: self.parser.span_from(start),
        })
    }

    /// Parse a literal pattern
    fn parse_literal_pattern(&mut self, start: Span) -> ParseResult<Pattern> {
        let token = self.parser.advance().unwrap();
        let span = token.span.clone();

        let literal = match token.kind {
            TokenType::Literal(lit_type) => {
                match lit_type {
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
                }
            }
            TokenType::Keyword(Keyword::True) => Literal::Boolean(true, span),
            TokenType::Keyword(Keyword::False) => Literal::Boolean(false, span),
            TokenType::Keyword(Keyword::Null) => Literal::Null(span),
            _ => {
                return Err(ParseError::new(
                    format!("expected literal pattern, found {:?}", token.kind),
                    start,
                ))
            }
        };

        Ok(Pattern::Literal {
            value: literal,
            span: self.parser.span_from(start),
        })
    }

    /// Parse a tuple pattern (pat1, pat2, ...)
    fn parse_tuple_pattern(&mut self, start: Span) -> ParseResult<Pattern> {
        let mut patterns = Vec::new();

        while !self.parser.check_delimiter(Delimiter::RParen) && !self.parser.is_at_end() {
            patterns.push(self.parse_pattern()?);

            if !self.parser.match_delimiter(Delimiter::Comma) {
                break;
            }
        }

        self.parser.consume_delimiter(Delimiter::RParen, "expected ')' to close tuple pattern")?;

        Ok(Pattern::Tuple {
            patterns,
            span: self.parser.span_from(start),
        })
    }

    /// Parse a list pattern [pat1, pat2, ...] or [head | tail]
    fn parse_list_pattern(&mut self, start: Span) -> ParseResult<Pattern> {
        let mut patterns = Vec::new();

        while !self.parser.check_delimiter(Delimiter::RBracket) && !self.parser.is_at_end() {
            patterns.push(self.parse_pattern()?);

            // Check for | (list cons pattern)
            if self.parser.match_operator(nevermind_lexer::token::Operator::BitOr) {
                let tail = self.parse_pattern()?;
                self.parser.consume_delimiter(Delimiter::RBracket, "expected ']' to close list pattern")?;

                return Ok(Pattern::ListCons {
                    head: Box::new(patterns.pop().unwrap()),
                    tail: Box::new(tail),
                    span: self.parser.span_from(start),
                });
            }

            if !self.parser.match_delimiter(Delimiter::Comma) {
                break;
            }
        }

        self.parser.consume_delimiter(Delimiter::RBracket, "expected ']' to close list pattern")?;

        Ok(Pattern::List {
            patterns,
            span: self.parser.span_from(start),
        })
    }

    /// Parse a struct pattern { field1: pat1, field2: pat2, ... }
    fn parse_struct_pattern(&mut self, start: Span) -> ParseResult<Pattern> {
        let name = self.parser.consume_identifier("expected struct name in struct pattern")?;

        let mut fields = Vec::new();

        while !self.parser.check_delimiter(Delimiter::RBrace) && !self.parser.is_at_end() {
            let field_name = self.parser.consume_identifier("expected field name")?;
            let shorthand = !self.parser.match_delimiter(Delimiter::Colon);

            let pattern = if shorthand {
                Pattern::Variable {
                    name: field_name.clone(),
                    span: self.parser.peek_span(),
                }
            } else {
                self.parse_pattern()?
            };

            fields.push(StructPatternField {
                name: field_name,
                pattern,
                shorthand,
            });

            if !self.parser.match_delimiter(Delimiter::Comma) {
                break;
            }
        }

        self.parser.consume_delimiter(Delimiter::RBrace, "expected '}' to close struct pattern")?;

        Ok(Pattern::Struct {
            name,
            fields,
            span: self.parser.span_from(start),
        })
    }
}
