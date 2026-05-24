use super::aarch64_addressing::{aarch64_parameter_register, emit_aarch64_store_local};
use super::aarch64_complex_abi::emit_aarch64_complex_parameter_stores;
use super::aarch64_temporaries::emit_aarch64_store_result;
use super::complex_abi::is_complex_scalar;
use super::stack_helpers::local_offset;
use super::widths::scalar_width;
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{LoweredExpr, LoweredFunction};
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_aarch64_store_local_instruction(
    local: (usize, usize, ScalarType),
    value: &LoweredExpr,
    temporary_base: usize,
    labels: &mut super::frames::LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let (slot, offset, scalar_type) = local;
    emit_aarch64_store_local(
        slot,
        offset,
        scalar_type,
        value,
        temporary_base,
        labels,
        assembly,
    )?;
    emit_aarch64_store_result(scalar_width(scalar_type), offset, assembly)
}

pub(in crate::codegen) fn emit_aarch64_parameter_stores(
    function: &LoweredFunction,
    assembly: &mut String,
) -> CompileResult<()> {
    const MAX_REGISTER_ARGS: usize = 8;
    if function.parameter_count > MAX_REGISTER_ARGS {
        return Err(CompileError::new("too many function parameters"));
    }
    if function
        .local_slots
        .iter()
        .take(function.parameter_count)
        .any(|slot| is_complex_scalar(slot.scalar_type))
    {
        return emit_aarch64_complex_parameter_stores(function, assembly);
    }
    for slot in 0..function.parameter_count {
        let Some(local_slot) = function.local_slots.get(slot) else {
            return Err(CompileError::new("internal error: missing parameter slot"));
        };
        let width = scalar_width(local_slot.scalar_type);
        let register = aarch64_parameter_register(slot, width);
        write_assembly!(
            assembly,
            "\tstr {register}, [sp, #{}]\n",
            local_offset(function, slot)?
        )?;
    }
    Ok(())
}
