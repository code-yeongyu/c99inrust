use super::frames::LabelAllocator;
use super::target::Target;
use super::widths::{ConditionalExpr, ValueWidth, expr_width};
use super::x86_64_expr::{emit_x86_64_expr, emit_x86_64_expr_with_width};
use crate::diagnostics::CompileResult;

pub(in crate::codegen) fn emit_x86_64_conditional(
    expr: ConditionalExpr<'_>,
    result_width: ValueWidth,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let else_label = labels.fresh();
    let end_label = labels.fresh();
    emit_x86_64_expr(
        expr.condition,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_compare_result_to_zero(expr_width(expr.condition), assembly);
    write_assembly!(assembly, "\tje {else_label}\n")?;
    emit_x86_64_expr_with_width(
        expr.then_expr,
        result_width,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "\tjmp {end_label}\n")?;
    write_assembly!(assembly, "{else_label}:\n")?;
    emit_x86_64_expr_with_width(
        expr.else_expr,
        result_width,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}
pub(in crate::codegen) fn emit_x86_64_compare_result_to_zero(
    width: ValueWidth,
    assembly: &mut String,
) {
    match width {
        ValueWidth::I32 => assembly.push_str("\tcmpl $0, %eax\n"),
        ValueWidth::I64 => assembly.push_str("\tcmpq $0, %rax\n"),
        ValueWidth::F64 => {
            assembly.push_str("\txorpd %xmm1, %xmm1\n");
            assembly.push_str("\tucomisd %xmm1, %xmm0\n");
        }
    }
}
