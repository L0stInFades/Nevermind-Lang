//! Token definitions for the Nevermind lexer

use std::fmt;

use nevermind_common::Span;

/// A token produced by the lexer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The type of this token
    pub kind: TokenType,

    /// The span (location) of this token in the source
    pub span: Span,

    /// The raw text of this token
    pub text: String,
}

impl Token {
    /// Create a new token
    pub fn new(kind: TokenType, span: Span, text: String) -> Self {
        Self { kind, span, text }
    }

    /// Create a token with a dummy span
    pub fn dummy(kind: TokenType, text: impl Into<String>) -> Self {
        Self {
            kind,
            span: Span::dummy(),
            text: text.into(),
        }
    }

    /// Check if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(self.kind, TokenType::Keyword(_))
    }

    /// Check if this token is an identifier
    pub fn is_identifier(&self) -> bool {
        matches!(self.kind, TokenType::Identifier)
    }

    /// Check if this token is a literal
    pub fn is_literal(&self) -> bool {
        matches!(self.kind, TokenType::Literal(_))
    }

    /// Check if this token is an operator
    pub fn is_operator(&self) -> bool {
        matches!(self.kind, TokenType::Operator(_))
    }

    /// Check if this token is a delimiter
    pub fn is_delimiter(&self) -> bool {
        matches!(self.kind, TokenType::Delimiter(_))
    }

    /// Check if this token is EOF
    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenType::EOF)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

/// The type of a token
#[derive(Debug, Clone, PartialEq, Eq, Hash, )]
pub enum TokenType {
    /// Keywords
    Keyword(Keyword),

    /// Identifier
    Identifier,

    /// Literals
    Literal(LiteralType),

    /// Operators
    Operator(Operator),

    /// Delimiters
    Delimiter(Delimiter),

    /// End of file
    EOF,
}

/// Keywords in Nevermind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, )]
pub enum Keyword {
    // Variable declarations
    Let,
    Var,

    // Control flow
    If,
    Then,
    Else,
    Elif,
    Match,
    Case,

    // Loops
    For,
    While,
    Forever,
    In,

    // Functions
    Fn,
    Return,

    // Control flow (loop)
    Break,
    Continue,

    // Types
    Type,

    // Error handling
    Try,
    Catch,
    Finally,
    Raise,

    // Concurrency
    Async,
    Await,
    Parallel,
    Sync,

    // Blocks
    Do,
    End,

    // Boolean literals
    True,
    False,

    // Null
    Null,

    // Modules
    Use,
    From,
    Import,
    As,

    // Classes
    Class,
    Extends,
    Implements,

    // Traits
    Trait,

    // Other
    Where,
}

impl Keyword {
    /// Get the keyword from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "let" => Some(Keyword::Let),
            "var" => Some(Keyword::Var),
            "if" => Some(Keyword::If),
            "then" => Some(Keyword::Then),
            "else" => Some(Keyword::Else),
            "elif" => Some(Keyword::Elif),
            "match" => Some(Keyword::Match),
            "case" => Some(Keyword::Case),
            "for" => Some(Keyword::For),
            "while" => Some(Keyword::While),
            "forever" => Some(Keyword::Forever),
            "in" => Some(Keyword::In),
            "fn" => Some(Keyword::Fn),
            "return" => Some(Keyword::Return),
            "break" => Some(Keyword::Break),
            "continue" => Some(Keyword::Continue),
            "type" => Some(Keyword::Type),
            "try" => Some(Keyword::Try),
            "catch" => Some(Keyword::Catch),
            "finally" => Some(Keyword::Finally),
            "raise" => Some(Keyword::Raise),
            "async" => Some(Keyword::Async),
            "await" => Some(Keyword::Await),
            "parallel" => Some(Keyword::Parallel),
            "sync" => Some(Keyword::Sync),
            "do" => Some(Keyword::Do),
            "end" => Some(Keyword::End),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            "null" => Some(Keyword::Null),
            "use" => Some(Keyword::Use),
            "from" => Some(Keyword::From),
            "import" => Some(Keyword::Import),
            "as" => Some(Keyword::As),
            "class" => Some(Keyword::Class),
            "extends" => Some(Keyword::Extends),
            "implements" => Some(Keyword::Implements),
            "trait" => Some(Keyword::Trait),
            "where" => Some(Keyword::Where),
            _ => None,
        }
    }
}

/// Literal types
#[derive(Debug, Clone, PartialEq, Eq, Hash, )]
pub enum LiteralType {
    Integer,
    Float,
    String,
    Char,
}

/// Operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, )]
pub enum Operator {
    // Arithmetic
    Add,        // +
    Sub,        // -
    Mul,        // *
    Div,        // /
    Mod,        // %
    Pow,        // **

    // Comparison
    Eq,         // ==
    Ne,         // !=
    Lt,         // <
    Gt,         // >
    Le,         // <=
    Ge,         // >=

    // Logical
    And,        // and
    Or,         // or
    Not,        // not

    // Bitwise
    BitAnd,     // &
    BitOr,      // |
    BitXor,     // ^
    BitNot,     // ~
    ShiftLeft,  // <<
    ShiftRight, // >>

    // Other
    Pipe,       // |>
    Assign,     // =
    Arrow,      // ->
    FatArrow,   // =>
    Dot,        // .
    DotDot,     // ..
    DotDotDot,  // ...
    Concat,     // ++
}

impl Operator {
    /// Get the operator from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "+" => Some(Operator::Add),
            "-" => Some(Operator::Sub),
            "*" => Some(Operator::Mul),
            "/" => Some(Operator::Div),
            "%" => Some(Operator::Mod),
            "**" => Some(Operator::Pow),
            "==" => Some(Operator::Eq),
            "!=" => Some(Operator::Ne),
            "<" => Some(Operator::Lt),
            ">" => Some(Operator::Gt),
            "<=" => Some(Operator::Le),
            ">=" => Some(Operator::Ge),
            "and" => Some(Operator::And),
            "or" => Some(Operator::Or),
            "not" => Some(Operator::Not),
            "&" => Some(Operator::BitAnd),
            "|" => Some(Operator::BitOr),
            "^" => Some(Operator::BitXor),
            "~" => Some(Operator::BitNot),
            "<<" => Some(Operator::ShiftLeft),
            ">>" => Some(Operator::ShiftRight),
            "|>" => Some(Operator::Pipe),
            "=" => Some(Operator::Assign),
            "->" => Some(Operator::Arrow),
            "=>" => Some(Operator::FatArrow),
            "." => Some(Operator::Dot),
            ".." => Some(Operator::DotDot),
            "..." => Some(Operator::DotDotDot),
            "++" => Some(Operator::Concat),
            _ => None,
        }
    }

    /// Get the symbol for this operator
    pub fn symbol(&self) -> &'static str {
        match self {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::Mod => "%",
            Operator::Pow => "**",
            Operator::Eq => "==",
            Operator::Ne => "!=",
            Operator::Lt => "<",
            Operator::Gt => ">",
            Operator::Le => "<=",
            Operator::Ge => ">=",
            Operator::And => "and",
            Operator::Or => "or",
            Operator::Not => "not",
            Operator::BitAnd => "&",
            Operator::BitOr => "|",
            Operator::BitXor => "^",
            Operator::BitNot => "~",
            Operator::ShiftLeft => "<<",
            Operator::ShiftRight => ">>",
            Operator::Pipe => "|>",
            Operator::Assign => "=",
            Operator::Arrow => "->",
            Operator::FatArrow => "=>",
            Operator::Dot => ".",
            Operator::DotDot => "..",
            Operator::DotDotDot => "...",
            Operator::Concat => "++",
        }
    }
}

/// Delimiters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, )]
pub enum Delimiter {
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]
    Comma,      // ,
    Colon,      // :
    Semicolon,  // ;
    At,         // @
    Question,   // ?
    Dollar,     // $
    Backtick,   // `
}

impl Delimiter {
    /// Get the delimiter from a character
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '(' => Some(Delimiter::LParen),
            ')' => Some(Delimiter::RParen),
            '{' => Some(Delimiter::LBrace),
            '}' => Some(Delimiter::RBrace),
            '[' => Some(Delimiter::LBracket),
            ']' => Some(Delimiter::RBracket),
            ',' => Some(Delimiter::Comma),
            ':' => Some(Delimiter::Colon),
            ';' => Some(Delimiter::Semicolon),
            '@' => Some(Delimiter::At),
            '?' => Some(Delimiter::Question),
            '$' => Some(Delimiter::Dollar),
            '`' => Some(Delimiter::Backtick),
            _ => None,
        }
    }

    /// Get the character for this delimiter
    pub fn as_char(&self) -> char {
        match self {
            Delimiter::LParen => '(',
            Delimiter::RParen => ')',
            Delimiter::LBrace => '{',
            Delimiter::RBrace => '}',
            Delimiter::LBracket => '[',
            Delimiter::RBracket => ']',
            Delimiter::Comma => ',',
            Delimiter::Colon => ':',
            Delimiter::Semicolon => ';',
            Delimiter::At => '@',
            Delimiter::Question => '?',
            Delimiter::Dollar => '$',
            Delimiter::Backtick => '`',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_from_str() {
        assert_eq!(Keyword::from_str("let"), Some(Keyword::Let));
        assert_eq!(Keyword::from_str("if"), Some(Keyword::If));
        assert_eq!(Keyword::from_str("xyz"), None);
    }

    #[test]
    fn test_operator_from_str() {
        assert_eq!(Operator::from_str("+"), Some(Operator::Add));
        assert_eq!(Operator::from_str("=="), Some(Operator::Eq));
        assert_eq!(Operator::from_str("xyz"), None);
    }
}
