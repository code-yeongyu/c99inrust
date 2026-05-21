use super::{LoweredExpr, LoweredLValue, LoweringContext, scalar_size};
use crate::parser::ScalarType;

impl LoweringContext {
    pub(in crate::ir) fn push_complex_object_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        value: LoweredExpr,
    ) {
        self.push_complex_element_store(pointer, 0, complex_lane_byte_size(scalar_type), value);
        self.zero_complex_tail(pointer, scalar_type);
    }

    fn zero_complex_tail(&mut self, pointer: &LoweredExpr, scalar_type: ScalarType) {
        let element_byte_size = complex_lane_byte_size(scalar_type);
        let tail_slots = scalar_size(scalar_type) / element_byte_size;
        for (index_value, _) in (1_i64..).zip(1..tail_slots) {
            self.push_complex_element_store(
                pointer,
                index_value,
                element_byte_size,
                LoweredExpr::DoubleLiteral("0.0".to_owned()),
            );
        }
    }

    fn push_complex_element_store(
        &mut self,
        pointer: &LoweredExpr,
        index: i64,
        element_byte_size: usize,
        value: LoweredExpr,
    ) {
        self.push_store(
            LoweredLValue::PointerSubscript {
                pointer: Box::new(pointer.clone()),
                index: Box::new(LoweredExpr::Integer(index)),
                element_type: ScalarType::Double,
                element_byte_size,
                element_unsigned: false,
            },
            value,
        );
    }
}

const fn complex_lane_byte_size(scalar_type: ScalarType) -> usize {
    match scalar_type {
        ScalarType::ComplexFloat => 4,
        _ => scalar_size(ScalarType::Double),
    }
}

pub(in crate::ir) const fn is_complex_scalar(scalar_type: ScalarType) -> bool {
    matches!(
        scalar_type,
        ScalarType::ComplexFloat | ScalarType::ComplexDouble | ScalarType::ComplexLongDouble
    )
}
