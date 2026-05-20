use super::aarch64_conditionals::emit_aarch64_compare_result_to_zero;
use super::aarch64_expr::emit_aarch64_expr;
use super::frames::LabelAllocator;
use super::widths::{ValueWidth, expr_width};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;
use crate::parser::UnaryOp;

pub(in crate::codegen) fn emit_aarch64_unary_expr(
    op: UnaryOp,
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    emit_aarch64_expr(expr, temporary_base, depth, labels, assembly)?;
    let width = expr_width(expr);
    match op {
        UnaryOp::Plus => {}
        UnaryOp::Minus => match width {
            ValueWidth::I32 => assembly.push_str("\tneg w0, w0\n"),
            ValueWidth::I64 => assembly.push_str("\tneg x0, x0\n"),
            ValueWidth::F64 => assembly.push_str("\tfneg d0, d0\n"),
        },
        UnaryOp::BitNot => match width {
            ValueWidth::I32 => assembly.push_str("\tmvn w0, w0\n"),
            ValueWidth::I64 => assembly.push_str("\tmvn x0, x0\n"),
            ValueWidth::F64 => {
                return Err(CompileError::new("unsupported double bitwise operator"));
            }
        },
        UnaryOp::LogicalNot => {
            emit_aarch64_compare_result_to_zero(width, assembly)?;
            assembly.push_str("\tcset w0, eq\n");
        }
    }
    Ok(())
}
