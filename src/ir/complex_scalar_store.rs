use super::{LoweredExpr, LoweredLValue, LoweringContext, pointer_field_address, scalar_size};
use crate::parser::{BinaryOp, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn push_complex_object_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        value: LoweredExpr,
    ) {
        if let Some(source) = complex_object_pointer(&value, scalar_type) {
            return self.push_complex_object_copy(pointer, &source, scalar_type);
        }
        if let Some((op, left, right)) = complex_binary_operands(&value, scalar_type) {
            return self.push_complex_binary_store(pointer, scalar_type, op, &left, &right);
        }
        self.push_complex_element_store(pointer, 0, complex_lane_byte_size(scalar_type), value);
        self.zero_complex_tail(pointer, scalar_type);
    }

    pub(in crate::ir) fn push_complex_indirect_store(
        &mut self,
        target: &LoweredLValue,
        value: LoweredExpr,
    ) -> bool {
        let Some((pointer, scalar_type)) = complex_indirect_target(target) else {
            return false;
        };
        self.push_complex_object_store(&pointer, scalar_type, value);
        true
    }

    fn push_complex_object_copy(
        &mut self,
        target_pointer: &LoweredExpr,
        source_pointer: &LoweredExpr,
        scalar_type: ScalarType,
    ) {
        let element_byte_size = complex_lane_byte_size(scalar_type);
        let tail_slots = scalar_size(scalar_type) / element_byte_size;
        for (index_value, _) in (0_i64..).zip(0..tail_slots) {
            self.push_complex_element_store(
                target_pointer,
                index_value,
                element_byte_size,
                complex_lane_expr(source_pointer, index_value, element_byte_size),
            );
        }
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

    fn push_complex_binary_store(
        &mut self,
        pointer: &LoweredExpr,
        scalar_type: ScalarType,
        op: BinaryOp,
        left: &LoweredExpr,
        right: &LoweredExpr,
    ) {
        let element_byte_size = complex_lane_byte_size(scalar_type);
        let tail_slots = scalar_size(scalar_type) / element_byte_size;
        for (index_value, _) in (0_i64..).zip(0..tail_slots) {
            self.push_complex_element_store(
                pointer,
                index_value,
                element_byte_size,
                LoweredExpr::Binary {
                    op,
                    left: Box::new(complex_lane_expr(left, index_value, element_byte_size)),
                    right: Box::new(complex_lane_expr(right, index_value, element_byte_size)),
                },
            );
        }
    }
}

fn complex_indirect_target(target: &LoweredLValue) -> Option<(LoweredExpr, ScalarType)> {
    match target {
        LoweredLValue::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
            ..
        } if is_complex_scalar(*element_type) => Some((
            LoweredExpr::PointerOffset {
                pointer: pointer.clone(),
                index: index.clone(),
                byte_size: *element_byte_size,
            },
            *element_type,
        )),
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
            ..
        } if is_complex_scalar(*scalar_type) => Some((
            pointer_field_address((**pointer).clone(), *offset),
            *scalar_type,
        )),
        _ => None,
    }
}

fn complex_object_pointer(value: &LoweredExpr, scalar_type: ScalarType) -> Option<LoweredExpr> {
    match value {
        LoweredExpr::Local {
            offset,
            scalar_type: source_type,
        } if *source_type == scalar_type => Some(LoweredExpr::LocalAddress {
            offset: *offset,
            byte_size: scalar_size(*source_type),
        }),
        LoweredExpr::Global {
            name,
            scalar_type: source_type,
        } if *source_type == scalar_type => Some(LoweredExpr::GlobalAddress { name: name.clone() }),
        LoweredExpr::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
            ..
        } if *element_type == scalar_type => Some(LoweredExpr::PointerOffset {
            pointer: pointer.clone(),
            index: index.clone(),
            byte_size: *element_byte_size,
        }),
        LoweredExpr::PointerField {
            pointer,
            offset,
            scalar_type: source_type,
            ..
        } if *source_type == scalar_type => {
            Some(pointer_field_address((**pointer).clone(), *offset))
        }
        _ => None,
    }
}

fn complex_lane_expr(
    source_pointer: &LoweredExpr,
    index: i64,
    element_byte_size: usize,
) -> LoweredExpr {
    LoweredExpr::PointerSubscript {
        pointer: Box::new(source_pointer.clone()),
        index: Box::new(LoweredExpr::Integer(index)),
        element_type: ScalarType::Double,
        element_byte_size,
        element_unsigned: false,
    }
}

fn complex_binary_operands(
    value: &LoweredExpr,
    scalar_type: ScalarType,
) -> Option<(BinaryOp, LoweredExpr, LoweredExpr)> {
    let LoweredExpr::Binary { op, left, right } = value else {
        return None;
    };
    if !matches!(op, BinaryOp::Add | BinaryOp::Sub) {
        return None;
    }
    Some((
        *op,
        complex_object_pointer(left, scalar_type)?,
        complex_object_pointer(right, scalar_type)?,
    ))
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
