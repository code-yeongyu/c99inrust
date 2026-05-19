use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{
    FieldType, GlobalStructInitializerAddress, GlobalStructInitializerValue, ScalarType,
    StructLayout,
};

use super::{
    GlobalBinding, LoweredGlobalInitializer, LoweredStructInitializerScalar,
    LoweredStructInitializerValue, scalar_size,
};

pub(super) fn lower_struct_array_global(
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
            values: lower_initializer_values(values, layout, global_bindings)?,
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

fn lower_initializer_values(
    rows: &[Vec<GlobalStructInitializerValue>],
    layout: &StructLayout,
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
                value: lower_value(value, &field.field_type, global_bindings)?,
            });
        }
    }
    Ok(values)
}

fn lower_value(
    value: &GlobalStructInitializerValue,
    field_type: &FieldType,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<LoweredStructInitializerScalar> {
    match field_type {
        FieldType::Scalar(ScalarType::Int) => lower_int(value),
        FieldType::Scalar(ScalarType::LongLong) => lower_long_long(value),
        FieldType::Scalar(ScalarType::Pointer) | FieldType::Pointer { .. } => {
            lower_pointer(value, global_bindings)
        }
        FieldType::Array {
            element_size,
            length,
            ..
        } => lower_array_field(value, *element_size, *length),
        FieldType::Scalar(ScalarType::Double | ScalarType::VaList)
        | FieldType::Struct(_)
        | FieldType::StructArray { .. } => Err(CompileError::new(
            "unsupported global struct initializer field",
        )),
    }
}

fn lower_int(
    value: &GlobalStructInitializerValue,
) -> CompileResult<LoweredStructInitializerScalar> {
    match value {
        GlobalStructInitializerValue::Integer(value) => Ok(LoweredStructInitializerScalar::Int(
            i32::try_from(*value)
                .map_err(|_| CompileError::new("global struct int initializer does not fit i32"))?,
        )),
        GlobalStructInitializerValue::String(value) => {
            Ok(LoweredStructInitializerScalar::IntString(value.clone()))
        }
        GlobalStructInitializerValue::Address(_) => Err(CompileError::new(
            "unsupported global struct int initializer address",
        )),
    }
}

fn lower_long_long(
    value: &GlobalStructInitializerValue,
) -> CompileResult<LoweredStructInitializerScalar> {
    match value {
        GlobalStructInitializerValue::Integer(value) => {
            Ok(LoweredStructInitializerScalar::LongLong(*value))
        }
        GlobalStructInitializerValue::String(_) | GlobalStructInitializerValue::Address(_) => Err(
            CompileError::new("unsupported global struct long long initializer"),
        ),
    }
}

fn lower_array_field(
    value: &GlobalStructInitializerValue,
    element_size: usize,
    length: usize,
) -> CompileResult<LoweredStructInitializerScalar> {
    let GlobalStructInitializerValue::String(value) = value else {
        return Err(CompileError::new(
            "unsupported global struct array field initializer",
        ));
    };
    let byte_len = length
        .checked_mul(element_size)
        .ok_or_else(|| CompileError::new("global struct array field size overflow"))?;
    let mut values = value.as_bytes().to_vec();
    if values.len() < byte_len {
        values.push(0);
    }
    if values.len() > byte_len {
        return Err(CompileError::new(
            "global struct string initializer exceeds field size",
        ));
    }
    values.resize(byte_len, 0);
    Ok(LoweredStructInitializerScalar::Bytes { values, byte_len })
}

fn lower_pointer(
    value: &GlobalStructInitializerValue,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<LoweredStructInitializerScalar> {
    match value {
        GlobalStructInitializerValue::Integer(0) => Ok(LoweredStructInitializerScalar::PointerNull),
        GlobalStructInitializerValue::Integer(value) => {
            Ok(LoweredStructInitializerScalar::PointerInteger(*value))
        }
        GlobalStructInitializerValue::String(value) => {
            Ok(LoweredStructInitializerScalar::PointerString(value.clone()))
        }
        GlobalStructInitializerValue::Address(address) => {
            lower_pointer_address(address, global_bindings)
        }
    }
}

fn lower_pointer_address(
    address: &GlobalStructInitializerAddress,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<LoweredStructInitializerScalar> {
    let byte_offset = if let Some(index) = address.index {
        let binding = global_bindings.get(&address.base).ok_or_else(|| {
            CompileError::new(format!(
                "unknown global struct initializer address base: {}",
                address.base
            ))
        })?;
        index
            .checked_mul(global_binding_element_size(binding))
            .ok_or_else(|| CompileError::new("global struct initializer address overflow"))?
    } else {
        0
    };
    Ok(LoweredStructInitializerScalar::PointerGlobalOffset {
        base: address.base.clone(),
        byte_offset,
    })
}

const fn global_binding_element_size(binding: &GlobalBinding) -> usize {
    match binding {
        GlobalBinding::Int | GlobalBinding::IntArray | GlobalBinding::IntMatrix { .. } => {
            scalar_size(ScalarType::Int)
        }
        GlobalBinding::DoubleArray => scalar_size(ScalarType::Double),
        GlobalBinding::Pointer { .. } | GlobalBinding::PointerArray { .. } => {
            scalar_size(ScalarType::Pointer)
        }
        GlobalBinding::StructObject { byte_size, .. }
        | GlobalBinding::StructArray { byte_size, .. } => *byte_size,
        GlobalBinding::UnsignedCharArray | GlobalBinding::UnsignedCharMatrix { .. } => 1,
    }
}
