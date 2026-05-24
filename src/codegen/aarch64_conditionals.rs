use super::aarch64_addressing::{aarch64_register_prefix, aarch64_result_register};
use super::aarch64_binary::emit_aarch64_i64_to_register;
use super::aarch64_expr::{emit_aarch64_expr, emit_aarch64_expr_with_width};
use super::aarch64_temporaries::{emit_aarch64_load_temporary, emit_aarch64_store_temporary};
use super::frames::LabelAllocator;
use super::widths::{ConditionalExpr, IndexSelectExpr, TEMPORARY_BYTES, ValueWidth, expr_width};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_aarch64_conditional(
    expr: ConditionalExpr<'_>,
    result_width: ValueWidth,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let else_label = labels.fresh();
    let end_label = labels.fresh();
    emit_aarch64_expr(expr.condition, temporary_base, depth, labels, assembly)?;
    emit_aarch64_compare_result_to_zero(expr_width(expr.condition), assembly)?;
    write_assembly!(assembly, "\tb.eq {else_label}\n")?;
    emit_aarch64_expr_with_width(
        expr.then_expr,
        result_width,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "\tb {end_label}\n")?;
    write_assembly!(assembly, "{else_label}:\n")?;
    emit_aarch64_expr_with_width(
        expr.else_expr,
        result_width,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_move_result_to_register(
    register: &str,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    if width == ValueWidth::F64 {
        return write_assembly!(assembly, "\tfmov d{register}, d0\n");
    }
    let prefix = aarch64_register_prefix(width);
    write_assembly!(assembly, "\tmov {prefix}{register}, {prefix}0\n")
}

pub(in crate::codegen) fn emit_aarch64_conditional_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
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
    emit_aarch64_conditional(
        ConditionalExpr {
            condition,
            then_expr,
            else_expr,
        },
        expr_width(expr),
        temporary_base,
        depth,
        labels,
        assembly,
    )
}

pub(in crate::codegen) fn emit_aarch64_index_select(
    expr: IndexSelectExpr<'_>,
    result_width: ValueWidth,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let index_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let end_label = labels.fresh();
    emit_aarch64_expr_with_width(
        expr.index,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, index_offset, assembly)?;
    for (case_index, case_expr) in expr.cases.iter().enumerate() {
        let next_label = labels.fresh();
        let case_index =
            i64::try_from(case_index).map_err(|_| CompileError::new("too many select cases"))?;
        emit_aarch64_load_temporary(ValueWidth::I64, index_offset, assembly)?;
        emit_aarch64_i64_to_register(case_index, "x16", assembly)?;
        assembly.push_str("\tcmp x0, x16\n");
        write_assembly!(assembly, "\tb.ne {next_label}\n")?;
        emit_aarch64_expr_with_width(
            case_expr,
            result_width,
            temporary_base,
            depth + 1,
            labels,
            assembly,
        )?;
        write_assembly!(assembly, "\tb {end_label}\n")?;
        write_assembly!(assembly, "{next_label}:\n")?;
    }
    emit_aarch64_expr_with_width(
        expr.default,
        result_width,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "{end_label}:\n")
}

pub(in crate::codegen) fn emit_aarch64_index_select_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
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
    emit_aarch64_index_select(
        IndexSelectExpr {
            index,
            cases,
            default,
        },
        expr_width(expr),
        temporary_base,
        depth,
        labels,
        assembly,
    )
}

pub(in crate::codegen) fn emit_aarch64_move_register_to_result(
    register: &str,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    if width == ValueWidth::F64 {
        return write_assembly!(assembly, "\tfmov d0, d{register}\n");
    }
    let prefix = aarch64_register_prefix(width);
    write_assembly!(assembly, "\tmov {prefix}0, {prefix}{register}\n")
}
pub(in crate::codegen) fn emit_aarch64_compare_result_to_zero(
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    match width {
        ValueWidth::I32 | ValueWidth::I64 => {
            let register = aarch64_result_register(width);
            write_assembly!(assembly, "\tcmp {register}, #0\n")
        }
        ValueWidth::F64 => {
            assembly.push_str("\tfcmp d0, #0.0\n");
            Ok(())
        }
    }
}
