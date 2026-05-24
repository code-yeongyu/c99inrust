use super::frames::LabelAllocator;
use super::stack_helpers::{x86_stack_byte_offset, x86_stack_object_offset};
use super::target::Target;
use super::widths::{TEMPORARY_BYTES, ValueWidth, expr_width};
use super::x86_64_conditionals::emit_x86_64_compare_result_to_zero;
use super::x86_64_expr::{emit_x86_64_expr, emit_x86_64_expr_with_width};
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
    let temp_offset = temporary_base + (depth * TEMPORARY_BYTES);
    {
        let mut materializer = X86_64ComplexArgMaterializer {
            scalar_type,
            temporary_base,
            depth,
            target: call.target,
            labels,
            assembly,
        };
        materializer.emit_value_to_temp(arg, temp_offset)?;
    }
    load_complex_expression_registers(scalar_type, temp_offset, call.first_register, assembly)
}

struct X86_64ComplexArgMaterializer<'a, 'label> {
    scalar_type: ScalarType,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &'a mut LabelAllocator<'label>,
    assembly: &'a mut String,
}

impl X86_64ComplexArgMaterializer<'_, '_> {
    fn emit_value_to_temp(&mut self, arg: &LoweredExpr, temp_offset: usize) -> CompileResult<()> {
        if let LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } = arg
        {
            return self.emit_conditional_value_to_temp(
                condition,
                then_expr,
                else_expr,
                temp_offset,
            );
        }
        if let LoweredExpr::Comma { left, right } = arg {
            return self.emit_comma_value_to_temp(left, right, temp_offset);
        }
        self.emit_lanes_to_temp(arg, temp_offset)
    }

    fn emit_comma_value_to_temp(
        &mut self,
        left: &LoweredExpr,
        right: &LoweredExpr,
        temp_offset: usize,
    ) -> CompileResult<()> {
        emit_x86_64_expr(
            left,
            self.temporary_base,
            self.depth + 1,
            self.target,
            self.labels,
            self.assembly,
        )?;
        self.emit_value_to_temp(right, temp_offset)
    }

    fn emit_conditional_value_to_temp(
        &mut self,
        condition: &LoweredExpr,
        then_expr: &LoweredExpr,
        else_expr: &LoweredExpr,
        temp_offset: usize,
    ) -> CompileResult<()> {
        let else_label = self.labels.fresh();
        let end_label = self.labels.fresh();
        emit_x86_64_expr(
            condition,
            self.temporary_base,
            self.depth + 1,
            self.target,
            self.labels,
            self.assembly,
        )?;
        emit_x86_64_compare_result_to_zero(expr_width(condition), self.assembly);
        write_assembly!(self.assembly, "\tje {else_label}\n")?;
        self.emit_value_to_temp(then_expr, temp_offset)?;
        write_assembly!(self.assembly, "\tjmp {end_label}\n")?;
        write_assembly!(self.assembly, "{else_label}:\n")?;
        self.emit_value_to_temp(else_expr, temp_offset)?;
        write_assembly!(self.assembly, "{end_label}:\n")
    }

    fn emit_lanes_to_temp(&mut self, arg: &LoweredExpr, temp_offset: usize) -> CompileResult<()> {
        let lane_size = complex_lane_byte_size(self.scalar_type);
        for (index, lane_index) in [0_i64, 1_i64].into_iter().enumerate() {
            let lane = complex_lane_value_expr(arg, self.scalar_type, lane_index, lane_size)
                .ok_or_else(|| CompileError::new("complex argument lane is unsupported"))?;
            emit_x86_64_expr_with_width(
                &lane,
                ValueWidth::F64,
                self.temporary_base,
                self.depth + 2,
                self.target,
                self.labels,
                self.assembly,
            )?;
            self.store_lane(index, temp_offset)?;
        }
        Ok(())
    }

    fn store_lane(&mut self, index: usize, temp_offset: usize) -> CompileResult<()> {
        match self.scalar_type {
            ScalarType::ComplexFloat => {
                self.assembly.push_str("\tcvtsd2ss %xmm0, %xmm0\n");
                write_assembly!(
                    self.assembly,
                    "\tmovss %xmm0, {}(%rbp)\n",
                    x86_stack_byte_offset(temp_offset, 8, temp_offset + (index * 4))
                )
            }
            ScalarType::ComplexDouble => emit_x86_64_store_temporary(
                ValueWidth::F64,
                temp_offset + (index * TEMPORARY_BYTES),
                self.assembly,
            ),
            _ => Err(CompileError::new(
                "complex expression argument supports float and double only",
            )),
        }
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
