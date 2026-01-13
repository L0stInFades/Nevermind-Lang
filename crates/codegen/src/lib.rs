//! Python bytecode code generator for Nevermind
//!
//! This module translates MIR to Python bytecode.

pub mod emit;
pub mod python;

pub use emit::{CodeEmitter, BytecodeChunk};
pub use python::PythonGenerator;

use nevermind_mir::{MirProgram};
use emit::Result;

/// Generate Python bytecode from MIR program
pub fn generate(program: &MirProgram) -> Result<String> {
    PythonGenerator::new().generate(program)
}