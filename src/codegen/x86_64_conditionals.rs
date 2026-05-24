use super::frames::LabelAllocator;
use super::target::Target;
use super::widths::{ConditionalExpr, IndexSelectExpr, TEMPORARY_BYTES, ValueWidth, expr_width};
use super::x86_64_expr::{emit_x86_64_expr, emit_x86_64_expr_with_width};
use super::x86_64_temporaries::{emit_x86_64_load_temporary, emit_x86_64_store_temporary};
use crate::diagnostics::CompileResult;
use crate::ir::LoweredExpr;

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

pub(in crate::codegen) fn emit_x86_64_conditional_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let LoweredExpr::Conditional {
        condition,
        then_expr,
        else_expr,
    } = expr
    else {
        return Ok(());
    };
    emit_x86_64_conditional(
        ConditionalExpr {
            condition,
            then_expr,
            else_expr,
        },
        expr_width(expr),
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )
}

pub(in crate::codegen) fn emit_x86_64_index_select(
    expr: IndexSelectExpr<'_>,
    result_width: ValueWidth,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let index_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let end_label = labels.fresh();
    emit_x86_64_expr_with_width(
        expr.index,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, index_offset, assembly)?;
    for (case_index, case_expr) in expr.cases.iter().enumerate() {
        let next_label = labels.fresh();
        emit_x86_64_load_temporary(ValueWidth::I64, index_offset, assembly)?;
        write_assembly!(assembly, "\tcmpq ${case_index}, %rax\n")?;
        write_assembly!(assembly, "\tjne {next_label}\n")?;
        emit_x86_64_expr_with_width(
            case_expr,
            result_width,
            temporary_base,
            depth + 1,
            target,
            labels,
            assembly,
        )?;
        write_assembly!(assembly, "\tjmp {end_label}\n")?;
        write_assembly!(assembly, "{next_label}:\n")?;
    }
    emit_x86_64_expr_with_width(
        expr.default,
        result_width,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "{end_label}:\n")
}

pub(in crate::codegen) fn emit_x86_64_index_select_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let LoweredExpr::IndexSelect {
        index,
        cases,
        default,
    } = expr
    else {
        return Ok(());
    };
    emit_x86_64_index_select(
        IndexSelectExpr {
            index,
            cases,
            default,
        },
        expr_width(expr),
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )
}
