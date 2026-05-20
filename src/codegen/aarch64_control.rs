use super::aarch64_binary::{
    aarch64_zero_branch_for_comparison, emit_aarch64_compare_result_to_rhs,
};
use super::aarch64_conditionals::emit_aarch64_move_result_to_register;
use super::aarch64_expr::{emit_aarch64_expr, emit_aarch64_expr_with_width};
use super::aarch64_temporaries::{emit_aarch64_load_temporary, emit_aarch64_store_temporary};
use super::frames::LabelAllocator;
use super::widths::expr_width;
use crate::diagnostics::CompileResult;
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_aarch64_prologue(
    preserved_temp_offset: Option<usize>,
    link_register_offset: Option<usize>,
    stack_bytes: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if stack_bytes > 0 {
        write_assembly!(assembly, "\tsub sp, sp, #{stack_bytes}\n")?;
    }
    if let Some(offset) = link_register_offset {
        write_assembly!(assembly, "\tstr x30, [sp, #{offset}]\n")?;
    }
    if let Some(offset) = preserved_temp_offset {
        write_assembly!(assembly, "\tstr x19, [sp, #{offset}]\n")?;
    }
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_epilogue(
    preserved_temp_offset: Option<usize>,
    link_register_offset: Option<usize>,
    stack_bytes: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if let Some(offset) = preserved_temp_offset {
        write_assembly!(assembly, "\tldr x19, [sp, #{offset}]\n")?;
    }
    if let Some(offset) = link_register_offset {
        write_assembly!(assembly, "\tldr x30, [sp, #{offset}]\n")?;
    }
    if stack_bytes > 0 {
        write_assembly!(assembly, "\tadd sp, sp, #{stack_bytes}\n")?;
    }
    assembly.push_str("\tret\n");
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_jump_if_zero(
    condition: &LoweredExpr,
    target_label: &str,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if let LoweredExpr::Binary { op, left, right } = condition
        && let Some(branch) = aarch64_zero_branch_for_comparison(*op)
    {
        emit_aarch64_compare(left, right, temporary_base, labels, assembly)?;
        write_assembly!(assembly, "\t{branch} {target_label}\n")?;
        return Ok(());
    }
    emit_aarch64_expr(condition, temporary_base, 0, labels, assembly)?;
    assembly.push_str("\tcmp w0, #0\n");
    write_assembly!(assembly, "\tb.eq {target_label}\n")
}

pub(in crate::codegen) fn emit_aarch64_compare(
    left: &LoweredExpr,
    right: &LoweredExpr,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = expr_width(left).max(expr_width(right));
    emit_aarch64_expr_with_width(left, width, temporary_base, 1, labels, assembly)?;
    emit_aarch64_store_temporary(width, temporary_base, assembly)?;
    emit_aarch64_expr_with_width(right, width, temporary_base, 1, labels, assembly)?;
    emit_aarch64_move_result_to_register("1", width, assembly)?;
    emit_aarch64_load_temporary(width, temporary_base, assembly)?;
    emit_aarch64_compare_result_to_rhs(width, assembly)?;
    Ok(())
}
