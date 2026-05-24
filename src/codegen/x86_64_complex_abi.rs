use super::complex_abi::expr_complex_scalar_type;
use super::frames::LabelAllocator;
use super::stack_helpers::{x86_stack_byte_offset, x86_stack_object_offset};
use super::target::Target;
use super::widths::{ValueWidth, expr_width};
use super::x86_64_expr::emit_x86_64_expr_with_width;
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{LoweredExpr, LoweredFunction};
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_x86_64_complex_argument(
    arg: &LoweredExpr,
    first_register: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let Some(ScalarType::ComplexDouble) = expr_complex_scalar_type(arg) else {
        return Err(CompileError::new("expected complex double argument"));
    };
    if first_register + 1 >= 8 {
        return Err(CompileError::new(
            "too many complex function call arguments",
        ));
    }
    match arg {
        LoweredExpr::Local { offset, .. } => {
            write_assembly!(
                assembly,
                "\tmovsd {}(%rbp), %xmm{first_register}\n",
                x86_stack_object_offset(*offset, 16)
            )?;
            write_assembly!(
                assembly,
                "\tmovsd {}(%rbp), %xmm{}\n",
                x86_stack_byte_offset(*offset, 16, offset + 8),
                first_register + 1
            )
        }
        _ => Err(CompileError::new(
            "complex argument currently requires an object value",
        )),
    }
}

pub(in crate::codegen) fn emit_x86_64_store_complex_return(
    pointer: &LoweredExpr,
    scalar_type: ScalarType,
    temporary_base: usize,
    target: Target,
    labels: &mut super::frames::LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if scalar_type != ScalarType::ComplexDouble {
        return Err(CompileError::new(
            "complex return store supports double only",
        ));
    }
    emit_x86_64_expr_with_width(
        pointer,
        ValueWidth::I64,
        temporary_base,
        0,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmovq %rax, %rcx\n");
    assembly.push_str("\tmovsd %xmm0, (%rcx)\n");
    assembly.push_str("\tmovsd %xmm1, 8(%rcx)\n");
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_complex_return_expr(
    expr: &LoweredExpr,
    assembly: &mut String,
) -> CompileResult<()> {
    let Some(ScalarType::ComplexDouble) = expr_complex_scalar_type(expr) else {
        return Err(CompileError::new("expected complex double return"));
    };
    match expr {
        LoweredExpr::Local { offset, .. } => {
            write_assembly!(
                assembly,
                "\tmovsd {}(%rbp), %xmm0\n",
                x86_stack_object_offset(*offset, 16)
            )?;
            write_assembly!(
                assembly,
                "\tmovsd {}(%rbp), %xmm1\n",
                x86_stack_byte_offset(*offset, 16, offset + 8)
            )
        }
        _ => Err(CompileError::new(
            "complex return currently requires an object value",
        )),
    }
}

pub(in crate::codegen) fn emit_x86_64_complex_parameter_stores(
    function: &LoweredFunction,
    assembly: &mut String,
) -> CompileResult<()> {
    let mut float_register = 0usize;
    let mut integer_register = 0usize;
    for slot in 0..function.parameter_count {
        let Some(local_slot) = function.local_slots.get(slot) else {
            return Err(CompileError::new("internal error: missing parameter slot"));
        };
        match local_slot.scalar_type {
            ScalarType::ComplexDouble => {
                if float_register + 1 >= 8 {
                    return Err(CompileError::new("too many complex function parameters"));
                }
                write_assembly!(
                    assembly,
                    "\tmovsd %xmm{float_register}, {}(%rbp)\n",
                    x86_stack_object_offset(local_slot.offset, 16)
                )?;
                write_assembly!(
                    assembly,
                    "\tmovsd %xmm{}, {}(%rbp)\n",
                    float_register + 1,
                    x86_stack_byte_offset(local_slot.offset, 16, local_slot.offset + 8)
                )?;
                float_register += 2;
            }
            ScalarType::Double | ScalarType::LongDouble => {
                write_assembly!(
                    assembly,
                    "\tmovsd %xmm{float_register}, {}(%rbp)\n",
                    x86_stack_object_offset(local_slot.offset, 8)
                )?;
                float_register += 1;
            }
            _ => {
                write_assembly!(
                    assembly,
                    "\tmovq %{}, {}(%rbp)\n",
                    integer_register_name(integer_register)?,
                    x86_stack_object_offset(local_slot.offset, 8)
                )?;
                integer_register += 1;
            }
        }
    }
    Ok(())
}

fn integer_register_name(index: usize) -> CompileResult<&'static str> {
    const REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
    REGISTERS
        .get(index)
        .copied()
        .ok_or_else(|| CompileError::new("too many function parameters"))
}

pub(in crate::codegen) fn emit_x86_64_complex_register_arguments(
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let mut float_register = 0usize;
    let mut integer_register = 0usize;
    for arg in args {
        if expr_complex_scalar_type(arg).is_some() {
            emit_x86_64_complex_argument(arg, float_register, assembly)?;
            float_register += 2;
            continue;
        }
        let width = expr_width(arg);
        emit_x86_64_expr_with_width(
            arg,
            width,
            temporary_base,
            depth + 1,
            target,
            labels,
            assembly,
        )?;
        match width {
            ValueWidth::F64 => {
                write_assembly!(assembly, "\tmovsd %xmm0, %xmm{float_register}\n")?;
                float_register += 1;
            }
            ValueWidth::I64 => {
                write_assembly!(
                    assembly,
                    "\tmovq %rax, {}\n",
                    call_integer_register_name(integer_register)?
                )?;
                integer_register += 1;
            }
            ValueWidth::I32 => {
                write_assembly!(
                    assembly,
                    "\tmovl %eax, {}\n",
                    call_integer_register_name_32(integer_register)?
                )?;
                integer_register += 1;
            }
        }
    }
    Ok(())
}

fn call_integer_register_name(index: usize) -> CompileResult<&'static str> {
    const REGISTERS: [&str; 6] = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];
    REGISTERS
        .get(index)
        .copied()
        .ok_or_else(|| CompileError::new("too many function call arguments"))
}

fn call_integer_register_name_32(index: usize) -> CompileResult<&'static str> {
    const REGISTERS: [&str; 6] = ["%edi", "%esi", "%edx", "%ecx", "%r8d", "%r9d"];
    REGISTERS
        .get(index)
        .copied()
        .ok_or_else(|| CompileError::new("too many function call arguments"))
}
