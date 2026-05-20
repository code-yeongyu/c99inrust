use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

use super::frames::LabelAllocator;
use super::stack_helpers::x86_stack_object_offset;
use super::target::Target;
use super::widths::{ValueWidth, X86_64_VARIADIC_FP_OFFSET, expr_width};
use super::x86_64_binary::emit_x86_64_width_adjustment;
use super::x86_64_expr::emit_x86_64_expr_with_width;

pub(in crate::codegen) fn emit_x86_64_va_start(
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if args.len() != 2 {
        return Err(CompileError::new("va_start expects two arguments"));
    }
    let Some(frame) = labels.x86_64_variadic else {
        return Err(CompileError::new("va_start used outside variadic function"));
    };
    emit_x86_64_expr_with_width(
        &args[0],
        ValueWidth::I64,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmovq %rax, %r10\n");
    write_assembly!(assembly, "\tmovl ${}, 0(%r10)\n", frame.gp_offset)?;
    write_assembly!(assembly, "\tmovl ${X86_64_VARIADIC_FP_OFFSET}, 4(%r10)\n")?;
    write_assembly!(
        assembly,
        "\tleaq {}(%rbp), %rax\n",
        frame.overflow_arg_offset
    )?;
    assembly.push_str("\tmovq %rax, 8(%r10)\n");
    write_assembly!(
        assembly,
        "\tleaq {}(%rbp), %rax\n",
        x86_stack_object_offset(frame.register_save_offset, frame.register_save_size)
    )?;
    assembly.push_str("\tmovq %rax, 16(%r10)\n");
    assembly.push_str("\txorl %eax, %eax\n");
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_alloca(
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let [size] = args else {
        return Err(CompileError::new("alloca expects one argument"));
    };
    let width = expr_width(size);
    emit_x86_64_expr_with_width(size, width, temporary_base, depth, target, labels, assembly)?;
    emit_x86_64_width_adjustment(width, ValueWidth::I64, assembly);
    assembly.push_str("\taddq $15, %rax\n");
    assembly.push_str("\tandq $-16, %rax\n");
    assembly.push_str("\tsubq %rax, %rsp\n");
    assembly.push_str("\tmovq %rsp, %rax\n");
    Ok(())
}
