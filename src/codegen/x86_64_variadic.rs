use super::frames::X86_64VariadicFrame;
use super::stack_helpers::{local_offset, x86_stack_byte_offset};
use super::widths::{
    TEMPORARY_BYTES, X86_64_VARIADIC_FP_OFFSET, X86_64_VARIADIC_FP_REGISTER_BYTES,
    X86_64_VARIADIC_FP_REGISTERS, X86_64_VARIADIC_GP_REGISTERS,
    X86_64_VARIADIC_REGISTER_SAVE_BYTES,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredFunction;

pub(in crate::codegen) fn x86_64_variadic_frame(
    function: &LoweredFunction,
) -> CompileResult<Option<X86_64VariadicFrame>> {
    let Some(slot) = function.variadic_save_slot else {
        return Ok(None);
    };
    let register_save_offset = local_offset(function, slot)?;
    let named_gp_args = function
        .parameter_count
        .min(X86_64_VARIADIC_GP_REGISTERS.len());
    let named_fp_args = function
        .parameter_count
        .min(X86_64_VARIADIC_FP_REGISTERS.len());
    let stack_named_args = function
        .parameter_count
        .saturating_sub(X86_64_VARIADIC_GP_REGISTERS.len());
    Ok(Some(X86_64VariadicFrame {
        gp_offset: named_gp_args
            .checked_mul(TEMPORARY_BYTES)
            .ok_or_else(|| CompileError::new("variadic gp offset overflow"))?,
        fp_offset: X86_64_VARIADIC_FP_OFFSET
            .checked_add(
                named_fp_args
                    .checked_mul(X86_64_VARIADIC_FP_REGISTER_BYTES)
                    .ok_or_else(|| CompileError::new("variadic fp offset overflow"))?,
            )
            .ok_or_else(|| CompileError::new("variadic fp offset overflow"))?,
        overflow_arg_offset: 16usize
            .checked_add(
                stack_named_args
                    .checked_mul(TEMPORARY_BYTES)
                    .ok_or_else(|| CompileError::new("variadic overflow offset overflow"))?,
            )
            .ok_or_else(|| CompileError::new("variadic overflow offset overflow"))?,
        register_save_offset,
        register_save_size: X86_64_VARIADIC_REGISTER_SAVE_BYTES,
    }))
}

pub(in crate::codegen) fn emit_x86_64_variadic_register_saves(
    function: &LoweredFunction,
    assembly: &mut String,
) -> CompileResult<()> {
    let Some(frame) = x86_64_variadic_frame(function)? else {
        return Ok(());
    };
    for (index, register) in X86_64_VARIADIC_GP_REGISTERS.iter().enumerate() {
        let offset = frame
            .register_save_offset
            .checked_add(index * TEMPORARY_BYTES)
            .ok_or_else(|| CompileError::new("variadic register save offset overflow"))?;
        emit_x86_64_register_save(frame, offset, register, assembly)?;
    }
    for (index, register) in X86_64_VARIADIC_FP_REGISTERS.iter().enumerate() {
        let offset = frame
            .register_save_offset
            .checked_add(X86_64_VARIADIC_FP_OFFSET)
            .and_then(|offset| offset.checked_add(index * X86_64_VARIADIC_FP_REGISTER_BYTES))
            .ok_or_else(|| CompileError::new("variadic fp register save offset overflow"))?;
        emit_x86_64_fp_register_save(frame, offset, register, assembly)?;
    }
    Ok(())
}

fn emit_x86_64_register_save(
    frame: X86_64VariadicFrame,
    offset: usize,
    register: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    let destination =
        x86_stack_byte_offset(frame.register_save_offset, frame.register_save_size, offset);
    write_assembly!(assembly, "\tmovq {register}, {destination}(%rbp)\n")
}

fn emit_x86_64_fp_register_save(
    frame: X86_64VariadicFrame,
    offset: usize,
    register: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    let destination =
        x86_stack_byte_offset(frame.register_save_offset, frame.register_save_size, offset);
    write_assembly!(assembly, "\tmovsd {register}, {destination}(%rbp)\n")
}
