use super::aarch64_analysis::instruction_depth;
use super::data_literals::{branch_label, label_name};
use super::frames::LabelAllocator;
use super::stack_helpers::{align_to, local_offset, local_stack_bytes, x86_stack_offset};
use super::target::Target;
use super::widths::{TEMPORARY_BYTES, ValueWidth, scalar_width};
use super::x86_64_addressing::{
    x86_64_argument_register, x86_64_instruction_suffix, x86_64_stack_argument_scratch_register,
};
use super::x86_64_expr::{emit_x86_64_expr, emit_x86_64_expr_with_width};
use super::x86_64_loads::{emit_x86_64_store_global, emit_x86_64_store_global_bool};
use super::x86_64_temporaries::{
    emit_x86_64_init_local_bytes, emit_x86_64_init_local_ints, emit_x86_64_store_result,
};
use super::x86_64_variadic::{emit_x86_64_variadic_register_saves, x86_64_variadic_frame};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{Instruction, LoweredExpr, LoweredFunction};
use crate::parser::ScalarType;

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
                let width = scalar_width(*scalar_type);
                emit_x86_64_expr_with_width(
                    value,
                    width,
                    temporary_base,
                    0,
                    target,
                    &mut labels,
                    assembly,
                )?;
                emit_x86_64_store_result(width, *offset, assembly)?;
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
            } => emit_x86_64_store_global_instruction(
                name,
                *scalar_type,
                value,
                temporary_base,
                target,
                &mut labels,
                assembly,
            )?,
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

fn emit_x86_64_store_global_instruction(
    name: &str,
    scalar_type: ScalarType,
    value: &LoweredExpr,
    temporary_base: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(scalar_type);
    emit_x86_64_expr_with_width(value, width, temporary_base, 0, target, labels, assembly)?;
    if scalar_type == ScalarType::Bool {
        emit_x86_64_store_global_bool(name, target, assembly)
    } else {
        emit_x86_64_store_global(name, width, target, assembly)
    }
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
