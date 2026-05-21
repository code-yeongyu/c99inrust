use super::aarch64_binary::i32_immediate;
use super::data_literals::emit_double_literal_data;
use super::frames::LabelAllocator;
use super::target::Target;
use super::widths::{ValueWidth, expr_width};
use super::x86_64_conditionals::emit_x86_64_compare_result_to_zero;
use super::x86_64_expr::emit_x86_64_expr;
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;
use crate::parser::UnaryOp;

pub(in crate::codegen) fn emit_x86_64_unary_expr(
    op: UnaryOp,
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    emit_x86_64_expr(expr, temporary_base, depth, target, labels, assembly)?;
    let width = expr_width(expr);
    match op {
        UnaryOp::Plus => {}
        UnaryOp::Minus => match width {
            ValueWidth::I32 => assembly.push_str("\tnegl %eax\n"),
            ValueWidth::I64 => assembly.push_str("\tnegq %rax\n"),
            ValueWidth::F64 => emit_x86_64_negate_f64(target, labels, assembly)?,
        },
        UnaryOp::BitNot => match width {
            ValueWidth::I32 => assembly.push_str("\tnotl %eax\n"),
            ValueWidth::I64 => assembly.push_str("\tnotq %rax\n"),
            ValueWidth::F64 => {
                return Err(CompileError::new("unsupported double bitwise operator"));
            }
        },
        UnaryOp::LogicalNot => {
            emit_x86_64_compare_result_to_zero(width, assembly);
            assembly.push_str("\tsete %al\n");
            assembly.push_str("\tmovzbl %al, %eax\n");
        }
    }
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_integer(
    value: i64,
    assembly: &mut String,
) -> CompileResult<()> {
    let value = i32_immediate(value)?;
    write_assembly!(assembly, "\tmovl ${value}, %eax\n")
}

pub(in crate::codegen) fn emit_x86_64_i64_integer(
    value: i64,
    assembly: &mut String,
) -> CompileResult<()> {
    write_assembly!(assembly, "\tmovabsq ${value}, %rax\n")
}

pub(in crate::codegen) fn emit_x86_64_negate_f64(
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = labels.fresh();
    write_assembly!(assembly, "\tmovsd {label}(%rip), %xmm1\n")?;
    assembly.push_str("\txorpd %xmm1, %xmm0\n");
    emit_double_literal_data(&label, 0x8000_0000_0000_0000, target, assembly)
}
