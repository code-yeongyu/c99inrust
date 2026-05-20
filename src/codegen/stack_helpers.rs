use super::aarch64_analysis::expr_depth;
use super::widths::{TEMPORARY_BYTES, ValueWidth, width_bytes};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{LoweredExpr, LoweredFunction};

pub(in crate::codegen) fn local_offset(
    function: &LoweredFunction,
    slot: usize,
) -> CompileResult<usize> {
    function
        .local_slots
        .get(slot)
        .map(|local_slot| local_slot.offset)
        .ok_or_else(|| CompileError::new("internal error: missing local slot"))
}

pub(in crate::codegen) fn x86_stack_offset(byte_offset: usize, width: ValueWidth) -> String {
    format!("-{}", byte_offset + width_bytes(width))
}

pub(in crate::codegen) fn x86_stack_object_offset(byte_offset: usize, byte_size: usize) -> String {
    format!("-{}", byte_offset + byte_size)
}

pub(in crate::codegen) fn x86_stack_byte_offset(
    object_offset: usize,
    object_size: usize,
    byte_offset: usize,
) -> String {
    let index = byte_offset - object_offset;
    format!("-{}", object_offset + object_size - index)
}

pub(in crate::codegen) fn local_stack_bytes(function: &LoweredFunction) -> usize {
    function
        .local_slots
        .iter()
        .map(|local_slot| local_slot.offset + local_slot.byte_size)
        .max()
        .unwrap_or(0)
}
pub(in crate::codegen) const fn align_to(value: usize, alignment: usize) -> usize {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

pub(in crate::codegen) fn call_stack_argument_bytes(
    arg_count: usize,
    register_count: usize,
) -> CompileResult<usize> {
    let stack_arg_count = arg_count.saturating_sub(register_count);
    let byte_count = stack_arg_count
        .checked_mul(TEMPORARY_BYTES)
        .ok_or_else(|| CompileError::new("call stack argument size overflow"))?;
    Ok(align_to(byte_count, 16))
}

pub(in crate::codegen) fn call_arg_depth(args: &[LoweredExpr]) -> usize {
    if args.is_empty() {
        0
    } else {
        args.len() + args.iter().map(expr_depth).max().unwrap_or(0)
    }
}

pub(in crate::codegen) const fn memory_scale_shift_for_byte_size(byte_size: usize) -> Option<u8> {
    match byte_size {
        1 => Some(0),
        2 => Some(1),
        4 => Some(2),
        8 => Some(3),
        _ => None,
    }
}

pub(in crate::codegen) const fn memory_scale_bytes_for_byte_size(byte_size: usize) -> Option<u8> {
    match byte_size {
        1 => Some(1),
        2 => Some(2),
        4 => Some(4),
        8 => Some(8),
        _ => None,
    }
}
