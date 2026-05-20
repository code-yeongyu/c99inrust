use super::aarch64_addressing::aarch64_register_prefix;
use super::aarch64_conditionals::{
    emit_aarch64_move_register_to_result, emit_aarch64_move_result_to_register,
};
use super::aarch64_expr::emit_aarch64_expr_with_width;
use super::aarch64_logical::{emit_aarch64_logical_and, emit_aarch64_logical_or};
use super::aarch64_temporaries::{emit_aarch64_load_temporary, emit_aarch64_store_temporary};
use super::call_usage::expr_is_direct_call;
use super::frames::LabelAllocator;
use super::widths::{BinaryExpr, TEMPORARY_BYTES, ValueWidth, binary_operand_width};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::BinaryOp;

pub(in crate::codegen) fn emit_aarch64_binary_expr(
    binary: BinaryExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if binary.op == BinaryOp::LogicalAnd {
        return emit_aarch64_logical_and(
            binary.left,
            binary.right,
            temporary_base,
            depth,
            labels,
            assembly,
        );
    }
    if binary.op == BinaryOp::LogicalOr {
        return emit_aarch64_logical_or(
            binary.left,
            binary.right,
            temporary_base,
            depth,
            labels,
            assembly,
        );
    }
    let operand_width = binary_operand_width(binary.op, binary.left, binary.right);
    let temporary_offset = temporary_base + (depth * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(
        binary.left,
        operand_width,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    if expr_is_direct_call(binary.right) {
        emit_aarch64_move_result_to_register("19", operand_width, assembly)?;
        emit_aarch64_expr_with_width(
            binary.right,
            operand_width,
            temporary_base,
            depth + 1,
            labels,
            assembly,
        )?;
        emit_aarch64_move_result_to_register("1", operand_width, assembly)?;
        emit_aarch64_move_register_to_result("19", operand_width, assembly)?;
    } else {
        emit_aarch64_store_temporary(operand_width, temporary_offset, assembly)?;
        emit_aarch64_expr_with_width(
            binary.right,
            operand_width,
            temporary_base,
            depth + 1,
            labels,
            assembly,
        )?;
        emit_aarch64_move_result_to_register("1", operand_width, assembly)?;
        emit_aarch64_load_temporary(operand_width, temporary_offset, assembly)?;
    }
    emit_aarch64_binary_op(binary.op, operand_width, assembly)?;
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_width_adjustment(
    actual_width: ValueWidth,
    target_width: ValueWidth,
    assembly: &mut String,
) {
    match (actual_width, target_width) {
        (ValueWidth::I32, ValueWidth::I64) => assembly.push_str("\tsxtw x0, w0\n"),
        (ValueWidth::I32, ValueWidth::F64) => assembly.push_str("\tscvtf d0, w0\n"),
        (ValueWidth::I64, ValueWidth::F64) => assembly.push_str("\tscvtf d0, x0\n"),
        (ValueWidth::F64, ValueWidth::I32) => assembly.push_str("\tfcvtzs w0, d0\n"),
        (ValueWidth::F64, ValueWidth::I64) => assembly.push_str("\tfcvtzs x0, d0\n"),
        _ => {}
    }
}
pub(in crate::codegen) fn emit_aarch64_binary_op(
    op: BinaryOp,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    match (op, width) {
        (BinaryOp::Mul, ValueWidth::I32) => assembly.push_str("\tmul w0, w0, w1\n"),
        (BinaryOp::Mul, ValueWidth::I64) => assembly.push_str("\tmul x0, x0, x1\n"),
        (BinaryOp::Mul, ValueWidth::F64) => assembly.push_str("\tfmul d0, d0, d1\n"),
        (BinaryOp::Div, ValueWidth::I32) => assembly.push_str("\tsdiv w0, w0, w1\n"),
        (BinaryOp::Div, ValueWidth::I64) => assembly.push_str("\tsdiv x0, x0, x1\n"),
        (BinaryOp::Div, ValueWidth::F64) => assembly.push_str("\tfdiv d0, d0, d1\n"),
        (BinaryOp::Mod, ValueWidth::I32) => {
            assembly.push_str("\tsdiv w2, w0, w1\n");
            assembly.push_str("\tmsub w0, w2, w1, w0\n");
        }
        (BinaryOp::Mod, ValueWidth::I64) => {
            assembly.push_str("\tsdiv x2, x0, x1\n");
            assembly.push_str("\tmsub x0, x2, x1, x0\n");
        }
        (BinaryOp::Add, ValueWidth::I32) => assembly.push_str("\tadd w0, w0, w1\n"),
        (BinaryOp::Add, ValueWidth::I64) => assembly.push_str("\tadd x0, x0, x1\n"),
        (BinaryOp::Add, ValueWidth::F64) => assembly.push_str("\tfadd d0, d0, d1\n"),
        (BinaryOp::Sub, ValueWidth::I32) => assembly.push_str("\tsub w0, w0, w1\n"),
        (BinaryOp::Sub, ValueWidth::I64) => assembly.push_str("\tsub x0, x0, x1\n"),
        (BinaryOp::Sub, ValueWidth::F64) => assembly.push_str("\tfsub d0, d0, d1\n"),
        (BinaryOp::ShiftLeft, ValueWidth::I32) => assembly.push_str("\tlsl w0, w0, w1\n"),
        (BinaryOp::ShiftLeft, ValueWidth::I64) => assembly.push_str("\tlsl x0, x0, x1\n"),
        (BinaryOp::ShiftRight, ValueWidth::I32) => assembly.push_str("\tasr w0, w0, w1\n"),
        (BinaryOp::ShiftRight, ValueWidth::I64) => assembly.push_str("\tasr x0, x0, x1\n"),
        (BinaryOp::Less, _) => emit_aarch64_comparison("lt", width, assembly)?,
        (BinaryOp::LessEqual, _) => emit_aarch64_comparison("le", width, assembly)?,
        (BinaryOp::Greater, _) => emit_aarch64_comparison("gt", width, assembly)?,
        (BinaryOp::GreaterEqual, _) => emit_aarch64_comparison("ge", width, assembly)?,
        (BinaryOp::Equal, _) => emit_aarch64_comparison("eq", width, assembly)?,
        (BinaryOp::NotEqual, _) => emit_aarch64_comparison("ne", width, assembly)?,
        (BinaryOp::BitAnd, ValueWidth::I32) => assembly.push_str("\tand w0, w0, w1\n"),
        (BinaryOp::BitAnd, ValueWidth::I64) => assembly.push_str("\tand x0, x0, x1\n"),
        (BinaryOp::BitXor, ValueWidth::I32) => assembly.push_str("\teor w0, w0, w1\n"),
        (BinaryOp::BitXor, ValueWidth::I64) => assembly.push_str("\teor x0, x0, x1\n"),
        (BinaryOp::BitOr, ValueWidth::I32) => assembly.push_str("\torr w0, w0, w1\n"),
        (BinaryOp::BitOr, ValueWidth::I64) => assembly.push_str("\torr x0, x0, x1\n"),
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
pub(in crate::codegen) fn emit_aarch64_comparison(
    condition: &str,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    emit_aarch64_compare_result_to_rhs(width, assembly)?;
    write_assembly!(assembly, "\tcset w0, {condition}\n")
}
pub(in crate::codegen) fn emit_aarch64_compare_result_to_rhs(
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    match width {
        ValueWidth::I32 | ValueWidth::I64 => {
            let prefix = aarch64_register_prefix(width);
            write_assembly!(assembly, "\tcmp {prefix}0, {prefix}1\n")
        }
        ValueWidth::F64 => {
            assembly.push_str("\tfcmp d0, d1\n");
            Ok(())
        }
    }
}

pub(in crate::codegen) fn emit_aarch64_i32_to_register(
    value: i64,
    register: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    let value = i32_immediate(value)?;
    let bits = u32::from_ne_bytes(value.to_ne_bytes());
    let low = bits & 0xffff;
    let high = (bits >> 16) & 0xffff;
    write_assembly!(assembly, "\tmovz {register}, #{low}\n")?;
    if high != 0 {
        write_assembly!(assembly, "\tmovk {register}, #{high}, lsl #16\n")?;
    }
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_i64_to_register(
    value: i64,
    register: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    let bits = u64::from_ne_bytes(value.to_ne_bytes());
    let low = bits & 0xffff;
    write_assembly!(assembly, "\tmovz {register}, #{low}\n")?;
    for shift in [16u32, 32, 48] {
        let part = (bits >> shift) & 0xffff;
        if part != 0 {
            write_assembly!(assembly, "\tmovk {register}, #{part}, lsl #{shift}\n")?;
        }
    }
    Ok(())
}

pub(in crate::codegen) fn i32_immediate(value: i64) -> CompileResult<i32> {
    if let Ok(value) = i32::try_from(value) {
        return Ok(value);
    }
    let value =
        u32::try_from(value).map_err(|_| CompileError::new("integer literal does not fit i32"))?;
    Ok(i32::from_ne_bytes(value.to_ne_bytes()))
}

pub(in crate::codegen) const fn aarch64_zero_branch_for_comparison(
    op: BinaryOp,
) -> Option<&'static str> {
    match op {
        BinaryOp::Less => Some("b.ge"),
        BinaryOp::LessEqual => Some("b.gt"),
        BinaryOp::Greater => Some("b.le"),
        BinaryOp::GreaterEqual => Some("b.lt"),
        BinaryOp::Equal => Some("b.ne"),
        BinaryOp::NotEqual => Some("b.eq"),
        BinaryOp::Mul
        | BinaryOp::Div
        | BinaryOp::Mod
        | BinaryOp::Add
        | BinaryOp::Sub
        | BinaryOp::ShiftLeft
        | BinaryOp::ShiftRight
        | BinaryOp::LogicalAnd
        | BinaryOp::LogicalOr
        | BinaryOp::BitAnd
        | BinaryOp::BitXor
        | BinaryOp::BitOr => None,
    }
}

pub(in crate::codegen) const fn aarch64_update_immediate(
    op: BinaryOp,
    value: i64,
) -> Option<(&'static str, u64)> {
    let magnitude = value.unsigned_abs();
    if magnitude > 4095 {
        return None;
    }
    match (op, value >= 0) {
        (BinaryOp::Add, true) | (BinaryOp::Sub, false) => Some(("add", magnitude)),
        (BinaryOp::Add, false) | (BinaryOp::Sub, true) => Some(("sub", magnitude)),
        _ => None,
    }
}
