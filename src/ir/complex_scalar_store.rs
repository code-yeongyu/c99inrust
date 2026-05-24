use super::{
    Instruction, LoweredExpr, LoweredLValue, LoweringContext, complex_binary_operands,
    complex_expr_scalar_type, complex_indirect_target, complex_lane_byte_size, complex_lane_expr,
    complex_lane_value_expr, complex_object_pointer, complex_unary_operand, scalar_size,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{ScalarType, UnaryOp};

impl LoweringContext {
    pub(in crate::ir) fn push_complex_object_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        value: LoweredExpr,
    ) -> CompileResult<()> {
        if let LoweredExpr::Comma { left, right } = value {
            self.instructions.push(Instruction::Eval(*left));
            return self.push_complex_object_store(pointer, scalar_type, *right);
        }
        if let LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } = value
        {
            return self.push_complex_conditional_store(
                pointer,
                scalar_type,
                *condition,
                *then_expr,
                *else_expr,
            );
        }
        if matches!(
            value,
            LoweredExpr::Call { .. } | LoweredExpr::IndirectCall { .. }
        ) {
            self.instructions.push(Instruction::Eval(value));
            self.instructions.push(Instruction::StoreComplexReturn {
                pointer: pointer.clone(),
                scalar_type,
            });
            return Ok(());
        }
        if let Some(source) = complex_object_pointer(&value, scalar_type) {
            return self.push_complex_object_copy(pointer, &source, scalar_type);
        }
        if let Some((op, source)) = complex_unary_operand(&value, scalar_type) {
            return self.push_complex_unary_store(pointer, scalar_type, op, &source);
        }
        if let Some((op, left, right)) = complex_binary_operands(&value, scalar_type) {
            return self.push_complex_binary_store(pointer, scalar_type, op, &left, &right);
        }
        if complex_expr_scalar_type(&value) == Some(scalar_type) {
            return self.push_complex_expr_store(pointer, scalar_type, &value);
        }
        self.push_complex_element_store(pointer, 0, complex_lane_byte_size(scalar_type), value)?;
        self.zero_complex_tail(pointer, scalar_type)
    }

    pub(in crate::ir) fn push_complex_indirect_store(
        &mut self,
        target: &LoweredLValue,
        value: LoweredExpr,
    ) -> CompileResult<bool> {
        let Some((pointer, scalar_type)) = complex_indirect_target(target) else {
            return Ok(false);
        };
        self.push_complex_object_store(&pointer, scalar_type, value)?;
        Ok(true)
    }

    pub(in crate::ir) fn push_complex_object_copy(
        &mut self,
        target_pointer: &LoweredExpr,
        source_pointer: &LoweredExpr,
        scalar_type: ScalarType,
    ) -> CompileResult<()> {
        let element_byte_size = complex_lane_byte_size(scalar_type);
        let tail_slots = scalar_size(scalar_type) / element_byte_size;
        for (index_value, _) in (0_i64..).zip(0..tail_slots) {
            self.push_complex_element_store(
                target_pointer,
                index_value,
                element_byte_size,
                complex_lane_expr(source_pointer, index_value, element_byte_size),
            )?;
        }
        Ok(())
    }

    fn push_complex_unary_store(
        &mut self,
        target_pointer: &LoweredExpr,
        scalar_type: ScalarType,
        op: UnaryOp,
        source_pointer: &LoweredExpr,
    ) -> CompileResult<()> {
        if op == UnaryOp::Plus {
            return self.push_complex_object_copy(target_pointer, source_pointer, scalar_type);
        }
        let element_byte_size = complex_lane_byte_size(scalar_type);
        let tail_slots = scalar_size(scalar_type) / element_byte_size;
        for (index_value, _) in (0_i64..).zip(0..tail_slots) {
            self.push_complex_element_store(
                target_pointer,
                index_value,
                element_byte_size,
                LoweredExpr::Unary {
                    op,
                    expr: Box::new(complex_lane_expr(
                        source_pointer,
                        index_value,
                        element_byte_size,
                    )),
                },
            )?;
        }
        Ok(())
    }

    fn zero_complex_tail(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
    ) -> CompileResult<()> {
        let element_byte_size = complex_lane_byte_size(scalar_type);
        let tail_slots = scalar_size(scalar_type) / element_byte_size;
        for (index_value, _) in (1_i64..).zip(1..tail_slots) {
            self.push_complex_element_store(
                pointer,
                index_value,
                element_byte_size,
                LoweredExpr::DoubleLiteral("0.0".to_owned()),
            )?;
        }
        Ok(())
    }

    fn push_complex_expr_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        value: &LoweredExpr,
    ) -> CompileResult<()> {
        let byte_size = scalar_size(scalar_type);
        let slot = self.declare_anonymous_slot(scalar_type, byte_size, byte_size)?;
        let temp_pointer = LoweredExpr::LocalAddress {
            offset: self.local_offset(slot)?,
            byte_size,
        };
        let element_byte_size = complex_lane_byte_size(scalar_type);
        let tail_slots = scalar_size(scalar_type) / element_byte_size;
        for (index_value, _) in (0_i64..).zip(0..tail_slots) {
            let lane = complex_lane_value_expr(value, scalar_type, index_value, element_byte_size)
                .ok_or_else(|| CompileError::new("complex expression lane is unsupported"))?;
            self.push_complex_element_store(&temp_pointer, index_value, element_byte_size, lane)?;
        }
        self.push_complex_object_copy(pointer, &temp_pointer, scalar_type)
    }

    pub(in crate::ir) fn push_complex_element_store(
        &mut self,
        pointer: &LoweredExpr,
        index: i64,
        element_byte_size: usize,
        value: LoweredExpr,
    ) -> CompileResult<()> {
        self.push_store(
            LoweredLValue::PointerSubscript {
                pointer: Box::new(pointer.clone()),
                index: Box::new(LoweredExpr::Integer(index)),
                element_type: ScalarType::Double,
                element_byte_size,
                element_unsigned: false,
            },
            value,
        )
    }

    fn push_complex_conditional_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        condition: LoweredExpr,
        then_expr: LoweredExpr,
        else_expr: LoweredExpr,
    ) -> CompileResult<()> {
        let else_label = self.fresh_label();
        let end_label = self.fresh_label();
        self.instructions.push(Instruction::JumpIfZero {
            condition,
            label: else_label,
        });
        self.push_complex_object_store(pointer, scalar_type, then_expr)?;
        self.instructions
            .push(Instruction::Jump { label: end_label });
        self.instructions
            .push(Instruction::Label { label: else_label });
        self.push_complex_object_store(pointer, scalar_type, else_expr)?;
        self.instructions
            .push(Instruction::Label { label: end_label });
        Ok(())
    }
}
