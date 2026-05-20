use super::aarch64_function::emit_aarch64_function;
use super::globals::emit_globals;
use super::target::Target;
use super::x86_64_function::emit_x86_64_function;
use crate::diagnostics::CompileResult;
use crate::ir::LoweredProgram;

/// Emits target assembly for a lowered program.
///
/// # Errors
///
/// Returns an error when an expression cannot be represented by the current
/// scalar backend.
pub fn emit_assembly(program: &LoweredProgram, target: Target) -> CompileResult<String> {
    let mut assembly = String::new();
    emit_globals(&program.globals, target, &mut assembly)?;
    if program.functions.is_empty() {
        if target == Target::X86_64UnknownLinuxGnu {
            assembly.push_str(".section .note.GNU-stack,\"\",@progbits\n");
        }
        return Ok(assembly);
    }
    assembly.push_str(".text\n");
    for function in &program.functions {
        match target {
            Target::Aarch64AppleDarwin => emit_aarch64_function(function, target, &mut assembly)?,
            Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
                emit_x86_64_function(function, target, &mut assembly)?;
            }
        }
    }
    if target == Target::X86_64UnknownLinuxGnu {
        assembly.push_str(".section .note.GNU-stack,\"\",@progbits\n");
    }
    Ok(assembly)
}
