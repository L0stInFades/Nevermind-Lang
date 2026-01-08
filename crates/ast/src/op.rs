//! Operators in Nevermind

/// Binary arithmetic operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    /// Addition (+)
    Add,

    /// Subtraction (-)
    Sub,

    /// Multiplication (*)
    Mul,

    /// Division (/)
    Div,

    /// Modulo (%)
    Mod,

    /// Exponentiation (**)
    Pow,

    /// Concatenation (++)
    Concat,
}

impl BinaryOp {
    /// Get the precedence of this operator (higher = tighter binding)
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Pow => 14,
            Self::Mul | Self::Div | Self::Mod => 13,
            Self::Add | Self::Sub => 12,
            Self::Concat => 11,
        }
    }

    /// Check if this operator is left-associative
    pub fn is_left_associative(&self) -> bool {
        match self {
            Self::Pow => false,  // Right-associative
            _ => true,
        }
    }

    /// Get the symbol for this operator
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::Pow => "**",
            Self::Concat => "++",
        }
    }
}

/// Binary comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonOp {
    /// Equal (==)
    Eq,

    /// Not equal (!=)
    Ne,

    /// Less than (<)
    Lt,

    /// Less than or equal (<=)
    Le,

    /// Greater than (>)
    Gt,

    /// Greater than or equal (>=)
    Ge,
}

impl ComparisonOp {
    /// Get the symbol for this operator
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// Logical negation (!)
    Not,

    /// Bitwise negation (~)
    BitNot,

    /// Negation (-)
    Neg,

    /// Dereference (*)
    Deref,

    /// Address of (&)
    Ref,
}

impl UnaryOp {
    /// Get the symbol for this operator
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Not => "!",
            Self::BitNot => "~",
            Self::Neg => "-",
            Self::Deref => "*",
            Self::Ref => "&",
        }
    }

    /// Check if this operator is postfix
    pub fn is_postfix(&self) -> bool {
        matches!(self, Self::Deref)
    }
}

/// Logical operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicalOp {
    /// Logical and (and)
    And,

    /// Logical or (or)
    Or,
}

impl LogicalOp {
    /// Get the precedence of this operator
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Or => 4,
            Self::And => 5,
        }
    }

    /// Get the symbol for this operator
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence() {
        assert!(BinaryOp::Mul.precedence() > BinaryOp::Add.precedence());
        assert!(BinaryOp::Pow.precedence() > BinaryOp::Mul.precedence());
    }

    #[test]
    fn test_associativity() {
        assert!(BinaryOp::Add.is_left_associative());
        assert!(!BinaryOp::Pow.is_left_associative());
    }
}
