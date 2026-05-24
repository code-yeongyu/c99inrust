use super::complex_abi::expr_complex_scalar_type;
use super::data_literals::label_name;
use super::frames::LabelAllocator;
use super::stack_helpers::call_stack_argument_bytes;
use super::target::Target;
use super::widths::{TEMPORARY_BYTES, ValueWidth, expr_width};
use super::x86_64_addressing::{
    x86_64_argument_register, x86_64_instruction_suffix, x86_64_stack_argument_scratch_register,
};
use super::x86_64_builtin_calls::{emit_x86_64_alloca, emit_x86_64_va_start};
use super::x86_64_complex_abi::emit_x86_64_complex_register_arguments;
use super::x86_64_expr::emit_x86_64_expr_with_width;
use super::x86_64_temporaries::{
    emit_x86_64_load_temporary_to_register, emit_x86_64_store_temporary,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_x86_64_call(
    callee: &str,
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    const MAX_REGISTER_ARGS: usize = 6;
    if callee == "alloca" {
        return emit_x86_64_alloca(args, temporary_base, depth, target, labels, assembly);
    }
    if callee == "va_start" {
        return emit_x86_64_va_start(args, temporary_base, depth, target, labels, assembly);
    }
    if callee == "va_end" {
        assembly.push_str("\txorl %eax, %eax\n");
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| expr_complex_scalar_type(arg).is_some())
    {
        emit_x86_64_complex_register_arguments(
            args,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        )?;
        write_assembly!(assembly, "\tcall {}\n", label_name(callee, target))?;
        return Ok(());
    }
    let register_count = args.len().min(MAX_REGISTER_ARGS);
    let stack_bytes = call_stack_argument_bytes(args.len(), MAX_REGISTER_ARGS)?;
    let arg_depth = depth + args.len();
    for (index, arg) in args.iter().enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        emit_x86_64_expr_with_width(
            arg,
            width,
            temporary_base,
            arg_depth,
            target,
            labels,
            assembly,
        )?;
        emit_x86_64_store_temporary(width, offset, assembly)?;
    }
    for (index, arg) in args.iter().enumerate().take(register_count) {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        let register = x86_64_argument_register(index, width)?;
        emit_x86_64_load_temporary_to_register(width, offset, register, assembly)?;
    }
    emit_x86_64_stack_arguments(
        args,
        temporary_base,
        depth,
        MAX_REGISTER_ARGS,
        stack_bytes,
        assembly,
    )?;
    write_assembly!(assembly, "\tcall {}\n", label_name(callee, target))?;
    emit_x86_64_pop_call_stack(stack_bytes, assembly)
}

pub(in crate::codegen) fn emit_x86_64_call_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Call { callee, args, .. } => emit_x86_64_call(
            callee,
            args,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::IndirectCall { callee, args, .. } => emit_x86_64_indirect_call(
            callee,
            args,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        _ => Err(CompileError::new(
            "internal error: expected x86_64 call expression",
        )),
    }
}

pub(in crate::codegen) fn emit_x86_64_indirect_call(
    callee: &LoweredExpr,
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    const MAX_REGISTER_ARGS: usize = 6;
    let register_count = args.len().min(MAX_REGISTER_ARGS);
    let stack_bytes = call_stack_argument_bytes(args.len(), MAX_REGISTER_ARGS)?;
    if args
        .iter()
        .any(|arg| expr_complex_scalar_type(arg).is_some())
    {
        emit_x86_64_complex_register_arguments(
            args,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        )?;
        emit_x86_64_expr_with_width(
            callee,
            ValueWidth::I64,
            temporary_base,
            depth + args.len() + 1,
            target,
            labels,
            assembly,
        )?;
        assembly.push_str("\tmovq %rax, %r11\n");
        assembly.push_str("\tcall *%r11\n");
        return Ok(());
    }
    let callee_offset = temporary_base + ((depth + args.len()) * TEMPORARY_BYTES);
    let arg_depth = depth + args.len() + 1;
    for (index, arg) in args.iter().enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        emit_x86_64_expr_with_width(
            arg,
            width,
            temporary_base,
            arg_depth,
            target,
            labels,
            assembly,
        )?;
        emit_x86_64_store_temporary(width, offset, assembly)?;
    }
    emit_x86_64_expr_with_width(
        callee,
        ValueWidth::I64,
        temporary_base,
        arg_depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, callee_offset, assembly)?;
    for (index, arg) in args.iter().enumerate().take(register_count) {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        let register = x86_64_argument_register(index, width)?;
        emit_x86_64_load_temporary_to_register(width, offset, register, assembly)?;
    }
    emit_x86_64_stack_arguments(
        args,
        temporary_base,
        depth,
        MAX_REGISTER_ARGS,
        stack_bytes,
        assembly,
    )?;
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, callee_offset, "%rax", assembly)?;
    write_assembly!(assembly, "\tcall *%rax\n")?;
    emit_x86_64_pop_call_stack(stack_bytes, assembly)
}

pub(in crate::codegen) fn emit_x86_64_stack_arguments(
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    register_args: usize,
    stack_bytes: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if stack_bytes == 0 {
        return Ok(());
    }
    write_assembly!(assembly, "\tsubq ${stack_bytes}, %rsp\n")?;
    for (index, arg) in args.iter().enumerate().skip(register_args) {
        let temporary_offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let stack_offset = (index - register_args)
            .checked_mul(TEMPORARY_BYTES)
            .ok_or_else(|| CompileError::new("call stack argument offset overflow"))?;
        let width = expr_width(arg);
        let register = x86_64_stack_argument_scratch_register(width);
        emit_x86_64_load_temporary_to_register(width, temporary_offset, register, assembly)?;
        let suffix = x86_64_instruction_suffix(width);
        write_assembly!(assembly, "\tmov{suffix} {register}, {stack_offset}(%rsp)\n")?;
    }
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_pop_call_stack(
    stack_bytes: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if stack_bytes == 0 {
        return Ok(());
    }
    write_assembly!(assembly, "\taddq ${stack_bytes}, %rsp\n")
}
