use super::{LoweredExpr, LoweringContext, complex_lane_byte_size, complex_lane_expr, scalar_size};
use crate::diagnostics::CompileResult;
use crate::parser::{BinaryOp, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn push_complex_binary_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        op: BinaryOp,
        left: &LoweredExpr,
        right: &LoweredExpr,
    ) -> CompileResult<()> {
        match op {
            BinaryOp::Add | BinaryOp::Sub => {
                self.push_complex_linear_binary_store(pointer, scalar_type, op, left, right)
            }
            BinaryOp::Mul => self.push_complex_mul_store(pointer, scalar_type, left, right),
            BinaryOp::Div => self.push_complex_div_store(pointer, scalar_type, left, right),
            _ => self.push_complex_linear_binary_store(pointer, scalar_type, op, left, right),
        }
    }

    fn push_complex_linear_binary_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        op: BinaryOp,
        left: &LoweredExpr,
        right: &LoweredExpr,
    ) -> CompileResult<()> {
        let element_byte_size = complex_lane_byte_size(scalar_type);
        let tail_slots = scalar_size(scalar_type) / element_byte_size;
        for (index_value, _) in (0_i64..).zip(0..tail_slots) {
            self.push_complex_element_store(
                pointer,
                index_value,
                element_byte_size,
                binary(
                    op,
                    complex_lane_expr(left, index_value, element_byte_size),
                    complex_lane_expr(right, index_value, element_byte_size),
                ),
            )?;
        }
        Ok(())
    }

    fn push_complex_mul_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        left: &LoweredExpr,
        right: &LoweredExpr,
    ) -> CompileResult<()> {
        let element_byte_size = complex_lane_byte_size(scalar_type);
        let a = complex_lane_expr(left, 0, element_byte_size);
        let b = complex_lane_expr(left, 1, element_byte_size);
        let c = complex_lane_expr(right, 0, element_byte_size);
        let d = complex_lane_expr(right, 1, element_byte_size);
        let real = binary(
            BinaryOp::Sub,
            binary(BinaryOp::Mul, a.clone(), c.clone()),
            binary(BinaryOp::Mul, b.clone(), d.clone()),
        );
        let imag = binary(
            BinaryOp::Add,
            binary(BinaryOp::Mul, a, d),
            binary(BinaryOp::Mul, b, c),
        );
        self.push_complex_temp_result_store(pointer, scalar_type, real, imag)
    }

    fn push_complex_div_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        left: &LoweredExpr,
        right: &LoweredExpr,
    ) -> CompileResult<()> {
        let element_byte_size = complex_lane_byte_size(scalar_type);
        let a = complex_lane_expr(left, 0, element_byte_size);
        let b = complex_lane_expr(left, 1, element_byte_size);
        let c = complex_lane_expr(right, 0, element_byte_size);
        let d = complex_lane_expr(right, 1, element_byte_size);
        let denominator = binary(
            BinaryOp::Add,
            binary(BinaryOp::Mul, c.clone(), c.clone()),
            binary(BinaryOp::Mul, d.clone(), d.clone()),
        );
        let real = binary(
            BinaryOp::Div,
            binary(
                BinaryOp::Add,
                binary(BinaryOp::Mul, a.clone(), c.clone()),
                binary(BinaryOp::Mul, b.clone(), d.clone()),
            ),
            denominator.clone(),
        );
        let imag = binary(
            BinaryOp::Div,
            binary(
                BinaryOp::Sub,
                binary(BinaryOp::Mul, b, c),
                binary(BinaryOp::Mul, a, d),
            ),
            denominator,
        );
        self.push_complex_temp_result_store(pointer, scalar_type, real, imag)
    }

    fn push_complex_temp_result_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        real: LoweredExpr,
        imag: LoweredExpr,
    ) -> CompileResult<()> {
        let byte_size = scalar_size(scalar_type);
        let slot = self.declare_anonymous_slot(scalar_type, byte_size, byte_size)?;
        let temp_pointer = LoweredExpr::LocalAddress {
            offset: self.local_offset(slot)?,
            byte_size,
        };
        let element_byte_size = complex_lane_byte_size(scalar_type);
        self.push_complex_element_store(&temp_pointer, 0, element_byte_size, real)?;
        self.push_complex_element_store(&temp_pointer, 1, element_byte_size, imag)?;
        self.push_complex_object_copy(pointer, &temp_pointer, scalar_type)
    }
}

fn binary(op: BinaryOp, left: LoweredExpr, right: LoweredExpr) -> LoweredExpr {
    LoweredExpr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}
