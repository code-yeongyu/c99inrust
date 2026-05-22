use super::{
    LoweredExpr, LoweredLValue, LoweringContext, StructAddress, pointer_field_address,
    struct_alignment, zero_expr_for,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, LValue, LocalStructInitializerValue, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn is_array_compound_element_address(initializer: &Expr) -> bool {
        array_compound_element_address(initializer).is_ok()
    }

    pub(in crate::ir) fn is_struct_compound_member_address(initializer: &Expr) -> bool {
        struct_compound_member_address(initializer).is_ok()
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
            referent: None,
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
            referent: None,
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

    pub(in crate::ir) fn lower_struct_compound_member_pointer_initializer(
        &mut self,
        pointer_slot: usize,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let target = LoweredLValue::Local {
            slot: pointer_slot,
            offset: self.local_offset(pointer_slot)?,
            scalar_type: ScalarType::Pointer,
            referent: None,
        };
        self.lower_struct_compound_member_pointer_assignment(target, initializer)
    }

    pub(in crate::ir) fn lower_struct_compound_member_pointer_assignment(
        &mut self,
        target: LoweredLValue,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let pointer = self.lower_struct_compound_member_pointer(initializer)?;
        self.push_store(target, pointer)
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

    fn lower_struct_compound_member_pointer(
        &mut self,
        initializer: &Expr,
    ) -> CompileResult<LoweredExpr> {
        let (struct_name, values, field) = struct_compound_member_address(initializer)?;
        let (byte_size, alignment, field_offset) = {
            let layout = self.struct_layout(struct_name)?;
            let field = layout
                .fields
                .iter()
                .find(|candidate| candidate.name == field)
                .ok_or_else(|| CompileError::new(format!("unknown struct field: {field}")))?;
            (layout.size, struct_alignment(layout), field.offset)
        };
        let slot = self.declare_anonymous_slot(ScalarType::Pointer, byte_size, alignment)?;
        let pointer = LoweredExpr::LocalAddress {
            offset: self.local_offset(slot)?,
            byte_size,
        };
        let target = StructAddress {
            pointer: pointer.clone(),
            offset: 0,
            struct_name: struct_name.to_owned(),
        };
        self.copy_struct_compound_literal_initializer(&target, struct_name, values)?;
        Ok(pointer_field_address(pointer, field_offset))
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

fn struct_compound_member_address(
    initializer: &Expr,
) -> CompileResult<(&str, &[LocalStructInitializerValue], &str)> {
    let Expr::AddressOf {
        target:
            LValue::Member {
                base,
                field,
                dereference: false,
            },
    } = initializer
    else {
        return Err(CompileError::new(
            "expected address of struct compound literal member",
        ));
    };
    let Expr::StructCompoundLiteral {
        struct_name,
        values,
    } = base.as_ref()
    else {
        return Err(CompileError::new(
            "expected address of struct compound literal member",
        ));
    };
    Ok((struct_name, values, field))
}
