use super::{LoweredExpr, LoweredLValue, LoweringContext, zero_expr_for};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_array_compound_pointer_initializer(
        &mut self,
        pointer_slot: usize,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let target = LoweredLValue::Local {
            slot: pointer_slot,
            offset: self.local_offset(pointer_slot)?,
            scalar_type: ScalarType::Pointer,
        };
        self.lower_array_compound_pointer_assignment(target, initializer)
    }

    pub(in crate::ir) fn lower_array_compound_pointer_assignment(
        &mut self,
        target: LoweredLValue,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let Expr::ArrayCompoundLiteral {
            element_type,
            element_byte_size,
            element_unsigned,
            length,
            values,
        } = initializer
        else {
            return Err(CompileError::new("expected array compound literal"));
        };
        if values.len() > *length {
            return Err(CompileError::new(
                "compound literal array initializer is too large",
            ));
        }
        let byte_size = length
            .checked_mul(*element_byte_size)
            .ok_or_else(|| CompileError::new("compound literal array size overflow"))?;
        let array_slot = self.declare_anonymous_slot(
            *element_type,
            byte_size,
            compound_array_alignment(*element_byte_size),
        )?;
        let array_pointer = LoweredExpr::LocalAddress {
            offset: self.local_offset(array_slot)?,
            byte_size,
        };
        for index in 0..*length {
            self.lower_array_compound_pointer_value(
                &array_pointer,
                *element_type,
                *element_byte_size,
                *element_unsigned,
                index,
                values.get(index),
            )?;
        }
        self.push_store(target, array_pointer)
    }

    fn lower_array_compound_pointer_value(
        &mut self,
        array_pointer: &LoweredExpr,
        element_type: ScalarType,
        element_byte_size: usize,
        element_unsigned: bool,
        index: usize,
        value: Option<&Expr>,
    ) -> CompileResult<()> {
        let index = i64::try_from(index)
            .map_err(|_| CompileError::new("compound literal array index overflow"))?;
        let value = match value {
            Some(value) => self.lower_expr(value)?,
            None => zero_expr_for(element_type),
        };
        self.push_store(
            LoweredLValue::PointerSubscript {
                pointer: Box::new(array_pointer.clone()),
                index: Box::new(LoweredExpr::Integer(index)),
                element_type,
                element_byte_size,
                element_unsigned,
            },
            value,
        )
    }
}

fn compound_array_alignment(element_byte_size: usize) -> usize {
    element_byte_size.clamp(1, 8)
}
