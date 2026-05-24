use super::stack_helpers::{x86_stack_byte_offset, x86_stack_object_offset};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredFunction;
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_x86_64_complex_parameter_stores(
    function: &LoweredFunction,
    assembly: &mut String,
) -> CompileResult<()> {
    let mut float_register = 0usize;
    let mut integer_register = 0usize;
    for slot in 0..function.parameter_count {
        let Some(local_slot) = function.local_slots.get(slot) else {
            return Err(CompileError::new("internal error: missing parameter slot"));
        };
        match local_slot.scalar_type {
            ScalarType::ComplexFloat => {
                if float_register >= 8 {
                    return Err(CompileError::new("too many complex function parameters"));
                }
                write_assembly!(
                    assembly,
                    "\tmovsd %xmm{float_register}, {}(%rbp)\n",
                    x86_stack_object_offset(local_slot.offset, 8)
                )?;
                float_register += 1;
            }
            ScalarType::ComplexDouble => {
                if float_register + 1 >= 8 {
                    return Err(CompileError::new("too many complex function parameters"));
                }
                write_assembly!(
                    assembly,
                    "\tmovsd %xmm{float_register}, {}(%rbp)\n",
                    x86_stack_object_offset(local_slot.offset, 16)
                )?;
                write_assembly!(
                    assembly,
                    "\tmovsd %xmm{}, {}(%rbp)\n",
                    float_register + 1,
                    x86_stack_byte_offset(local_slot.offset, 16, local_slot.offset + 8)
                )?;
                float_register += 2;
            }
            ScalarType::Double | ScalarType::LongDouble => {
                write_assembly!(
                    assembly,
                    "\tmovsd %xmm{float_register}, {}(%rbp)\n",
                    x86_stack_object_offset(local_slot.offset, 8)
                )?;
                float_register += 1;
            }
            _ => {
                write_assembly!(
                    assembly,
                    "\tmovq %{}, {}(%rbp)\n",
                    integer_register_name(integer_register)?,
                    x86_stack_object_offset(local_slot.offset, 8)
                )?;
                integer_register += 1;
            }
        }
    }
    Ok(())
}

fn integer_register_name(index: usize) -> CompileResult<&'static str> {
    const REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
    REGISTERS
        .get(index)
        .copied()
        .ok_or_else(|| CompileError::new("too many function parameters"))
}
