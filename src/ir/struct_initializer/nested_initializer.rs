use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{FieldType, GlobalStructInitializerValue, ScalarType, StructLayout};

use super::{
    GlobalBinding, LoweredStructInitializerScalar, array_field, lower_int, lower_long_long,
    lower_pointer,
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
    let GlobalStructInitializerValue::Nested(values) = value else {
        return lower_single_field(value, layout, structs, global_bindings);
    };
    if values.len() == 1 && layout.fields.iter().all(|field| field.offset == 0) {
        return lower_single_field(&values[0], layout, structs, global_bindings);
    }
    Ok(LoweredStructInitializerScalar::Bytes {
        values: lower_struct_bytes(values, layout, structs, global_bindings)?,
        byte_len: layout.size,
    })
}

fn lower_single_field(
    value: &GlobalStructInitializerValue,
    layout: &StructLayout,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<LoweredStructInitializerScalar> {
    let Some(field) = layout.fields.first() else {
        return Err(CompileError::new(
            "unsupported empty nested global struct initializer field",
        ));
    };
    if field.offset != 0 {
        return Err(CompileError::new(
            "unsupported nested global struct initializer field",
        ));
    }
    lower_field(value, &field.field_type, structs, global_bindings)
}

fn lower_struct_bytes(
    values: &[GlobalStructInitializerValue],
    layout: &StructLayout,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<Vec<u8>> {
    if values.len() > layout.fields.len() {
        return Err(CompileError::new(
            "too many nested global struct initializer values",
        ));
    }
    let mut bytes = vec![0; layout.size];
    for (index, value) in values.iter().enumerate() {
        let field = &layout.fields[index];
        let scalar = lower_field(value, &field.field_type, structs, global_bindings)?;
        write_scalar(&mut bytes, field.offset, &scalar, &layout.name, &field.name)?;
    }
    Ok(bytes)
}

fn lower_field(
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
        FieldType::Struct(struct_name) => lower(value, struct_name, structs, global_bindings),
        FieldType::Scalar(_) | FieldType::StructArray { .. } => Err(CompileError::new(
            "unsupported nested global struct initializer field",
        )),
    }
}

fn write_scalar(
    bytes: &mut [u8],
    offset: usize,
    scalar: &LoweredStructInitializerScalar,
    struct_name: &str,
    field_name: &str,
) -> CompileResult<()> {
    match scalar {
        LoweredStructInitializerScalar::Int { value, byte_size } => write_bytes(
            bytes,
            offset,
            &i64::from(*value).to_le_bytes()[..*byte_size],
        ),
        LoweredStructInitializerScalar::LongLong(value)
        | LoweredStructInitializerScalar::PointerInteger(value) => {
            write_bytes(bytes, offset, &value.to_le_bytes())
        }
        LoweredStructInitializerScalar::Bytes { values, byte_len } => {
            write_bytes(bytes, offset, &values[..*byte_len])
        }
        LoweredStructInitializerScalar::PointerNull => write_bytes(bytes, offset, &[0; 8]),
        LoweredStructInitializerScalar::IntString { .. }
        | LoweredStructInitializerScalar::PointerString(_, _)
        | LoweredStructInitializerScalar::PointerGlobalOffset { .. } => {
            Err(CompileError::new(format!(
                "unsupported relocatable nested global struct initializer: {struct_name}.{field_name}"
            )))
        }
    }
}

fn write_bytes(bytes: &mut [u8], offset: usize, values: &[u8]) -> CompileResult<()> {
    let end = offset
        .checked_add(values.len())
        .ok_or_else(|| CompileError::new("nested global struct initializer offset overflow"))?;
    let Some(target) = bytes.get_mut(offset..end) else {
        return Err(CompileError::new(
            "nested global struct initializer exceeds field size",
        ));
    };
    target.copy_from_slice(values);
    Ok(())
}
