use super::frames::LabelAllocator;
use super::target::Target;
use super::widths::{BinaryExpr, TEMPORARY_BYTES, ValueWidth, binary_operand_width};
use super::x86_64_expr::{emit_x86_64_expr, emit_x86_64_expr_with_width};
use super::x86_64_temporaries::{
    emit_x86_64_load_temporary, emit_x86_64_move_result_to_rhs, emit_x86_64_store_temporary,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;
use crate::parser::BinaryOp;

pub(in crate::codegen) fn emit_x86_64_binary_expr(
    binary: BinaryExpr<'_>,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if binary.op == BinaryOp::LogicalAnd {
        return emit_x86_64_logical_and(
            binary.left,
            binary.right,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        );
    }
    if binary.op == BinaryOp::LogicalOr {
        return emit_x86_64_logical_or(
            binary.left,
            binary.right,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        );
    }
    let operand_width = binary_operand_width(binary.op, binary.left, binary.right);
    let temporary_offset = temporary_base + (depth * TEMPORARY_BYTES);
    emit_x86_64_expr_with_width(
        binary.left,
        operand_width,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(operand_width, temporary_offset, assembly)?;
    emit_x86_64_expr_with_width(
        binary.right,
        operand_width,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_move_result_to_rhs(operand_width, assembly);
    emit_x86_64_load_temporary(operand_width, temporary_offset, assembly)?;
    emit_x86_64_binary_op(binary.op, operand_width, assembly)?;
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_width_adjustment(
    actual_width: ValueWidth,
    target_width: ValueWidth,
    assembly: &mut String,
) {
    match (actual_width, target_width) {
        (ValueWidth::I32, ValueWidth::I64) => assembly.push_str("\tcltq\n"),
        (ValueWidth::I32, ValueWidth::F64) => assembly.push_str("\tcvtsi2sdl %eax, %xmm0\n"),
        (ValueWidth::I64, ValueWidth::F64) => assembly.push_str("\tcvtsi2sdq %rax, %xmm0\n"),
        (ValueWidth::F64, ValueWidth::I32) => assembly.push_str("\tcvttsd2sil %xmm0, %eax\n"),
        (ValueWidth::F64, ValueWidth::I64) => assembly.push_str("\tcvttsd2siq %xmm0, %rax\n"),
        _ => {}
    }
}
pub(in crate::codegen) fn emit_x86_64_binary_op(
    op: BinaryOp,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    match (op, width) {
        (BinaryOp::Mul, ValueWidth::I32) => assembly.push_str("\timull %ecx, %eax\n"),
        (BinaryOp::Mul, ValueWidth::I64) => assembly.push_str("\timulq %rcx, %rax\n"),
        (BinaryOp::Mul, ValueWidth::F64) => assembly.push_str("\tmulsd %xmm1, %xmm0\n"),
        (BinaryOp::Div, ValueWidth::I32) => {
            assembly.push_str("\tcltd\n");
            assembly.push_str("\tidivl %ecx\n");
        }
        (BinaryOp::Div, ValueWidth::I64) => {
            assembly.push_str("\tcqto\n");
            assembly.push_str("\tidivq %rcx\n");
        }
        (BinaryOp::Div, ValueWidth::F64) => assembly.push_str("\tdivsd %xmm1, %xmm0\n"),
        (BinaryOp::Mod, ValueWidth::I32) => {
            assembly.push_str("\tcltd\n");
            assembly.push_str("\tidivl %ecx\n");
            assembly.push_str("\tmovl %edx, %eax\n");
        }
        (BinaryOp::Mod, ValueWidth::I64) => {
            assembly.push_str("\tcqto\n");
            assembly.push_str("\tidivq %rcx\n");
            assembly.push_str("\tmovq %rdx, %rax\n");
        }
        (BinaryOp::Add, ValueWidth::I32) => assembly.push_str("\taddl %ecx, %eax\n"),
        (BinaryOp::Add, ValueWidth::I64) => assembly.push_str("\taddq %rcx, %rax\n"),
        (BinaryOp::Add, ValueWidth::F64) => assembly.push_str("\taddsd %xmm1, %xmm0\n"),
        (BinaryOp::Sub, ValueWidth::I32) => assembly.push_str("\tsubl %ecx, %eax\n"),
        (BinaryOp::Sub, ValueWidth::I64) => assembly.push_str("\tsubq %rcx, %rax\n"),
        (BinaryOp::Sub, ValueWidth::F64) => assembly.push_str("\tsubsd %xmm1, %xmm0\n"),
        (BinaryOp::ShiftLeft, ValueWidth::I32) => assembly.push_str("\tsall %cl, %eax\n"),
        (BinaryOp::ShiftLeft, ValueWidth::I64) => assembly.push_str("\tsalq %cl, %rax\n"),
        (BinaryOp::ShiftRight, ValueWidth::I32) => assembly.push_str("\tsarl %cl, %eax\n"),
        (BinaryOp::ShiftRight, ValueWidth::I64) => assembly.push_str("\tsarq %cl, %rax\n"),
        (BinaryOp::Less, _) => emit_x86_64_comparison("setl", width, assembly)?,
        (BinaryOp::LessEqual, _) => emit_x86_64_comparison("setle", width, assembly)?,
        (BinaryOp::Greater, _) => emit_x86_64_comparison("setg", width, assembly)?,
        (BinaryOp::GreaterEqual, _) => emit_x86_64_comparison("setge", width, assembly)?,
        (BinaryOp::Equal, _) => emit_x86_64_comparison("sete", width, assembly)?,
        (BinaryOp::NotEqual, _) => emit_x86_64_comparison("setne", width, assembly)?,
        (BinaryOp::BitAnd, ValueWidth::I32) => assembly.push_str("\tandl %ecx, %eax\n"),
        (BinaryOp::BitAnd, ValueWidth::I64) => assembly.push_str("\tandq %rcx, %rax\n"),
        (BinaryOp::BitXor, ValueWidth::I32) => assembly.push_str("\txorl %ecx, %eax\n"),
        (BinaryOp::BitXor, ValueWidth::I64) => assembly.push_str("\txorq %rcx, %rax\n"),
        (BinaryOp::BitOr, ValueWidth::I32) => assembly.push_str("\torl %ecx, %eax\n"),
        (BinaryOp::BitOr, ValueWidth::I64) => assembly.push_str("\torq %rcx, %rax\n"),
        (
            BinaryOp::Mod
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight
            | BinaryOp::BitAnd
            | BinaryOp::BitXor
            | BinaryOp::BitOr,
            ValueWidth::F64,
        ) => return Err(CompileError::new("unsupported double operator")),
        (BinaryOp::LogicalAnd | BinaryOp::LogicalOr, _) => {}
    }
    Ok(())
}
pub(in crate::codegen) fn emit_x86_64_logical_and(
    left: &LoweredExpr,
    right: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let false_label = labels.fresh();
    let end_label = labels.fresh();
    emit_x86_64_expr(left, temporary_base, depth, target, labels, assembly)?;
    assembly.push_str("\tcmpl $0, %eax\n");
    write_assembly!(assembly, "\tje {false_label}\n")?;
    emit_x86_64_expr(right, temporary_base, depth, target, labels, assembly)?;
    assembly.push_str("\tcmpl $0, %eax\n");
    write_assembly!(assembly, "\tje {false_label}\n")?;
    assembly.push_str("\tmovl $1, %eax\n");
    write_assembly!(assembly, "\tjmp {end_label}\n")?;
    write_assembly!(assembly, "{false_label}:\n")?;
    assembly.push_str("\tmovl $0, %eax\n");
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_logical_or(
    left: &LoweredExpr,
    right: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let true_label = labels.fresh();
    let end_label = labels.fresh();
    emit_x86_64_expr(left, temporary_base, depth, target, labels, assembly)?;
    assembly.push_str("\tcmpl $0, %eax\n");
    write_assembly!(assembly, "\tjne {true_label}\n")?;
    emit_x86_64_expr(right, temporary_base, depth, target, labels, assembly)?;
    assembly.push_str("\tcmpl $0, %eax\n");
    write_assembly!(assembly, "\tjne {true_label}\n")?;
    assembly.push_str("\tmovl $0, %eax\n");
    write_assembly!(assembly, "\tjmp {end_label}\n")?;
    write_assembly!(assembly, "{true_label}:\n")?;
    assembly.push_str("\tmovl $1, %eax\n");
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}
pub(in crate::codegen) fn emit_x86_64_comparison(
    instruction: &str,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    match width {
        ValueWidth::I32 => assembly.push_str("\tcmpl %ecx, %eax\n"),
        ValueWidth::I64 => assembly.push_str("\tcmpq %rcx, %rax\n"),
        ValueWidth::F64 => {
            let condition = match instruction {
                "setl" => "setb",
                "setle" => "setbe",
                "setg" => "seta",
                "setge" => "setae",
                "sete" => "sete",
                "setne" => "setne",
                _ => return Err(CompileError::new("unsupported comparison operator")),
            };
            assembly.push_str("\tucomisd %xmm1, %xmm0\n");
            write_assembly!(assembly, "\t{condition} %al\n")?;
            assembly.push_str("\tmovzbl %al, %eax\n");
            return Ok(());
        }
    }
    write_assembly!(assembly, "\t{instruction} %al\n")?;
    assembly.push_str("\tmovzbl %al, %eax\n");
    Ok(())
}
