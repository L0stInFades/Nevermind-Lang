//! Python bytecode code generator for Nevermind
//!
//! This module translates MIR to Python bytecode.

pub mod emit;
pub mod python;

pub use emit::{BytecodeChunk, CodeEmitter};
pub use python::{PythonGenerator, PythonModuleContext};

use emit::Result;
use nevermind_mir::MirProgram;

/// Generate Python bytecode from MIR program
pub fn generate(program: &MirProgram) -> Result<String> {
    PythonGenerator::new().generate(program)
}

pub fn generate_with_context(
    program: &MirProgram,
    module_context: PythonModuleContext,
) -> Result<String> {
    PythonGenerator::with_module_context(module_context).generate(program)
}
