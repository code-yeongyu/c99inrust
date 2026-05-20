use super::aarch64_binary::emit_aarch64_i32_to_register;
use super::aarch64_expr::emit_aarch64_expr;
use super::frames::LabelAllocator;
use crate::diagnostics::CompileResult;
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_aarch64_logical_and(
    left: &LoweredExpr,
    right: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let false_label = labels.fresh();
    let end_label = labels.fresh();
    emit_aarch64_expr(left, temporary_base, depth, labels, assembly)?;
    assembly.push_str("\tcmp w0, #0\n");
    write_assembly!(assembly, "\tb.eq {false_label}\n")?;
    emit_aarch64_expr(right, temporary_base, depth, labels, assembly)?;
    assembly.push_str("\tcmp w0, #0\n");
    write_assembly!(assembly, "\tb.eq {false_label}\n")?;
    emit_aarch64_i32_to_register(1, "w0", assembly)?;
    write_assembly!(assembly, "\tb {end_label}\n")?;
    write_assembly!(assembly, "{false_label}:\n")?;
    emit_aarch64_i32_to_register(0, "w0", assembly)?;
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_logical_or(
    left: &LoweredExpr,
    right: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let true_label = labels.fresh();
    let end_label = labels.fresh();
    emit_aarch64_expr(left, temporary_base, depth, labels, assembly)?;
    assembly.push_str("\tcmp w0, #0\n");
    write_assembly!(assembly, "\tb.ne {true_label}\n")?;
    emit_aarch64_expr(right, temporary_base, depth, labels, assembly)?;
    assembly.push_str("\tcmp w0, #0\n");
    write_assembly!(assembly, "\tb.ne {true_label}\n")?;
    emit_aarch64_i32_to_register(0, "w0", assembly)?;
    write_assembly!(assembly, "\tb {end_label}\n")?;
    write_assembly!(assembly, "{true_label}:\n")?;
    emit_aarch64_i32_to_register(1, "w0", assembly)?;
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}
