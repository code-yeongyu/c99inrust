use super::{
    Instruction, LoweredExpr, LoweredLValue, LoweringContext, StructAddress, scalar_size,
    zero_expr_for,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{FieldType, LocalStructInitializerValue, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn copy_struct_compound_literal_initializer(
        &mut self,
        target: &StructAddress,
        source_struct_name: &str,
        values: &[LocalStructInitializerValue],
    ) -> CompileResult<()> {
        if source_struct_name != target.struct_name {
            return Err(CompileError::new("incompatible local struct initializer"));
        }
        self.zero_struct_compound_target(target)?;
        let mut value_index = 0usize;
        self.push_local_struct_initializer_values(target, values, &mut value_index)?;
        if value_index == values.len() {
            Ok(())
        } else {
            Err(CompileError::new(
                "too many struct compound literal initializer values",
            ))
        }
    }

    fn zero_struct_compound_target(&mut self, target: &StructAddress) -> CompileResult<()> {
        let byte_size = self.struct_layout(&target.struct_name)?.size;
        if let LoweredExpr::LocalAddress { offset, .. } = &target.pointer {
            let offset = offset
                .checked_add(target.offset)
                .ok_or_else(|| CompileError::new("compound literal struct initializer overflow"))?;
            self.instructions.push(Instruction::InitLocalBytes {
                offset,
                values: vec![0; byte_size],
            });
            return Ok(());
        }
        self.push_zero_struct_fields(target)
    }

    fn push_zero_struct_fields(&mut self, target: &StructAddress) -> CompileResult<()> {
        let layout = self.struct_layout(&target.struct_name)?.clone();
        for field in layout.fields {
            let offset = target
                .offset
                .checked_add(field.offset)
                .ok_or_else(|| CompileError::new("struct zero offset overflow"))?;
            match field.field_type {
                FieldType::Scalar(field) => self.push_zero_scalar_field(
                    target,
                    offset,
                    field.scalar_type,
                    field.byte_size,
                    field.is_unsigned,
                )?,
                FieldType::Pointer { .. } => self.push_zero_scalar_field(
                    target,
                    offset,
                    ScalarType::Pointer,
                    scalar_size(ScalarType::Pointer),
                    false,
                )?,
                FieldType::Array {
                    element_type,
                    element_size,
                    element_unsigned,
                    length,
                    ..
                } => self.push_zero_array_field(
                    target,
                    offset,
                    element_type,
                    element_size,
                    element_unsigned,
                    length,
                )?,
                FieldType::Struct(struct_name) => {
                    let nested_target = StructAddress {
                        pointer: target.pointer.clone(),
                        offset,
                        struct_name,
                    };
                    self.push_zero_struct_fields(&nested_target)?;
                }
                FieldType::StructArray {
                    struct_name,
                    length,
                } => self.push_zero_struct_array_field(target, offset, &struct_name, length)?,
            }
        }
        Ok(())
    }

    fn push_zero_array_field(
        &mut self,
        target: &StructAddress,
        offset: usize,
        element_type: ScalarType,
        element_size: usize,
        element_unsigned: bool,
        length: usize,
    ) -> CompileResult<()> {
        for index in 0..length {
            let element_offset = index
                .checked_mul(element_size)
                .and_then(|index_offset| offset.checked_add(index_offset))
                .ok_or_else(|| CompileError::new("struct array zero offset overflow"))?;
            self.push_zero_scalar_field(
                target,
                element_offset,
                element_type,
                element_size,
                element_unsigned,
            )?;
        }
        Ok(())
    }

    fn push_zero_struct_array_field(
        &mut self,
        target: &StructAddress,
        offset: usize,
        struct_name: &str,
        length: usize,
    ) -> CompileResult<()> {
        let element_size = self.struct_layout(struct_name)?.size;
        for index in 0..length {
            let element_offset = index
                .checked_mul(element_size)
                .and_then(|index_offset| offset.checked_add(index_offset))
                .ok_or_else(|| CompileError::new("struct array zero offset overflow"))?;
            let element_target = StructAddress {
                pointer: target.pointer.clone(),
                offset: element_offset,
                struct_name: struct_name.to_owned(),
            };
            self.push_zero_struct_fields(&element_target)?;
        }
        Ok(())
    }

    fn push_zero_scalar_field(
        &mut self,
        target: &StructAddress,
        offset: usize,
        scalar_type: ScalarType,
        byte_size: usize,
        is_unsigned: bool,
    ) -> CompileResult<()> {
        self.push_store(
            LoweredLValue::PointerField {
                pointer: Box::new(target.pointer.clone()),
                offset,
                scalar_type,
                byte_size,
                is_unsigned,
            },
            zero_expr_for(scalar_type),
        )
    }
}
