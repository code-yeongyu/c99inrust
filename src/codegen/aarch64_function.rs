use super::aarch64_addressing::{aarch64_parameter_register, emit_aarch64_store_local};
use super::aarch64_analysis::should_share_aarch64_epilogue;
use super::aarch64_control::{
    emit_aarch64_epilogue, emit_aarch64_jump_if_zero, emit_aarch64_prologue,
};
use super::aarch64_expr::emit_aarch64_expr;
use super::aarch64_loads::emit_aarch64_store_global;
use super::aarch64_temporaries::{
    emit_aarch64_init_local_bytes, emit_aarch64_init_local_ints, emit_aarch64_store_result,
};
use super::data_literals::{branch_label, label_name};
use super::frames::{Aarch64Epilogue, Aarch64Frame, LabelAllocator};
use super::stack_helpers::local_offset;
use super::target::Target;
use super::widths::scalar_width;
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{Instruction, LoweredExpr, LoweredFunction};
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_aarch64_function(
    function: &LoweredFunction,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(&function.name, target);
    let frame = Aarch64Frame::new(function);
    let mut labels = LabelAllocator::new(function, target);
    let shared_epilogue = if should_share_aarch64_epilogue(function, frame.stack_bytes) {
        Some(labels.fresh())
    } else {
        None
    };
    write_assembly!(assembly, ".globl {label}\n")?;
    assembly.push_str(".p2align 2\n");
    write_assembly!(assembly, "{label}:\n")?;
    emit_aarch64_prologue(
        frame.preserved_temp_offset,
        frame.link_register_offset,
        frame.stack_bytes,
        assembly,
    )?;
    emit_aarch64_parameter_stores(function, assembly)?;
    for instruction in &function.instructions {
        match instruction {
            Instruction::StoreLocal {
                slot,
                offset,
                scalar_type,
                value,
            } => emit_aarch64_store_local_instruction(
                (*slot, *offset, *scalar_type),
                value,
                frame.temporary_base,
                &mut labels,
                assembly,
            )?,
            Instruction::InitLocalBytes { offset, values } => {
                emit_aarch64_init_local_bytes(*offset, values, assembly)?;
            }
            Instruction::InitLocalInts { offset, values } => {
                emit_aarch64_init_local_ints(*offset, values, assembly)?;
            }
            Instruction::StoreGlobal {
                name,
                scalar_type,
                value,
            } => {
                emit_aarch64_expr(value, frame.temporary_base, 0, &mut labels, assembly)?;
                emit_aarch64_store_global(name, scalar_width(*scalar_type), target, assembly)?;
            }
            Instruction::JumpIfZero { condition, label } => {
                let target_label = branch_label(&function.name, *label, target);
                emit_aarch64_jump_if_zero(
                    condition,
                    &target_label,
                    frame.temporary_base,
                    &mut labels,
                    assembly,
                )?;
            }
            Instruction::Jump { label } => {
                write_assembly!(
                    assembly,
                    "\tb {}\n",
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
                emit_aarch64_expr(expr, frame.temporary_base, 0, &mut labels, assembly)?;
            }
            Instruction::Return(expr) => {
                emit_aarch64_return(
                    expr.as_ref(),
                    Aarch64Epilogue {
                        preserved_temp_offset: frame.preserved_temp_offset,
                        link_register_offset: frame.link_register_offset,
                        stack_bytes: frame.stack_bytes,
                        shared_label: shared_epilogue.as_deref(),
                    },
                    frame.temporary_base,
                    &mut labels,
                    assembly,
                )?;
            }
        }
    }
    if let Some(label) = shared_epilogue {
        write_assembly!(assembly, "{label}:\n")?;
        emit_aarch64_epilogue(
            frame.preserved_temp_offset,
            frame.link_register_offset,
            frame.stack_bytes,
            assembly,
        )?;
    }
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_store_local_instruction(
    local: (usize, usize, ScalarType),
    value: &LoweredExpr,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let (slot, offset, scalar_type) = local;
    emit_aarch64_store_local(
        slot,
        offset,
        scalar_type,
        value,
        temporary_base,
        labels,
        assembly,
    )?;
    emit_aarch64_store_result(scalar_width(scalar_type), offset, assembly)
}
pub(in crate::codegen) fn emit_aarch64_return(
    expr: Option<&LoweredExpr>,
    epilogue: Aarch64Epilogue<'_>,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if let Some(expr) = expr {
        emit_aarch64_expr(expr, temporary_base, 0, labels, assembly)?;
    }
    if let Some(label) = epilogue.shared_label {
        write_assembly!(assembly, "\tb {label}\n")?;
        return Ok(());
    }
    emit_aarch64_epilogue(
        epilogue.preserved_temp_offset,
        epilogue.link_register_offset,
        epilogue.stack_bytes,
        assembly,
    )
}
pub(in crate::codegen) fn emit_aarch64_parameter_stores(
    function: &LoweredFunction,
    assembly: &mut String,
) -> CompileResult<()> {
    const MAX_REGISTER_ARGS: usize = 8;
    if function.parameter_count > MAX_REGISTER_ARGS {
        return Err(CompileError::new("too many function parameters"));
    }
    for slot in 0..function.parameter_count {
        let Some(local_slot) = function.local_slots.get(slot) else {
            return Err(CompileError::new("internal error: missing parameter slot"));
        };
        let width = scalar_width(local_slot.scalar_type);
        let register = aarch64_parameter_register(slot, width);
        write_assembly!(
            assembly,
            "\tstr {register}, [sp, #{}]\n",
            local_offset(function, slot)?
        )?;
    }
    Ok(())
}
