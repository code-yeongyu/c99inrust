use super::{Instruction, LoweredExpr, LoweredLValue, LoweringContext, StructAddress, scalar_size};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{FieldType, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn push_store(&mut self, target: LoweredLValue, value: LoweredExpr) {
        match target {
            LoweredLValue::Local {
                slot,
                offset,
                scalar_type,
            } => self.instructions.push(Instruction::StoreLocal {
                slot,
                offset,
                scalar_type,
                value: store_value_for_scalar(scalar_type, value),
            }),
            LoweredLValue::Global { name, scalar_type } => {
                self.instructions.push(Instruction::StoreGlobal {
                    name,
                    scalar_type,
                    value: store_value_for_scalar(scalar_type, value),
                });
            }
            target @ (LoweredLValue::GlobalByteSubscript { .. }
            | LoweredLValue::GlobalIntSubscript { .. }
            | LoweredLValue::GlobalPointerSubscript { .. }
            | LoweredLValue::PointerSubscript { .. }
            | LoweredLValue::PointerField { .. }) => {
                let value = store_value_for_lvalue(&target, value);
                self.instructions
                    .push(Instruction::Eval(LoweredExpr::Assign {
                        target,
                        value: Box::new(value),
                    }));
            }
        }
    }

    pub(in crate::ir) fn push_struct_copy(
        &mut self,
        target: &StructAddress,
        source: &StructAddress,
    ) -> CompileResult<()> {
        let layout = self.struct_layout(&target.struct_name)?.clone();
        for field in layout.fields {
            let target_offset = target
                .offset
                .checked_add(field.offset)
                .ok_or_else(|| CompileError::new("struct member offset overflow"))?;
            let source_offset = source
                .offset
                .checked_add(field.offset)
                .ok_or_else(|| CompileError::new("struct member offset overflow"))?;
            match field.field_type {
                FieldType::Scalar(field) => self.push_struct_scalar_copy(
                    target,
                    source,
                    target_offset,
                    source_offset,
                    field.scalar_type,
                    field.byte_size,
                ),
                FieldType::Pointer { .. } => self.push_struct_scalar_copy(
                    target,
                    source,
                    target_offset,
                    source_offset,
                    ScalarType::Pointer,
                    scalar_size(ScalarType::Pointer),
                ),
                FieldType::Array {
                    element_type,
                    element_size,
                    length,
                    ..
                } => {
                    for index in 0..length {
                        let element_offset = index
                            .checked_mul(element_size)
                            .and_then(|offset| target_offset.checked_add(offset))
                            .ok_or_else(|| CompileError::new("struct array offset overflow"))?;
                        let source_element_offset = index
                            .checked_mul(element_size)
                            .and_then(|offset| source_offset.checked_add(offset))
                            .ok_or_else(|| CompileError::new("struct array offset overflow"))?;
                        self.push_struct_scalar_copy(
                            target,
                            source,
                            element_offset,
                            source_element_offset,
                            element_type,
                            element_size,
                        );
                    }
                }
                FieldType::Struct(struct_name) => {
                    self.push_struct_copy(
                        &StructAddress {
                            pointer: target.pointer.clone(),
                            offset: target_offset,
                            struct_name: struct_name.clone(),
                        },
                        &StructAddress {
                            pointer: source.pointer.clone(),
                            offset: source_offset,
                            struct_name,
                        },
                    )?;
                }
                FieldType::StructArray {
                    struct_name,
                    length,
                } => {
                    let element_size = self.struct_layout(&struct_name)?.size;
                    for index in 0..length {
                        let element_offset = index
                            .checked_mul(element_size)
                            .and_then(|offset| target_offset.checked_add(offset))
                            .ok_or_else(|| CompileError::new("struct array offset overflow"))?;
                        let source_element_offset = index
                            .checked_mul(element_size)
                            .and_then(|offset| source_offset.checked_add(offset))
                            .ok_or_else(|| CompileError::new("struct array offset overflow"))?;
                        self.push_struct_copy(
                            &StructAddress {
                                pointer: target.pointer.clone(),
                                offset: element_offset,
                                struct_name: struct_name.clone(),
                            },
                            &StructAddress {
                                pointer: source.pointer.clone(),
                                offset: source_element_offset,
                                struct_name: struct_name.clone(),
                            },
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    pub(in crate::ir) fn push_struct_scalar_copy(
        &mut self,
        target: &StructAddress,
        source: &StructAddress,
        target_offset: usize,
        source_offset: usize,
        scalar_type: ScalarType,
        byte_size: usize,
    ) {
        self.push_store(
            LoweredLValue::PointerField {
                pointer: Box::new(target.pointer.clone()),
                offset: target_offset,
                scalar_type,
                byte_size,
                is_unsigned: false,
            },
            LoweredExpr::PointerField {
                pointer: Box::new(source.pointer.clone()),
                offset: source_offset,
                scalar_type,
                byte_size,
                is_unsigned: false,
            },
        );
    }
}

fn store_value_for_lvalue(target: &LoweredLValue, value: LoweredExpr) -> LoweredExpr {
    match target {
        LoweredLValue::PointerSubscript { element_type, .. }
        | LoweredLValue::PointerField {
            scalar_type: element_type,
            ..
        } => store_value_for_scalar(*element_type, value),
        LoweredLValue::Local { .. }
        | LoweredLValue::Global { .. }
        | LoweredLValue::GlobalByteSubscript { .. }
        | LoweredLValue::GlobalIntSubscript { .. }
        | LoweredLValue::GlobalPointerSubscript { .. } => value,
    }
}

fn store_value_for_scalar(scalar_type: ScalarType, value: LoweredExpr) -> LoweredExpr {
    if scalar_type != ScalarType::Bool {
        return value;
    }
    LoweredExpr::Binary {
        op: crate::parser::BinaryOp::NotEqual,
        left: Box::new(value),
        right: Box::new(LoweredExpr::Integer(0)),
    }
}
