//! Python bytecode code generator for Nevermind
//!
//! This module translates MIR to Python bytecode.

pub mod emit;
pub mod python;

pub use emit::{BytecodeChunk, CodeEmitter};
pub use python::PythonGenerator;

use emit::Result;
use nevermind_mir::MirProgram;

/// Generate Python bytecode from MIR program
pub fn generate(program: &MirProgram) -> Result<String> {
    PythonGenerator::new().generate(program)
}
