use super::aarch64_expr::emit_aarch64_expr_with_width;
use super::complex_abi::expr_complex_scalar_type;
use super::frames::LabelAllocator;
use super::widths::ValueWidth;
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{LoweredExpr, LoweredFunction};
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_aarch64_complex_argument(
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
            write_assembly!(assembly, "\tldr d{first_register}, [sp, #{offset}]\n")?;
            write_assembly!(
                assembly,
                "\tldr d{}, [sp, #{}]\n",
                first_register + 1,
                offset + 8
            )
        }
        _ => Err(CompileError::new(
            "complex argument currently requires an object value",
        )),
    }
}

pub(in crate::codegen) fn emit_aarch64_store_complex_return(
    pointer: &LoweredExpr,
    scalar_type: ScalarType,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if scalar_type != ScalarType::ComplexDouble {
        return Err(CompileError::new(
            "complex return store supports double only",
        ));
    }
    emit_aarch64_expr_with_width(
        pointer,
        ValueWidth::I64,
        temporary_base,
        0,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov x16, x0\n");
    assembly.push_str("\tstr d0, [x16]\n");
    assembly.push_str("\tstr d1, [x16, #8]\n");
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_complex_return_expr(
    expr: &LoweredExpr,
    assembly: &mut String,
) -> CompileResult<()> {
    let Some(ScalarType::ComplexDouble) = expr_complex_scalar_type(expr) else {
        return Err(CompileError::new("expected complex double return"));
    };
    match expr {
        LoweredExpr::Local { offset, .. } => {
            write_assembly!(assembly, "\tldr d0, [sp, #{offset}]\n")?;
            write_assembly!(assembly, "\tldr d1, [sp, #{}]\n", offset + 8)
        }
        _ => Err(CompileError::new(
            "complex return currently requires an object value",
        )),
    }
}

pub(in crate::codegen) fn emit_aarch64_complex_parameter_stores(
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
                    "\tstr d{float_register}, [sp, #{}]\n",
                    local_slot.offset
                )?;
                write_assembly!(
                    assembly,
                    "\tstr d{}, [sp, #{}]\n",
                    float_register + 1,
                    local_slot.offset + 8
                )?;
                float_register += 2;
            }
            ScalarType::Double | ScalarType::LongDouble => {
                write_assembly!(
                    assembly,
                    "\tstr d{float_register}, [sp, #{}]\n",
                    local_slot.offset
                )?;
                float_register += 1;
            }
            _ => {
                write_assembly!(
                    assembly,
                    "\tstr x{integer_register}, [sp, #{}]\n",
                    local_slot.offset
                )?;
                integer_register += 1;
            }
        }
    }
    Ok(())
}
