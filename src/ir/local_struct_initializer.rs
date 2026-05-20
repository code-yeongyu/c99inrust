use super::{
    Instruction, LocalStructArrayField, LoweredExpr, LoweredLValue, LoweringContext, StructAddress,
    scalar_size,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{
    Expr, FieldType, LocalStructInitializer, LocalStructInitializerValue, ScalarType,
};

impl LoweringContext {
    pub(in crate::ir) fn lower_local_struct_initializer(
        &mut self,
        name: &str,
        struct_name: &str,
        slot: usize,
        initializer: &LocalStructInitializer,
    ) -> CompileResult<()> {
        let layout = self.struct_layout(struct_name)?.clone();
        let target = StructAddress {
            pointer: LoweredExpr::LocalAddress {
                offset: self.local_offset(slot)?,
                byte_size: layout.size,
            },
            offset: 0,
            struct_name: struct_name.to_owned(),
        };
        match initializer {
            LocalStructInitializer::Copy(expr) => self.copy_local_struct_initializer(&target, expr),
            LocalStructInitializer::Values(values) => {
                self.instructions.push(Instruction::InitLocalBytes {
                    offset: self.local_offset(slot)?,
                    values: vec![0; layout.size],
                });
                let mut value_index = 0usize;
                self.push_local_struct_initializer_values(&target, values, &mut value_index)?;
                if value_index == values.len() {
                    Ok(())
                } else {
                    Err(CompileError::new(format!(
                        "too many local struct initializer values for {name}"
                    )))
                }
            }
        }
    }

    fn copy_local_struct_initializer(
        &mut self,
        target: &StructAddress,
        expr: &Expr,
    ) -> CompileResult<()> {
        let source = self.resolve_struct_address(expr)?;
        if source.struct_name != target.struct_name {
            return Err(CompileError::new("incompatible local struct initializer"));
        }
        self.push_struct_copy(target, &source)
    }

    fn push_local_struct_initializer_values(
        &mut self,
        target: &StructAddress,
        values: &[LocalStructInitializerValue],
        value_index: &mut usize,
    ) -> CompileResult<()> {
        let layout = self.struct_layout(&target.struct_name)?.clone();
        for field in layout.fields {
            if *value_index >= values.len() {
                return Ok(());
            }
            let offset = target
                .offset
                .checked_add(field.offset)
                .ok_or_else(|| CompileError::new("local struct initializer offset overflow"))?;
            match field.field_type {
                FieldType::Scalar(field) => {
                    let expr = expr_initializer_value(&values[*value_index])?;
                    *value_index += 1;
                    self.push_store(
                        LoweredLValue::PointerField {
                            pointer: Box::new(target.pointer.clone()),
                            offset,
                            scalar_type: field.scalar_type,
                            byte_size: field.byte_size,
                            is_unsigned: field.is_unsigned,
                        },
                        self.lower_expr(expr)?,
                    );
                }
                FieldType::Pointer { .. } => {
                    let expr = expr_initializer_value(&values[*value_index])?;
                    *value_index += 1;
                    self.push_store(
                        LoweredLValue::PointerField {
                            pointer: Box::new(target.pointer.clone()),
                            offset,
                            scalar_type: ScalarType::Pointer,
                            byte_size: scalar_size(ScalarType::Pointer),
                            is_unsigned: false,
                        },
                        self.lower_expr(expr)?,
                    );
                }
                FieldType::Struct(struct_name) => {
                    let nested_target = StructAddress {
                        pointer: target.pointer.clone(),
                        offset,
                        struct_name,
                    };
                    self.push_nested_local_struct_initializer(&nested_target, values, value_index)?;
                }
                FieldType::Array {
                    element_type,
                    element_size,
                    element_unsigned,
                    length,
                    ..
                } => self.push_local_struct_array_field_initializer(
                    LocalStructArrayField {
                        target,
                        offset,
                        element_type,
                        element_size,
                        element_unsigned,
                        length,
                    },
                    values,
                    value_index,
                )?,
                FieldType::StructArray { .. } => {
                    return Err(CompileError::new(
                        "unsupported local struct initializer field",
                    ));
                }
            }
        }
        Ok(())
    }

    fn push_nested_local_struct_initializer(
        &mut self,
        target: &StructAddress,
        values: &[LocalStructInitializerValue],
        value_index: &mut usize,
    ) -> CompileResult<()> {
        let LocalStructInitializerValue::Nested(nested_values) = &values[*value_index] else {
            return self.push_local_struct_initializer_values(target, values, value_index);
        };
        *value_index += 1;
        let mut nested_index = 0usize;
        self.push_local_struct_initializer_values(target, nested_values, &mut nested_index)?;
        if nested_index == nested_values.len() {
            Ok(())
        } else {
            Err(CompileError::new(
                "too many nested local struct initializer values",
            ))
        }
    }
}

fn expr_initializer_value(value: &LocalStructInitializerValue) -> CompileResult<&Expr> {
    match value {
        LocalStructInitializerValue::Expr(expr) => Ok(expr),
        LocalStructInitializerValue::Nested(values) if values.len() == 1 => {
            expr_initializer_value(&values[0])
        }
        LocalStructInitializerValue::Nested(_) => Err(CompileError::new(
            "unsupported local scalar initializer braces",
        )),
    }
}
