//! Type definitions

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::NodeId;
use nevermind_common::Span;

/// A type annotation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeAnnotation {
    pub id: NodeId,
    pub span: Span,
    pub kind: Type,
}

impl TypeAnnotation {
    pub fn new(kind: Type, span: Span) -> Self {
        Self {
            id: crate::new_node_id(),
            span,
            kind,
        }
    }
}

/// Types in Nevermind
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Primitive types
    Primitive(PrimitiveType),

    /// User-defined type name
    Identifier(String),

    /// List type (e.g., List[Int])
    List(Box<TypeAnnotation>),

    /// Map type (e.g., Map[String, Int])
    Map {
        key: Box<TypeAnnotation>,
        value: Box<TypeAnnotation>,
    },

    /// Set type (e.g., Set[Int])
    Set(Box<TypeAnnotation>),

    /// Tuple type (e.g., (Int, String, Bool))
    Tuple(Vec<TypeAnnotation>),

    /// Function type (e.g., fn(Int, String) -> Bool)
    Function {
        params: Vec<TypeAnnotation>,
        return_type: Box<TypeAnnotation>,
    },

    /// Option type (e.g., Option[Int])
    Option(Box<TypeAnnotation>),

    /// Result type (e.g., Result[Int, String])
    Result {
        ok: Box<TypeAnnotation>,
        error: Box<TypeAnnotation>,
    },

    /// Union type (e.g., Int | String)
    Union(Vec<TypeAnnotation>),

    /// Intersection type (e.g., Readable & Writable)
    Intersection(Vec<TypeAnnotation>),

    /// Generic type with parameters (e.g., MyClass[T, U])
    Generic {
        name: String,
        params: Vec<TypeAnnotation>,
    },
}

/// Primitive types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    /// Signed integer
    Int,

    /// Unsigned integer
    UInt,

    /// 64-bit signed integer
    Int64,

    /// 64-bit unsigned integer
    UInt64,

    /// 32-bit signed integer
    Int32,

    /// 32-bit unsigned integer
    UInt32,

    /// Floating point number
    Float,

    /// 64-bit floating point
    Float64,

    /// 32-bit floating point
    Float32,

    /// Boolean
    Bool,

    /// String
    String,

    /// Character
    Char,

    /// Unit type (only one value: ())
    Unit,

    /// Null/Nil type
    Null,
}

impl PrimitiveType {
    /// Get the name of this primitive type
    pub fn name(&self) -> &'static str {
        match self {
            Self::Int => "Int",
            Self::UInt => "UInt",
            Self::Int64 => "Int64",
            Self::UInt64 => "UInt64",
            Self::Int32 => "Int32",
            Self::UInt32 => "UInt32",
            Self::Float => "Float",
            Self::Float64 => "Float64",
            Self::Float32 => "Float32",
            Self::Bool => "Bool",
            Self::String => "String",
            Self::Char => "Char",
            Self::Unit => "()",
            Self::Null => "Null",
        }
    }

    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Int | Self::UInt | Self::Int64 | Self::UInt64 |
            Self::Int32 | Self::UInt32 | Self::Float | Self::Float64 | Self::Float32
        )
    }

    /// Check if this is an integer type
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::Int | Self::UInt | Self::Int64 | Self::UInt64 |
            Self::Int32 | Self::UInt32
        )
    }

    /// Check if this is a floating-point type
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float | Self::Float64 | Self::Float32)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(p) => write!(f, "{}", p.name()),
            Type::Identifier(name) => write!(f, "{}", name),
            Type::List(inner) => write!(f, "List[{}]", inner.kind),
            Type::Map { key, value } => write!(f, "Map[{}, {}]", key.kind, value.kind),
            Type::Set(inner) => write!(f, "Set[{}]", inner.kind),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty.kind)?;
                }
                write!(f, ")")
            }
            Type::Function { params, return_type } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param.kind)?;
                }
                write!(f, ") -> {}", return_type.kind)
            }
            Type::Option(inner) => write!(f, "Option[{}]", inner.kind),
            Type::Result { ok, error } => write!(f, "Result[{}, {}]", ok.kind, error.kind),
            Type::Union(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", ty.kind)?;
                }
                Ok(())
            }
            Type::Intersection(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " & ")?;
                    }
                    write!(f, "{}", ty.kind)?;
                }
                Ok(())
            }
            Type::Generic { name, params } => {
                write!(f, "{}[", name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param.kind)?;
                }
                write!(f, "]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_names() {
        assert_eq!(PrimitiveType::Int.name(), "Int");
        assert_eq!(PrimitiveType::String.name(), "String");
        assert_eq!(PrimitiveType::Bool.name(), "Bool");
    }

    #[test]
    fn test_type_display() {
        let int_type = Type::Primitive(PrimitiveType::Int);
        assert_eq!(int_type.to_string(), "Int");

        let list_type = Type::List(Box::new(TypeAnnotation::new(
            Type::Primitive(PrimitiveType::String),
            Span::dummy(),
        )));
        assert_eq!(list_type.to_string(), "List[String]");
    }
}
