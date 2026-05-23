use std::collections::HashMap;

mod array_field;
mod nested_initializer;
mod pointer_field;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{FieldType, GlobalStructInitializerValue, ScalarType, StructLayout};

use super::{
    GlobalBinding, LoweredGlobalInitializer, LoweredStructInitializerScalar,
    LoweredStructInitializerValue,
};

pub(in crate::ir) fn lower_struct_array_global(
    struct_name: &str,
    length: usize,
    columns: Option<usize>,
    values: &[Vec<GlobalStructInitializerValue>],
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    let layout = structs
        .get(struct_name)
        .ok_or_else(|| CompileError::new(format!("unknown struct-array type: {struct_name}")))?;
    let byte_len = length
        .checked_mul(layout.size)
        .ok_or_else(|| CompileError::new("global struct-array size overflow"))?;
    let initializer = if values.is_empty() {
        LoweredGlobalInitializer::ZeroBytes(byte_len)
    } else {
        LoweredGlobalInitializer::StructArray {
            byte_len,
            values: lower_initializer_values(values, layout, structs, global_bindings)?,
        }
    };
    Ok((
        initializer,
        GlobalBinding::StructArray {
            struct_name: struct_name.to_owned(),
            byte_size: layout.size,
            length: Some(length),
            columns,
        },
    ))
}

pub(in crate::ir) fn lower_struct_object_global(
    struct_name: &str,
    values: &[GlobalStructInitializerValue],
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    let layout = structs
        .get(struct_name)
        .ok_or_else(|| CompileError::new(format!("unknown struct object type: {struct_name}")))?;
    let initializer = if values.is_empty() {
        LoweredGlobalInitializer::ZeroBytes(layout.size)
    } else {
        let rows = vec![values.to_vec()];
        LoweredGlobalInitializer::StructArray {
            byte_len: layout.size,
            values: lower_initializer_values(&rows, layout, structs, global_bindings)?,
        }
    };
    Ok((
        initializer,
        GlobalBinding::StructObject {
            struct_name: struct_name.to_owned(),
            byte_size: layout.size,
        },
    ))
}

fn lower_initializer_values(
    rows: &[Vec<GlobalStructInitializerValue>],
    layout: &StructLayout,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<Vec<LoweredStructInitializerValue>> {
    let mut values = Vec::new();
    for (row_index, row) in rows.iter().enumerate() {
        if row.len() > layout.fields.len() {
            return Err(CompileError::new(
                "too many global struct initializer values",
            ));
        }
        let row_offset = row_index
            .checked_mul(layout.size)
            .ok_or_else(|| CompileError::new("global struct initializer offset overflow"))?;
        for (field_index, value) in row.iter().enumerate() {
            let field = &layout.fields[field_index];
            values.push(LoweredStructInitializerValue {
                byte_offset: row_offset.checked_add(field.offset).ok_or_else(|| {
                    CompileError::new("global struct initializer offset overflow")
                })?,
                value: lower_value(value, &field.field_type, structs, global_bindings)?,
            });
        }
    }
    Ok(values)
}

fn lower_value(
    value: &GlobalStructInitializerValue,
    field_type: &FieldType,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<LoweredStructInitializerScalar> {
    match field_type {
        FieldType::Scalar(field) if field.scalar_type == ScalarType::Int => {
            lower_int(value, field.byte_size)
        }
        FieldType::Scalar(field) if field.scalar_type == ScalarType::LongLong => {
            lower_long_long(value)
        }
        FieldType::Scalar(field) if field.scalar_type == ScalarType::Pointer => {
            lower_pointer(value, global_bindings)
        }
        FieldType::Pointer { .. } => lower_pointer(value, global_bindings),
        FieldType::Array {
            element_size,
            length,
            ..
        } => array_field::lower(value, *element_size, *length),
        FieldType::Struct(struct_name) => {
            nested_initializer::lower(value, struct_name, structs, global_bindings)
        }
        FieldType::Scalar(_) | FieldType::StructArray { .. } => Err(CompileError::new(
            "unsupported global struct initializer field",
        )),
    }
}

fn lower_int(
    value: &GlobalStructInitializerValue,
    byte_size: usize,
) -> CompileResult<LoweredStructInitializerScalar> {
    match value {
        GlobalStructInitializerValue::Integer(value) => Ok(LoweredStructInitializerScalar::Int {
            value: i32::try_from(*value)
                .map_err(|_| CompileError::new("global struct int initializer does not fit i32"))?,
            byte_size,
        }),
        GlobalStructInitializerValue::String(value) => {
            Ok(LoweredStructInitializerScalar::IntString {
                value: value.clone(),
                byte_size,
                byte_offset: 0,
            })
        }
        GlobalStructInitializerValue::StringPointer {
            value,
            byte_offset,
            cast_target,
        } if string_pointer_can_initialize_int(*cast_target) => {
            Ok(LoweredStructInitializerScalar::IntString {
                value: value.clone(),
                byte_size,
                byte_offset: *byte_offset,
            })
        }
        GlobalStructInitializerValue::StringPointer { .. } => Err(CompileError::new(
            "unsupported global struct int initializer string pointer cast",
        )),
        GlobalStructInitializerValue::Address(_) => Err(CompileError::new(
            "unsupported global struct int initializer address",
        )),
        GlobalStructInitializerValue::Nested(values) if values.len() == 1 => {
            lower_int(&values[0], byte_size)
        }
        GlobalStructInitializerValue::Nested(_) => Err(CompileError::new(
            "unsupported global struct int initializer",
        )),
    }
}

const fn string_pointer_can_initialize_int(cast_target: Option<ScalarType>) -> bool {
    matches!(
        cast_target,
        None | Some(ScalarType::Int | ScalarType::LongLong)
    )
}

fn lower_long_long(
    value: &GlobalStructInitializerValue,
) -> CompileResult<LoweredStructInitializerScalar> {
    match value {
        GlobalStructInitializerValue::Integer(value) => {
            Ok(LoweredStructInitializerScalar::LongLong(*value))
        }
        GlobalStructInitializerValue::Nested(values) if values.len() == 1 => {
            lower_long_long(&values[0])
        }
        GlobalStructInitializerValue::String(_)
        | GlobalStructInitializerValue::StringPointer { .. }
        | GlobalStructInitializerValue::Address(_)
        | GlobalStructInitializerValue::Nested(_) => Err(CompileError::new(
            "unsupported global struct long long initializer",
        )),
    }
}

pub(super) fn lower_pointer(
    value: &GlobalStructInitializerValue,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<LoweredStructInitializerScalar> {
    pointer_field::lower(value, global_bindings)
}
