use super::frames::LabelAllocator;
use super::stack_helpers::{x86_stack_byte_offset, x86_stack_object_offset};
use super::target::Target;
use super::widths::{TEMPORARY_BYTES, ValueWidth};
use super::x86_64_expr::emit_x86_64_expr_with_width;
use super::x86_64_temporaries::{
    emit_x86_64_load_temporary_to_register, emit_x86_64_store_temporary,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{LoweredExpr, complex_lane_byte_size, complex_lane_value_expr};
use crate::parser::ScalarType;

#[derive(Clone, Copy)]
pub(in crate::codegen) struct X86_64ComplexExpressionArg {
    first_register: usize,
    target: Target,
}

impl X86_64ComplexExpressionArg {
    pub(in crate::codegen) const fn new(first_register: usize, target: Target) -> Self {
        Self {
            first_register,
            target,
        }
    }
}

pub(in crate::codegen) fn emit_x86_64_complex_expression_argument(
    arg: &LoweredExpr,
    scalar_type: ScalarType,
    call: X86_64ComplexExpressionArg,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let lane_size = complex_lane_byte_size(scalar_type);
    let temp_offset = temporary_base + (depth * TEMPORARY_BYTES);
    for (index, lane_index) in [0_i64, 1_i64].into_iter().enumerate() {
        let lane = complex_lane_value_expr(arg, scalar_type, lane_index, lane_size)
            .ok_or_else(|| CompileError::new("complex argument lane is unsupported"))?;
        emit_x86_64_expr_with_width(
            &lane,
            ValueWidth::F64,
            temporary_base,
            depth + 2,
            call.target,
            labels,
            assembly,
        )?;
        store_complex_expression_lane(scalar_type, temp_offset, index, assembly)?;
    }
    load_complex_expression_registers(scalar_type, temp_offset, call.first_register, assembly)
}

fn store_complex_expression_lane(
    scalar_type: ScalarType,
    temp_offset: usize,
    index: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    match scalar_type {
        ScalarType::ComplexFloat => {
            assembly.push_str("\tcvtsd2ss %xmm0, %xmm0\n");
            write_assembly!(
                assembly,
                "\tmovss %xmm0, {}(%rbp)\n",
                x86_stack_byte_offset(temp_offset, 8, temp_offset + (index * 4))
            )
        }
        ScalarType::ComplexDouble => emit_x86_64_store_temporary(
            ValueWidth::F64,
            temp_offset + (index * TEMPORARY_BYTES),
            assembly,
        ),
        _ => Err(CompileError::new(
            "complex expression argument supports float and double only",
        )),
    }
}

fn load_complex_expression_registers(
    scalar_type: ScalarType,
    temp_offset: usize,
    first_register: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    match scalar_type {
        ScalarType::ComplexFloat => write_assembly!(
            assembly,
            "\tmovsd {}(%rbp), %xmm{first_register}\n",
            x86_stack_object_offset(temp_offset, 8)
        ),
        ScalarType::ComplexDouble => {
            emit_x86_64_load_temporary_to_register(
                ValueWidth::F64,
                temp_offset,
                &format!("%xmm{first_register}"),
                assembly,
            )?;
            emit_x86_64_load_temporary_to_register(
                ValueWidth::F64,
                temp_offset + TEMPORARY_BYTES,
                &format!("%xmm{}", first_register + 1),
                assembly,
            )
        }
        _ => Err(CompileError::new(
            "complex expression argument supports float and double only",
        )),
    }
}
