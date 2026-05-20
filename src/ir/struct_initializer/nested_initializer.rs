use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{FieldType, GlobalStructInitializerValue, StructLayout};

use super::{
    GlobalBinding, LoweredStructInitializerScalar, lower_int, lower_long_long, lower_pointer,
};

pub(super) fn lower(
    value: &GlobalStructInitializerValue,
    struct_name: &str,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<LoweredStructInitializerScalar> {
    let layout = structs
        .get(struct_name)
        .ok_or_else(|| CompileError::new(format!("unknown nested struct field: {struct_name}")))?;
    if layout.fields.iter().any(|field| field.offset != 0) {
        return Err(CompileError::new(
            "unsupported nested global struct initializer field",
        ));
    }
    let Some(field) = layout.fields.first() else {
        return Err(CompileError::new(
            "unsupported empty nested global struct initializer field",
        ));
    };
    match &field.field_type {
        FieldType::Scalar(field) if field.scalar_type == crate::parser::ScalarType::Int => {
            lower_int(value, field.byte_size)
        }
        FieldType::Scalar(field) if field.scalar_type == crate::parser::ScalarType::LongLong => {
            lower_long_long(value)
        }
        FieldType::Scalar(field) if field.scalar_type == crate::parser::ScalarType::Pointer => {
            lower_pointer(value, global_bindings)
        }
        FieldType::Pointer { .. } => lower_pointer(value, global_bindings),
        FieldType::Scalar(_)
        | FieldType::Struct(_)
        | FieldType::Array { .. }
        | FieldType::StructArray { .. } => Err(CompileError::new(
            "unsupported nested global struct initializer field",
        )),
    }
}
