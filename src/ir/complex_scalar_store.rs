use super::{LoweredExpr, LoweredLValue, LoweringContext, scalar_size};
use crate::parser::ScalarType;

impl LoweringContext {
    pub(in crate::ir) fn push_complex_object_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        value: LoweredExpr,
    ) {
        self.push_complex_element_store(pointer, 0, value);
        self.zero_complex_tail(pointer, scalar_type);
    }

    fn zero_complex_tail(&mut self, pointer: &LoweredExpr, scalar_type: ScalarType) {
        let tail_slots = scalar_size(scalar_type) / scalar_size(ScalarType::Double);
        for (index_value, _) in (1_i64..).zip(1..tail_slots) {
            self.push_complex_element_store(
                pointer,
                index_value,
                LoweredExpr::DoubleLiteral("0.0".to_owned()),
            );
        }
    }

    fn push_complex_element_store(
        &mut self,
        pointer: &LoweredExpr,
        index: i64,
        value: LoweredExpr,
    ) {
        self.push_store(
            LoweredLValue::PointerSubscript {
                pointer: Box::new(pointer.clone()),
                index: Box::new(LoweredExpr::Integer(index)),
                element_type: ScalarType::Double,
                element_byte_size: scalar_size(ScalarType::Double),
                element_unsigned: false,
            },
            value,
        );
    }
}

pub(in crate::ir) const fn is_complex_scalar(scalar_type: ScalarType) -> bool {
    matches!(
        scalar_type,
        ScalarType::ComplexFloat | ScalarType::ComplexDouble | ScalarType::ComplexLongDouble
    )
}
