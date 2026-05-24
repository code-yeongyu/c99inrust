use super::{
    LoweredExpr, LoweredLValue, LoweringContext, StructAddress, pointer_field_address,
    struct_alignment,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, LValue, LocalStructInitializerValue, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn is_struct_compound_member_address(initializer: &Expr) -> bool {
        struct_compound_member_address(initializer).is_ok()
    }

    pub(in crate::ir) fn is_struct_compound_address(initializer: &Expr) -> bool {
        struct_compound_address(initializer).is_ok()
    }

    pub(in crate::ir) fn lower_struct_compound_member_pointer_initializer(
        &mut self,
        pointer_slot: usize,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let target = self.struct_compound_pointer_slot_lvalue(pointer_slot)?;
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

    pub(in crate::ir) fn lower_struct_compound_pointer_initializer(
        &mut self,
        pointer_slot: usize,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let target = self.struct_compound_pointer_slot_lvalue(pointer_slot)?;
        self.lower_struct_compound_pointer_assignment(target, initializer)
    }

    pub(in crate::ir) fn lower_struct_compound_pointer_assignment(
        &mut self,
        target: LoweredLValue,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let pointer = self.lower_struct_compound_pointer(initializer)?;
        self.push_store(target, pointer)
    }

    fn struct_compound_pointer_slot_lvalue(
        &self,
        pointer_slot: usize,
    ) -> CompileResult<LoweredLValue> {
        Ok(LoweredLValue::Local {
            slot: pointer_slot,
            offset: self.local_offset(pointer_slot)?,
            scalar_type: ScalarType::Pointer,
            referent: None,
        })
    }

    fn lower_struct_compound_member_pointer(
        &mut self,
        initializer: &Expr,
    ) -> CompileResult<LoweredExpr> {
        let (struct_name, values, field) = struct_compound_member_address(initializer)?;
        let field_offset = {
            let layout = self.struct_layout(struct_name)?;
            layout
                .fields
                .iter()
                .find(|candidate| candidate.name == field)
                .ok_or_else(|| CompileError::new(format!("unknown struct field: {field}")))?
                .offset
        };
        let pointer = self.lower_struct_compound_pointer_storage(struct_name, values)?;
        Ok(pointer_field_address(pointer, field_offset))
    }

    fn lower_struct_compound_pointer(&mut self, initializer: &Expr) -> CompileResult<LoweredExpr> {
        let (struct_name, values) = struct_compound_address(initializer)?;
        self.lower_struct_compound_pointer_storage(struct_name, values)
    }

    fn lower_struct_compound_pointer_storage(
        &mut self,
        struct_name: &str,
        values: &[LocalStructInitializerValue],
    ) -> CompileResult<LoweredExpr> {
        let (byte_size, alignment) = {
            let layout = self.struct_layout(struct_name)?;
            (layout.size, struct_alignment(layout))
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
        Ok(pointer)
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

fn struct_compound_address(
    initializer: &Expr,
) -> CompileResult<(&str, &[LocalStructInitializerValue])> {
    let Expr::AddressOf {
        target:
            LValue::StructCompoundLiteral {
                struct_name,
                values,
            },
    } = initializer
    else {
        return Err(CompileError::new(
            "expected address of struct compound literal",
        ));
    };
    Ok((struct_name, values))
}
