//! The Nevermind lexer

use std::iter::Peekable;
use std::str::Chars;

use thiserror::Error;

use nevermind_common::{Error, ErrorKind, Result, SourceLocation, Span};
use super::token::{Token, TokenType, Keyword, Operator, Delimiter, LiteralType};

/// The Nevermind lexer
pub struct Lexer<'a> {
    /// The source code being lexed
    source: &'a str,

    /// The iterator over the source code
    chars: Peekable<Chars<'a>>,

    /// Current location in the source
    location: SourceLocation,

    /// Indentation stack for significant indentation
    indent_stack: Vec<usize>,

    /// Whether we're at the start of a line
    at_line_start: bool,

    /// Pending dedent tokens
    pending_dedents: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            location: SourceLocation::anonymous(),
            indent_stack: vec![0],
            at_line_start: true,
            pending_dedents: 0,
        }
    }

    /// Create a new lexer for a file
    pub fn from_file(source: &'a str, file_path: std::path::PathBuf) -> Self {
        let mut lexer = Self::new(source);
        lexer.location = SourceLocation::start_of_file(file_path);
        lexer
    }

    /// Get the current location
    pub fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Tokenize the entire source
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            tokens.push(token.clone());

            if token.is_eof() {
                break;
            }
        }

        Ok(tokens)
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token> {
        // Emit pending dedents first
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            return Ok(Token::new(
                TokenType::Delimiter(Delimiter::Semicolon),
                Span::point(self.location.clone()),
                "\n".to_string(),
            ));
        }

        // Handle indentation at the start of a line
        if self.at_line_start {
            self.handle_indentation()?;
            self.at_line_start = false;
        }

        // Skip whitespace (except newlines which are handled above)
        self.skip_whitespace();

        // Check for EOF
        if self.peek() == None {
            // Emit remaining dedents
            let dedent_count = self.indent_stack.len() - 1;
            if dedent_count > 0 {
                self.indent_stack.truncate(1);
                if dedent_count > 1 {
                    self.pending_dedents = dedent_count - 1;
                    return Ok(Token::new(
                        TokenType::Delimiter(Delimiter::Semicolon),
                        Span::point(self.location.clone()),
                        "\n".to_string(),
                    ));
                }
            }

            return Ok(Token::new(
                TokenType::EOF,
                Span::point(self.location.clone()),
                "".to_string(),
            ));
        }

        // Get the next character
        let c = self.peek().unwrap();

        // Dispatch based on the character
        let token = match c {
            '0'..='9' => self.lex_number()?,

            '"' => self.lex_string()?,

            '\'' => self.lex_char()?,

            '#' => {
                self.consume_line_comment();
                self.next_token()?
            }

            '/' if self.peek2() == Some('/') => {
                self.consume_line_comment();
                self.next_token()?
            }

            '/' if self.peek2() == Some('*') => {
                self.consume_block_comment()?;
                self.next_token()?
            }

            '(' => self.lex_delimiter(Delimiter::LParen),
            ')' => self.lex_delimiter(Delimiter::RParen),
            '{' => self.lex_delimiter(Delimiter::LBrace),
            '}' => self.lex_delimiter(Delimiter::RBrace),
            '[' => self.lex_delimiter(Delimiter::LBracket),
            ']' => self.lex_delimiter(Delimiter::RBracket),
            ',' => self.lex_delimiter(Delimiter::Comma),
            ':' => self.lex_delimiter(Delimiter::Colon),
            ';' => self.lex_delimiter(Delimiter::Semicolon),
            '@' => self.lex_delimiter(Delimiter::At),
            '?' => self.lex_delimiter(Delimiter::Question),
            '$' => self.lex_delimiter(Delimiter::Dollar),
            '`' => self.lex_delimiter(Delimiter::Backtick),

            '\n' | '\r' => {
                self.advance();
                self.at_line_start = true;
                self.next_token()?
            }

            _ => {
                // Try to lex operators or identifiers
                if let Some(token) = self.lex_operator_or_keyword()? {
                    token
                } else if c.is_alphabetic() || c == '_' {
                    self.lex_identifier_or_keyword()?
                } else {
                    return Err(Error::lexical(
                        format!("unexpected character '{}'", c),
                        Span::point(self.location.clone()),
                    ));
                }
            }
        };

        Ok(token)
    }

    /// Handle indentation (significant whitespace)
    fn handle_indentation(&mut self) -> Result<()> {
        let mut spaces = 0;

        while let Some(&c) = self.chars.peek() {
            if c == ' ' {
                spaces += 1;
                self.advance();
            } else if c == '\t' {
                return Err(Error::lexical(
                    "tabs are not allowed for indentation, use spaces",
                    Span::point(self.location.clone()),
                ));
            } else {
                break;
            }
        }

        let current_indent = *self.indent_stack.last().unwrap();

        if spaces > current_indent {
            // Increase indentation
            self.indent_stack.push(spaces);
        } else if spaces < current_indent {
            // Decrease indentation
            while let Some(&top) = self.indent_stack.last() {
                if top == spaces {
                    break;
                } else if top > spaces {
                    self.indent_stack.pop();
                    self.pending_dedents += 1;
                } else {
                    return Err(Error::lexical(
                        format!("inconsistent indentation (expected {}, got {})", top, spaces),
                        Span::point(self.location.clone()),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Lex a number
    fn lex_number(&mut self) -> Result<Token> {
        let start = self.location.clone();
        let mut text = String::new();
        let mut is_float = false;

        // Integer part
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Fractional part
        if self.peek() == Some('.') {
            if let Some(c) = self.peek2() {
                if c.is_ascii_digit() {
                    is_float = true;
                    text.push(self.advance().unwrap());  // consume '.'
                    while let Some(&c) = self.chars.peek() {
                        if c.is_ascii_digit() {
                            text.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // Exponent part
        if self.peek() == Some('e') || self.peek() == Some('E') {
            is_float = true;
            text.push(self.advance().unwrap());  // consume 'e' or 'E'

            if self.peek() == Some('+') || self.peek() == Some('-') {
                text.push(self.advance().unwrap());
            }

            while let Some(&c) = self.chars.peek() {
                if c.is_ascii_digit() {
                    text.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let span = Span::new(start, self.location.clone());

        Ok(Token::new(
            TokenType::Literal(if is_float {
                LiteralType::Float
            } else {
                LiteralType::Integer
            }),
            span,
            text,
        ))
    }

    /// Lex a string literal
    fn lex_string(&mut self) -> Result<Token> {
        let start = self.location.clone();
        self.advance();  // consume opening '"'

        let mut text = String::new();

        while let Some(&c) = self.chars.peek() {
            match c {
                '"' => {
                    self.advance();
                    let span = Span::new(start, self.location.clone());
                    return Ok(Token::new(
                        TokenType::Literal(LiteralType::String),
                        span,
                        text,
                    ));
                }
                '\\' => {
                    self.advance();
                    if let Some(escaped) = self.lex_escape_sequence()? {
                        text.push(escaped);
                    }
                }
                '\n' | '\r' => {
                    return Err(Error::lexical(
                        "unterminated string literal",
                        Span::new(start, self.location.clone()),
                    ));
                }
                _ => {
                    text.push(c);
                    self.advance();
                }
            }
        }

        Err(Error::lexical(
            "unterminated string literal",
            Span::new(start, self.location.clone()),
        ))
    }

    /// Lex a character literal
    fn lex_char(&mut self) -> Result<Token> {
        let start = self.location.clone();
        self.advance();  // consume opening '\''

        let c = if self.peek() == Some('\\') {
            self.advance();
            self.lex_escape_sequence()?.ok_or_else(|| {
                Error::lexical(
                    "invalid escape sequence",
                    Span::point(self.location.clone()),
                )
            })?
        } else {
            self.advance().ok_or_else(|| {
                Error::lexical(
                    "unterminated character literal",
                    Span::new(start.clone(), self.location.clone()),
                )
            })?
        };

        if self.peek() != Some('\'') {
            return Err(Error::lexical(
                "unterminated character literal",
                Span::new(start, self.location.clone()),
            ));
        }

        self.advance();  // consume closing '\''

        let span = Span::new(start.clone(), self.location.clone());

        Ok(Token::new(
            TokenType::Literal(LiteralType::Char),
            span,
            c.to_string(),
        ))
    }

    /// Lex an escape sequence
    fn lex_escape_sequence(&mut self) -> Result<Option<char>> {
        let c = self.advance().ok_or_else(|| {
            Error::lexical(
                "incomplete escape sequence",
                Span::point(self.location.clone()),
            )
        })?;

        Ok(match c {
            'n' => Some('\n'),
            'r' => Some('\r'),
            't' => Some('\t'),
            '0' => Some('\0'),
            '\\' => Some('\\'),
            '"' => Some('"'),
            '\'' => Some('\''),
            'x' => {
                // Hex escape \xNN
                let mut code = 0;
                for _ in 0..2 {
                    if let Some(c) = self.peek() {
                        if c.is_ascii_hexdigit() {
                            code = code * 16 + c.to_digit(16).unwrap() as u32;
                            self.advance();
                        }
                    }
                }
                std::char::from_u32(code)
            }
            'u' => {
                // Unicode escape \u{NNNN}
                if self.peek() != Some('{') {
                    return Err(Error::lexical(
                        "invalid unicode escape",
                        Span::point(self.location.clone()),
                    ));
                }
                self.advance();  // consume '{'

                let mut code = 0;
                while let Some(&c) = self.chars.peek() {
                    if c == '}' {
                        break;
                    }
                    if c.is_ascii_hexdigit() {
                        code = code * 16 + c.to_digit(16).unwrap() as u32;
                        self.advance();
                    } else {
                        return Err(Error::lexical(
                            "invalid unicode escape",
                            Span::point(self.location.clone()),
                        ));
                    }
                }

                if self.peek() != Some('}') {
                    return Err(Error::lexical(
                        "unterminated unicode escape",
                        Span::point(self.location.clone()),
                    ));
                }
                self.advance();  // consume '}'

                std::char::from_u32(code)
            }
            _ => Some(c),
        })
    }

    /// Lex an identifier or keyword
    fn lex_identifier_or_keyword(&mut self) -> Result<Token> {
        let start = self.location.clone();
        let mut text = String::new();

        // First character must be alphabetic or underscore
        if let Some(c) = self.peek() {
            if c.is_alphabetic() || c == '_' {
                text.push(c);
                self.advance();
            }
        }

        // Subsequent characters can be alphanumeric, underscore, or apostrophe
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' || c == '\'' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let span = Span::new(start, self.location.clone());

        // Check if it's an operator (like "and", "or", "not")
        if let Some(op) = Operator::from_str(&text) {
            return Ok(Token::new(
                TokenType::Operator(op),
                span,
                text,
            ));
        }

        // Check if it's a keyword
        if let Some(keyword) = Keyword::from_str(&text) {
            return Ok(Token::new(
                TokenType::Keyword(keyword),
                span,
                text,
            ));
        }

        // Check if it's a boolean literal
        match text.as_str() {
            "true" => {
                return Ok(Token::new(
                    TokenType::Keyword(Keyword::True),
                    span,
                    text,
                ))
            }
            "false" => {
                return Ok(Token::new(
                    TokenType::Keyword(Keyword::False),
                    span,
                    text,
                ))
            }
            "null" => {
                return Ok(Token::new(
                    TokenType::Keyword(Keyword::Null),
                    span,
                    text,
                ))
            }
            _ => {}
        }

        Ok(Token::new(
            TokenType::Identifier,
            span,
            text,
        ))
    }

    /// Lex an operator
    fn lex_operator_or_keyword(&mut self) -> Result<Option<Token>> {
        let start = self.location.clone();

        // First, try to match multi-character operators by peeking ahead
        let c1 = self.peek();
        let c2 = self.peek2();

        // Try 3-character operators first
        if let (Some(c1), Some(c2), Some(c3)) = (c1, c2, self.chars.clone().nth(2)) {
            let three_char = format!("{}{}{}", c1, c2, c3);
            if let Some(op) = Operator::from_str(&three_char) {
                self.advance(); // consume c1
                self.advance(); // consume c2
                self.advance(); // consume c3
                let span = Span::new(start, self.location.clone());
                return Ok(Some(Token::new(TokenType::Operator(op), span, three_char)));
            }
        }

        // Try 2-character operators
        if let (Some(c1), Some(c2)) = (c1, c2) {
            let two_char = format!("{}{}", c1, c2);
            if let Some(op) = Operator::from_str(&two_char) {
                self.advance(); // consume c1
                self.advance(); // consume c2
                let span = Span::new(start, self.location.clone());
                return Ok(Some(Token::new(TokenType::Operator(op), span, two_char)));
            }
        }

        // Try 1-character operators
        if let Some(c) = c1 {
            let one_char = format!("{}", c);
            if let Some(op) = Operator::from_str(&one_char) {
                self.advance(); // consume c1
                let span = Span::new(start, self.location.clone());
                return Ok(Some(Token::new(TokenType::Operator(op), span, one_char)));
            }
        }

        Ok(None)
    }

    /// Lex a delimiter
    fn lex_delimiter(&mut self, delimiter: Delimiter) -> Token {
        let start = self.location.clone();
        let c = self.advance().unwrap();
        let span = Span::new(start, self.location.clone());

        Token::new(
            TokenType::Delimiter(delimiter),
            span,
            c.to_string(),
        )
    }

    /// Consume a line comment
    fn consume_line_comment(&mut self) {
        self.advance();  // consume '#' or first '/'

        if self.peek() == Some('/') {
            self.advance();
        }

        while let Some(&c) = self.chars.peek() {
            if c == '\n' || c == '\r' {
                break;
            }
            self.advance();
        }
    }

    /// Consume a block comment
    fn consume_block_comment(&mut self) -> Result<()> {
        self.advance();  // consume first '/'
        self.advance();  // consume '*'

        let mut depth = 1;

        while depth > 0 {
            if let Some(c) = self.advance() {
                if c == '/' && self.peek() == Some('*') {
                    self.advance();
                    depth += 1;
                } else if c == '*' && self.peek() == Some('/') {
                    self.advance();
                    depth -= 1;
                }
            } else {
                return Err(Error::lexical(
                    "unterminated block comment",
                    Span::point(self.location.clone()),
                ));
            }
        }

        Ok(())
    }

    /// Skip whitespace (not including newlines)
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Peek at the next character
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Peek at the second next character
    fn peek2(&mut self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next()
    }

    /// Advance to the next character
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(c) = c {
            self.location.advance_char(c);
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_numbers() {
        let mut lexer = Lexer::new("42 3.14 1e10");

        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenType::Literal(LiteralType::Integer));
        assert_eq!(tokens[1].kind, TokenType::Literal(LiteralType::Float));
        assert_eq!(tokens[2].kind, TokenType::Literal(LiteralType::Float));
    }

    #[test]
    fn test_lex_keywords() {
        let mut lexer = Lexer::new("let var if then else fn return");

        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenType::Keyword(Keyword::Let));
        assert_eq!(tokens[1].kind, TokenType::Keyword(Keyword::Var));
        assert_eq!(tokens[2].kind, TokenType::Keyword(Keyword::If));
        assert_eq!(tokens[3].kind, TokenType::Keyword(Keyword::Then));
        assert_eq!(tokens[4].kind, TokenType::Keyword(Keyword::Else));
        assert_eq!(tokens[5].kind, TokenType::Keyword(Keyword::Fn));
        assert_eq!(tokens[6].kind, TokenType::Keyword(Keyword::Return));
    }

    #[test]
    fn test_lex_operators() {
        let mut lexer = Lexer::new("+ - * / == != < > <= >=");

        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenType::Operator(Operator::Add));
        assert_eq!(tokens[1].kind, TokenType::Operator(Operator::Sub));
        assert_eq!(tokens[2].kind, TokenType::Operator(Operator::Mul));
        assert_eq!(tokens[3].kind, TokenType::Operator(Operator::Div));
        assert_eq!(tokens[4].kind, TokenType::Operator(Operator::Eq));
        assert_eq!(tokens[5].kind, TokenType::Operator(Operator::Ne));
        assert_eq!(tokens[6].kind, TokenType::Operator(Operator::Lt));
        assert_eq!(tokens[7].kind, TokenType::Operator(Operator::Gt));
        assert_eq!(tokens[8].kind, TokenType::Operator(Operator::Le));
        assert_eq!(tokens[9].kind, TokenType::Operator(Operator::Ge));
    }

    #[test]
    fn test_lex_string() {
        let mut lexer = Lexer::new(r#""hello""#);

        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenType::Literal(LiteralType::String));
        assert_eq!(tokens[0].text, "hello");
    }

    #[test]
    fn test_indentation() {
        let source = r#"
let x = 1
  let y = 2
let z = 3
"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        // Should have tokens for indentation changes
        // (exact behavior depends on how we encode indentation in tokens)
    }
}
