use super::{LoweredExpr, LoweredLValue, LoweringContext, zero_expr_for};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, LValue, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn is_array_compound_element_address(initializer: &Expr) -> bool {
        array_compound_element_address(initializer).is_ok()
    }

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
        let (array_pointer, _) = self.lower_array_compound_pointer(initializer)?;
        self.push_store(target, array_pointer)
    }

    pub(in crate::ir) fn lower_array_compound_element_pointer_initializer(
        &mut self,
        pointer_slot: usize,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let target = LoweredLValue::Local {
            slot: pointer_slot,
            offset: self.local_offset(pointer_slot)?,
            scalar_type: ScalarType::Pointer,
        };
        self.lower_array_compound_element_pointer_assignment(target, initializer)
    }

    pub(in crate::ir) fn lower_array_compound_element_pointer_assignment(
        &mut self,
        target: LoweredLValue,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let (array, index) = array_compound_element_address(initializer)?;
        let (array_pointer, element_byte_size) = self.lower_array_compound_pointer(array)?;
        let element_pointer = LoweredExpr::PointerOffset {
            pointer: Box::new(array_pointer),
            index: Box::new(self.lower_expr(index)?),
            byte_size: element_byte_size,
        };
        self.push_store(target, element_pointer)
    }

    fn lower_array_compound_pointer(
        &mut self,
        initializer: &Expr,
    ) -> CompileResult<(LoweredExpr, usize)> {
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
        Ok((array_pointer, *element_byte_size))
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

fn array_compound_element_address(initializer: &Expr) -> CompileResult<(&Expr, &Expr)> {
    let Expr::AddressOf {
        target: LValue::Subscript { array, index },
    } = initializer
    else {
        return Err(CompileError::new(
            "expected address of array compound literal element",
        ));
    };
    if matches!(array.as_ref(), Expr::ArrayCompoundLiteral { .. }) {
        Ok((array, index))
    } else {
        Err(CompileError::new(
            "expected address of array compound literal element",
        ))
    }
}
