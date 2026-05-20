use super::aarch64_addressing::aarch64_register_prefix;
use super::aarch64_expr::emit_aarch64_expr_with_width;
use super::aarch64_temporaries::{
    emit_aarch64_load_temporary_to_register, emit_aarch64_store_temporary,
};
use super::data_literals::label_name;
use super::frames::LabelAllocator;
use super::stack_helpers::call_stack_argument_bytes;
use super::widths::{TEMPORARY_BYTES, ValueWidth, expr_width};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_aarch64_call(
    callee: &str,
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    const REGISTERS: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];
    let register_count = args.len().min(REGISTERS.len());
    let registers = &REGISTERS[..register_count];
    let stack_bytes = call_stack_argument_bytes(args.len(), REGISTERS.len())?;
    let arg_depth = depth + args.len();
    for (index, arg) in args.iter().enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        emit_aarch64_expr_with_width(arg, width, temporary_base, arg_depth, labels, assembly)?;
        emit_aarch64_store_temporary(width, offset, assembly)?;
    }
    for (index, (arg, register)) in args.iter().zip(registers.iter()).enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        emit_aarch64_load_temporary_to_register(expr_width(arg), offset, register, assembly)?;
    }
    emit_aarch64_stack_arguments(args, temporary_base, depth, stack_bytes, assembly)?;
    write_assembly!(assembly, "\tbl {}\n", label_name(callee, labels.target))?;
    emit_aarch64_pop_call_stack(stack_bytes, assembly)
}

pub(in crate::codegen) fn emit_aarch64_call_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Call { callee, args, .. } => {
            emit_aarch64_call(callee, args, temporary_base, depth, labels, assembly)
        }
        LoweredExpr::IndirectCall { callee, args } => {
            emit_aarch64_indirect_call(callee, args, temporary_base, depth, labels, assembly)
        }
        _ => Err(CompileError::new(
            "internal error: expected aarch64 call expression",
        )),
    }
}

pub(in crate::codegen) fn emit_aarch64_indirect_call(
    callee: &LoweredExpr,
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    const REGISTERS: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];
    let register_count = args.len().min(REGISTERS.len());
    let registers = &REGISTERS[..register_count];
    let stack_bytes = call_stack_argument_bytes(args.len(), REGISTERS.len())?;
    let callee_offset = temporary_base + ((depth + args.len()) * TEMPORARY_BYTES);
    let arg_depth = depth + args.len() + 1;
    for (index, arg) in args.iter().enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        emit_aarch64_expr_with_width(arg, width, temporary_base, arg_depth, labels, assembly)?;
        emit_aarch64_store_temporary(width, offset, assembly)?;
    }
    emit_aarch64_expr_with_width(
        callee,
        ValueWidth::I64,
        temporary_base,
        arg_depth,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, callee_offset, assembly)?;
    for (index, (arg, register)) in args.iter().zip(registers.iter()).enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        emit_aarch64_load_temporary_to_register(expr_width(arg), offset, register, assembly)?;
    }
    emit_aarch64_stack_arguments(args, temporary_base, depth, stack_bytes, assembly)?;
    let adjusted_callee_offset = callee_offset
        .checked_add(stack_bytes)
        .ok_or_else(|| CompileError::new("call callee offset overflow"))?;
    emit_aarch64_load_temporary_to_register(
        ValueWidth::I64,
        adjusted_callee_offset,
        "16",
        assembly,
    )?;
    write_assembly!(assembly, "\tblr x16\n")?;
    emit_aarch64_pop_call_stack(stack_bytes, assembly)
}

pub(in crate::codegen) fn emit_aarch64_stack_arguments(
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    stack_bytes: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    const REGISTER_ARGS: usize = 8;
    if stack_bytes == 0 {
        return Ok(());
    }
    write_assembly!(assembly, "\tsub sp, sp, #{stack_bytes}\n")?;
    for (index, arg) in args.iter().enumerate().skip(REGISTER_ARGS) {
        let temporary_offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let adjusted_offset = temporary_offset
            .checked_add(stack_bytes)
            .ok_or_else(|| CompileError::new("call argument offset overflow"))?;
        let stack_offset = (index - REGISTER_ARGS)
            .checked_mul(TEMPORARY_BYTES)
            .ok_or_else(|| CompileError::new("call stack argument offset overflow"))?;
        let width = expr_width(arg);
        emit_aarch64_load_temporary_to_register(width, adjusted_offset, "17", assembly)?;
        let prefix = aarch64_register_prefix(width);
        write_assembly!(assembly, "\tstr {prefix}17, [sp, #{stack_offset}]\n")?;
    }
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_pop_call_stack(
    stack_bytes: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if stack_bytes == 0 {
        return Ok(());
    }
    write_assembly!(assembly, "\tadd sp, sp, #{stack_bytes}\n")
}
