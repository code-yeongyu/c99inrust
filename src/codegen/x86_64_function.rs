use super::aarch64_analysis::instruction_depth;
use super::data_literals::{branch_label, label_name};
use super::frames::{LabelAllocator, X86_64VariadicFrame};
use super::stack_helpers::{
    align_to, local_offset, local_stack_bytes, x86_stack_byte_offset, x86_stack_offset,
};
use super::target::Target;
use super::widths::{
    TEMPORARY_BYTES, ValueWidth, X86_64_VARIADIC_GP_REGISTERS, X86_64_VARIADIC_GP_SAVE_BYTES,
    scalar_width,
};
use super::x86_64_addressing::{
    x86_64_argument_register, x86_64_instruction_suffix, x86_64_stack_argument_scratch_register,
};
use super::x86_64_expr::emit_x86_64_expr;
use super::x86_64_loads::emit_x86_64_store_global;
use super::x86_64_temporaries::{
    emit_x86_64_init_local_bytes, emit_x86_64_init_local_ints, emit_x86_64_store_result,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{Instruction, LoweredFunction};

pub(in crate::codegen) fn emit_x86_64_function(
    function: &LoweredFunction,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(&function.name, target);
    let temporary_count = function
        .instructions
        .iter()
        .map(instruction_depth)
        .max()
        .unwrap_or(0);
    let local_bytes = local_stack_bytes(function);
    let temporary_base = align_to(local_bytes, TEMPORARY_BYTES);
    let stack_bytes = align_to(temporary_base + (temporary_count * TEMPORARY_BYTES), 16);
    let mut labels = LabelAllocator::new(function, target);
    labels.x86_64_variadic = x86_64_variadic_frame(function)?;
    write_assembly!(assembly, ".globl {label}\n")?;
    write_assembly!(assembly, "{label}:\n")?;
    assembly.push_str("\tpushq %rbp\n");
    assembly.push_str("\tmovq %rsp, %rbp\n");
    if stack_bytes > 0 {
        write_assembly!(assembly, "\tsubq ${stack_bytes}, %rsp\n")?;
    }
    emit_x86_64_variadic_register_saves(function, assembly)?;
    emit_x86_64_parameter_stores(function, assembly)?;
    for instruction in &function.instructions {
        match instruction {
            Instruction::StoreLocal {
                slot: _,
                offset,
                scalar_type,
                value,
            } => {
                emit_x86_64_expr(value, temporary_base, 0, target, &mut labels, assembly)?;
                emit_x86_64_store_result(scalar_width(*scalar_type), *offset, assembly)?;
            }
            Instruction::InitLocalBytes { offset, values } => {
                emit_x86_64_init_local_bytes(*offset, values, assembly)?;
            }
            Instruction::InitLocalInts { offset, values } => {
                emit_x86_64_init_local_ints(*offset, values, assembly)?;
            }
            Instruction::StoreGlobal {
                name,
                scalar_type,
                value,
            } => {
                emit_x86_64_expr(value, temporary_base, 0, target, &mut labels, assembly)?;
                emit_x86_64_store_global(name, scalar_width(*scalar_type), target, assembly)?;
            }
            Instruction::JumpIfZero { condition, label } => {
                emit_x86_64_expr(condition, temporary_base, 0, target, &mut labels, assembly)?;
                assembly.push_str("\tcmpl $0, %eax\n");
                write_assembly!(
                    assembly,
                    "\tje {}\n",
                    branch_label(&function.name, *label, target)
                )?;
            }
            Instruction::Jump { label } => {
                write_assembly!(
                    assembly,
                    "\tjmp {}\n",
                    branch_label(&function.name, *label, target)
                )?;
            }
            Instruction::Label { label } => {
                write_assembly!(
                    assembly,
                    "{}:\n",
                    branch_label(&function.name, *label, target)
                )?;
            }
            Instruction::Eval(expr) => {
                emit_x86_64_expr(expr, temporary_base, 0, target, &mut labels, assembly)?;
            }
            Instruction::Return(expr) => {
                if let Some(expr) = expr {
                    emit_x86_64_expr(expr, temporary_base, 0, target, &mut labels, assembly)?;
                }
                assembly.push_str("\tleave\n");
                assembly.push_str("\tret\n");
            }
        }
    }
    Ok(())
}
pub(in crate::codegen) fn emit_x86_64_parameter_stores(
    function: &LoweredFunction,
    assembly: &mut String,
) -> CompileResult<()> {
    for slot in 0..function.parameter_count {
        let Some(local_slot) = function.local_slots.get(slot) else {
            return Err(CompileError::new("internal error: missing parameter slot"));
        };
        let width = scalar_width(local_slot.scalar_type);
        if let Some(register) = x86_64_parameter_register(slot, width) {
            write_assembly!(
                assembly,
                "\tmov{} {register}, {}(%rbp)\n",
                x86_64_instruction_suffix(width),
                x86_stack_offset(local_offset(function, slot)?, width)
            )?;
            continue;
        }
        let source_offset = x86_64_stack_parameter_offset(slot)?;
        let register = x86_64_stack_argument_scratch_register(width);
        write_assembly!(
            assembly,
            "\tmov{} {source_offset}(%rbp), {register}\n",
            x86_64_instruction_suffix(width)
        )?;
        write_assembly!(
            assembly,
            "\tmov{} {register}, {}(%rbp)\n",
            x86_64_instruction_suffix(width),
            x86_stack_offset(local_offset(function, slot)?, width)
        )?;
    }
    Ok(())
}

pub(in crate::codegen) fn x86_64_variadic_frame(
    function: &LoweredFunction,
) -> CompileResult<Option<X86_64VariadicFrame>> {
    let Some(slot) = function.variadic_save_slot else {
        return Ok(None);
    };
    let register_save_offset = local_offset(function, slot)?;
    let named_gp_args = function
        .parameter_count
        .min(X86_64_VARIADIC_GP_REGISTERS.len());
    let stack_named_args = function
        .parameter_count
        .saturating_sub(X86_64_VARIADIC_GP_REGISTERS.len());
    Ok(Some(X86_64VariadicFrame {
        gp_offset: named_gp_args
            .checked_mul(TEMPORARY_BYTES)
            .ok_or_else(|| CompileError::new("variadic gp offset overflow"))?,
        overflow_arg_offset: 16usize
            .checked_add(
                stack_named_args
                    .checked_mul(TEMPORARY_BYTES)
                    .ok_or_else(|| CompileError::new("variadic overflow offset overflow"))?,
            )
            .ok_or_else(|| CompileError::new("variadic overflow offset overflow"))?,
        register_save_offset,
        register_save_size: X86_64_VARIADIC_GP_SAVE_BYTES,
    }))
}

pub(in crate::codegen) fn emit_x86_64_variadic_register_saves(
    function: &LoweredFunction,
    assembly: &mut String,
) -> CompileResult<()> {
    let Some(frame) = x86_64_variadic_frame(function)? else {
        return Ok(());
    };
    for (index, register) in X86_64_VARIADIC_GP_REGISTERS.iter().enumerate() {
        let offset = frame
            .register_save_offset
            .checked_add(index * TEMPORARY_BYTES)
            .ok_or_else(|| CompileError::new("variadic register save offset overflow"))?;
        let destination =
            x86_stack_byte_offset(frame.register_save_offset, frame.register_save_size, offset);
        write_assembly!(assembly, "\tmovq {register}, {destination}(%rbp)\n")?;
    }
    Ok(())
}

pub(in crate::codegen) fn x86_64_parameter_register(
    index: usize,
    width: ValueWidth,
) -> Option<&'static str> {
    x86_64_argument_register(index, width).ok()
}

pub(in crate::codegen) fn x86_64_stack_parameter_offset(index: usize) -> CompileResult<usize> {
    const REGISTER_ARGS: usize = 6;
    const RETURN_ADDRESS_BYTES: usize = 8;
    const SAVED_BASE_POINTER_BYTES: usize = 8;
    let stack_index = index
        .checked_sub(REGISTER_ARGS)
        .ok_or_else(|| CompileError::new("internal error: register parameter has no stack slot"))?;
    stack_index
        .checked_mul(TEMPORARY_BYTES)
        .and_then(|offset| {
            offset
                .checked_add(RETURN_ADDRESS_BYTES)
                .and_then(|offset| offset.checked_add(SAVED_BASE_POINTER_BYTES))
        })
        .ok_or_else(|| CompileError::new("stack parameter offset overflow"))
}
