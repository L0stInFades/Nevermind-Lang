//! Nevermind MIR - Mid-level Intermediate Representation
//!
//! The MIR is a desugared, normalized IR that makes code generation easier.
//! Key properties:
//! - All syntactic sugar removed
//! - Control flow normalized (if-else chains, no if-expressions)
//! - Pattern matching compiled to decision trees
//! - Explicit temporaries for complex expressions
//! - Type information attached to all nodes

mod expr;
mod function;
pub mod lowering;
mod pattern;
mod stmt;

pub use expr::{BinOp, Literal, MirBlock, MirExpr, MirExprStmt, UnaryOp};
pub use function::{MirFunction, MirProgram};
pub use pattern::MirPattern;
pub use stmt::{MirMatchArm, MirStmt, Param};

/// Unique identifier for MIR nodes (re-export from AST crate)
pub type NodeId = nevermind_ast::NodeId;

/// Lower a typed AST program to MIR
pub fn lower_program(ast_program: &Vec<nevermind_ast::Stmt>) -> lowering::Result<MirProgram> {
    let mut program = MirProgram::new();

    for stmt in ast_program {
        let mir_stmt = lowering::lower_statement(stmt)?;
        program.add_statement(mir_stmt);
    }

    Ok(program)
}
