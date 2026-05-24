use super::complex_abi::expr_complex_scalar_type;
use super::frames::LabelAllocator;
use super::stack_helpers::{x86_stack_byte_offset, x86_stack_object_offset};
use super::target::Target;
use super::widths::{ValueWidth, expr_width};
use super::x86_64_complex_expr_args::{
    X86_64ComplexExpressionArg, emit_x86_64_complex_expression_argument,
};
use super::x86_64_expr::emit_x86_64_expr_with_width;
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_x86_64_complex_argument(
    arg: &LoweredExpr,
    first_register: usize,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let Some(scalar_type) = expr_complex_scalar_type(arg) else {
        return Err(CompileError::new("expected complex argument"));
    };
    let register_count = x86_64_complex_register_count(scalar_type)?;
    if first_register + register_count > 8 {
        return Err(CompileError::new(
            "too many complex function call arguments",
        ));
    }
    match (arg, scalar_type) {
        (LoweredExpr::Local { offset, .. }, ScalarType::ComplexFloat) => write_assembly!(
            assembly,
            "\tmovsd {}(%rbp), %xmm{first_register}\n",
            x86_stack_object_offset(*offset, 8)
        ),
        (LoweredExpr::Local { offset, .. }, ScalarType::ComplexDouble) => {
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
        _ => emit_x86_64_complex_expression_argument(
            arg,
            scalar_type,
            X86_64ComplexExpressionArg::new(first_register, target),
            temporary_base,
            depth,
            labels,
            assembly,
        ),
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
    match scalar_type {
        ScalarType::ComplexFloat => assembly.push_str("\tmovsd %xmm0, (%rcx)\n"),
        ScalarType::ComplexDouble => {
            assembly.push_str("\tmovsd %xmm0, (%rcx)\n");
            assembly.push_str("\tmovsd %xmm1, 8(%rcx)\n");
        }
        _ => {
            return Err(CompileError::new(
                "complex return store supports float and double only",
            ));
        }
    }
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_complex_return_expr(
    expr: &LoweredExpr,
    assembly: &mut String,
) -> CompileResult<()> {
    let Some(scalar_type) = expr_complex_scalar_type(expr) else {
        return Err(CompileError::new("expected complex return"));
    };
    match (expr, scalar_type) {
        (LoweredExpr::Local { offset, .. }, ScalarType::ComplexFloat) => write_assembly!(
            assembly,
            "\tmovsd {}(%rbp), %xmm0\n",
            x86_stack_object_offset(*offset, 8)
        ),
        (LoweredExpr::Local { offset, .. }, ScalarType::ComplexDouble) => {
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
        if let Some(scalar_type) = expr_complex_scalar_type(arg) {
            emit_x86_64_complex_argument(
                arg,
                float_register,
                temporary_base,
                depth,
                target,
                labels,
                assembly,
            )?;
            float_register += x86_64_complex_register_count(scalar_type)?;
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

fn x86_64_complex_register_count(scalar_type: ScalarType) -> CompileResult<usize> {
    match scalar_type {
        ScalarType::ComplexFloat => Ok(1),
        ScalarType::ComplexDouble => Ok(2),
        _ => Err(CompileError::new(
            "complex function ABI supports float and double only",
        )),
    }
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
