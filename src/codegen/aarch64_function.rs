use super::aarch64_analysis::should_share_aarch64_epilogue;
use super::aarch64_complex_abi::{
    emit_aarch64_complex_return_expr, emit_aarch64_store_complex_return,
};
use super::aarch64_control::{
    emit_aarch64_epilogue, emit_aarch64_jump_if_zero, emit_aarch64_prologue,
};
use super::aarch64_expr::emit_aarch64_expr;
use super::aarch64_function_params::{
    emit_aarch64_parameter_stores, emit_aarch64_store_local_instruction,
};
use super::aarch64_loads::{emit_aarch64_store_global, emit_aarch64_store_global_bool};
use super::aarch64_temporaries::{emit_aarch64_init_local_bytes, emit_aarch64_init_local_ints};
use super::aarch64_variadic::{aarch64_variadic_frame, emit_aarch64_variadic_register_saves};
use super::complex_abi::return_complex_scalar_type;
use super::data_literals::{branch_label, label_name};
use super::frames::{Aarch64Epilogue, Aarch64Frame, LabelAllocator};
use super::target::Target;
use super::widths::scalar_width;
use crate::diagnostics::CompileResult;
use crate::ir::{Instruction, LoweredExpr, LoweredFunction};
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_aarch64_function(
    function: &LoweredFunction,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(&function.name, target);
    let frame = Aarch64Frame::new(function);
    let mut labels = aarch64_label_allocator(function, target, frame.stack_bytes)?;
    let shared_epilogue = if should_share_aarch64_epilogue(function, frame.stack_bytes) {
        Some(labels.fresh())
    } else {
        None
    };
    emit_aarch64_function_header(function, &label, &frame, &labels, assembly)?;
    emit_aarch64_instructions(
        function,
        target,
        &frame,
        shared_epilogue.as_deref(),
        &mut labels,
        assembly,
    )?;
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

fn emit_aarch64_instructions(
    function: &LoweredFunction,
    target: Target,
    frame: &Aarch64Frame,
    shared_label: Option<&str>,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
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
                labels,
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
            } => emit_aarch64_store_global_instruction(
                name,
                *scalar_type,
                value,
                frame.temporary_base,
                target,
                labels,
                assembly,
            )?,
            Instruction::StoreComplexReturn {
                pointer,
                scalar_type,
            } => emit_aarch64_store_complex_return(
                pointer,
                *scalar_type,
                frame.temporary_base,
                labels,
                assembly,
            )?,
            Instruction::JumpIfZero { condition, label } => {
                let target_label = branch_label(&function.name, *label, target);
                emit_aarch64_jump_if_zero(
                    condition,
                    &target_label,
                    frame.temporary_base,
                    labels,
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
                emit_aarch64_expr(expr, frame.temporary_base, 0, labels, assembly)?;
            }
            Instruction::Return(expr) => {
                emit_aarch64_function_return(
                    function,
                    expr.as_ref(),
                    frame,
                    shared_label,
                    labels,
                    assembly,
                )?;
            }
        }
    }
    Ok(())
}

fn emit_aarch64_function_return(
    function: &LoweredFunction,
    expr: Option<&LoweredExpr>,
    frame: &Aarch64Frame,
    shared_label: Option<&str>,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    emit_aarch64_return(
        expr,
        return_complex_scalar_type(function.return_type),
        Aarch64Epilogue {
            preserved_temp_offset: frame.preserved_temp_offset,
            link_register_offset: frame.link_register_offset,
            stack_bytes: frame.stack_bytes,
            shared_label,
        },
        frame.temporary_base,
        labels,
        assembly,
    )
}

fn emit_aarch64_function_header(
    function: &LoweredFunction,
    label: &str,
    frame: &Aarch64Frame,
    labels: &LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    write_assembly!(assembly, ".globl {label}\n")?;
    assembly.push_str(".p2align 2\n");
    write_assembly!(assembly, "{label}:\n")?;
    emit_aarch64_prologue(
        frame.preserved_temp_offset,
        frame.link_register_offset,
        frame.stack_bytes,
        assembly,
    )?;
    emit_aarch64_variadic_register_saves(function, labels.aarch64_variadic, assembly)?;
    emit_aarch64_parameter_stores(function, assembly)
}

fn emit_aarch64_store_global_instruction(
    name: &str,
    scalar_type: ScalarType,
    value: &LoweredExpr,
    temporary_base: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    emit_aarch64_expr(value, temporary_base, 0, labels, assembly)?;
    if scalar_type == ScalarType::Bool {
        emit_aarch64_store_global_bool(name, target, assembly)
    } else {
        emit_aarch64_store_global(name, scalar_width(scalar_type), target, assembly)
    }
}

fn aarch64_label_allocator(
    function: &LoweredFunction,
    target: Target,
    stack_bytes: usize,
) -> CompileResult<LabelAllocator<'_>> {
    let mut labels = LabelAllocator::new(function, target);
    labels.aarch64_variadic = aarch64_variadic_frame(function, stack_bytes)?;
    Ok(labels)
}

pub(in crate::codegen) fn emit_aarch64_return(
    expr: Option<&LoweredExpr>,
    complex_return: Option<ScalarType>,
    epilogue: Aarch64Epilogue<'_>,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if let Some(expr) = expr {
        if complex_return.is_some() {
            emit_aarch64_complex_return_expr(expr, assembly)?;
        } else {
            emit_aarch64_expr(expr, temporary_base, 0, labels, assembly)?;
        }
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
