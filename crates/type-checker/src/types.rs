//! Type representation for Nevermind

use std::fmt;
use std::rc::Rc;
use nevermind_common::Span;

/// A type in the Nevermind type system
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Type variable (for inference)
    Var(TypeVarRef),

    /// Basic types
    Int,
    Float,
    String,
    Bool,
    Null,
    Unit,

    /// Function type (args -> return)
    Function(Vec<Type>, Box<Type>),

    /// List type [T]
    List(Box<Type>),

    /// Map type {String: T}
    Map(Box<Type>),

    /// Tuple type (T1, T2, ..., Tn)
    Tuple(Vec<Type>),

    /// User-defined type
    User(String),
}

/// Reference to a type variable (shared, mutable)
#[derive(Debug, Clone)]
pub struct TypeVarRef {
    inner: Rc<std::cell::RefCell<TypeVarInner>>,
}

impl PartialEq for TypeVarRef {
    fn eq(&self, other: &Self) -> bool {
        self.inner.borrow().id == other.inner.borrow().id
    }
}

#[derive(Debug, Clone)]
struct TypeVarInner {
    id: usize,
    name: Option<String>,
}

impl TypeVarRef {
    pub fn new(id: usize) -> Self {
        Self {
            inner: Rc::new(std::cell::RefCell::new(TypeVarInner {
                id,
                name: None,
            })),
        }
    }

    pub fn with_name(id: usize, name: String) -> Self {
        Self {
            inner: Rc::new(std::cell::RefCell::new(TypeVarInner {
                id,
                name: Some(name),
            })),
        }
    }

    pub fn id(&self) -> usize {
        self.inner.borrow().id
    }

    pub fn set_name(&self, name: String) {
        self.inner.borrow_mut().name = Some(name);
    }

    pub fn get_name(&self) -> Option<String> {
        self.inner.borrow().name.clone()
    }
}

impl Type {
    /// Create a fresh type variable
    pub fn var(id: usize) -> Self {
        Type::Var(TypeVarRef::new(id))
    }

    /// Create a named type variable
    pub fn var_with_name(id: usize, name: String) -> Self {
        Type::Var(TypeVarRef::with_name(id, name))
    }

    /// Create a function type
    pub fn function(params: Vec<Type>, ret: Type) -> Self {
        Type::Function(params, Box::new(ret))
    }

    /// Create a list type
    pub fn list(elem: Type) -> Self {
        Type::List(Box::new(elem))
    }

    /// Create a map type
    pub fn map(value: Type) -> Self {
        Type::Map(Box::new(value))
    }

    /// Create a tuple type
    pub fn tuple(elems: Vec<Type>) -> Self {
        Type::Tuple(elems)
    }

    /// Create a user-defined type
    pub fn user(name: String) -> Self {
        Type::User(name)
    }

    /// Check if this type is a type variable
    pub fn is_var(&self) -> bool {
        matches!(self, Type::Var(_))
    }

    /// Get the type variable ID if this is a type variable
    pub fn as_var_id(&self) -> Option<usize> {
        match self {
            Type::Var(var) => Some(var.id()),
            _ => None,
        }
    }

    /// Get the display name for this type
    pub fn display_name(&self) -> String {
        match self {
            Type::Var(var) => {
                if let Some(name) = var.get_name() {
                    name
                } else {
                    format!("t{}", var.id())
                }
            }
            Type::Int => "Int".to_string(),
            Type::Float => "Float".to_string(),
            Type::String => "String".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Null => "Null".to_string(),
            Type::Unit => "Unit".to_string(),
            Type::Function(params, ret) => {
                let params: Vec<String> = params.iter().map(|p| p.display_name()).collect();
                format!("({}) -> {}", params.join(", "), ret.display_name())
            }
            Type::List(elem) => format!("[{}]", elem.display_name()),
            Type::Map(value) => format!("{{String: {}}}", value.display_name()),
            Type::Tuple(elems) => {
                let elems: Vec<String> = elems.iter().map(|e| e.display_name()).collect();
                format!("({})", elems.join(", "))
            }
            Type::User(name) => name.clone(),
        }
    }

    /// Check if this type is a function type
    pub fn is_function(&self) -> bool {
        matches!(self, Type::Function(_, _))
    }

    /// Check if this type is a numeric type (Int or Float)
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A type variable (for inference)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    id: usize,
}

impl TypeVar {
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl fmt::Display for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_creation() {
        let int_type = Type::Int;
        assert_eq!(int_type.display_name(), "Int");

        let fn_type = Type::function(vec![Type::Int, Type::Int], Type::Int);
        assert!(fn_type.is_function());
    }

    #[test]
    fn test_list_type() {
        let list_type = Type::list(Type::Int);
        assert_eq!(list_type.display_name(), "[Int]");
    }

    #[test]
    fn test_map_type() {
        let map_type = Type::map(Type::String);
        assert_eq!(map_type.display_name(), "{String: String}");
    }

    #[test]
    fn test_tuple_type() {
        let tuple_type = Type::tuple(vec![Type::Int, Type::Bool]);
        assert_eq!(tuple_type.display_name(), "(Int, Bool)");
    }

    #[test]
    fn test_type_var() {
        let var = Type::var(0);
        assert!(var.is_var());
        assert_eq!(var.as_var_id(), Some(0));
    }

    #[test]
    fn test_named_type_var() {
        let var = Type::var_with_name(0, "T".to_string());
        assert!(var.is_var());
        assert_eq!(var.display_name(), "T");
    }
}
