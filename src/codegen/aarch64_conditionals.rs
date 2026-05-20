use super::aarch64_addressing::{aarch64_register_prefix, aarch64_result_register};
use super::aarch64_expr::{emit_aarch64_expr, emit_aarch64_expr_with_width};
use super::frames::LabelAllocator;
use super::widths::{ConditionalExpr, ValueWidth, expr_width};
use crate::diagnostics::CompileResult;

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
