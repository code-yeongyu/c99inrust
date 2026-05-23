use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{FieldType, GlobalPointerAddress, ScalarType, StructLayout};

use super::{GlobalBinding, pointer_arithmetic, scalar_size};

pub(in crate::ir) fn resolve(
    referent: Option<&str>,
    address: &GlobalPointerAddress,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<(String, usize)> {
    let byte_offset = if address.fields.is_empty() {
        linear_offset(referent, address.index, structs)?
    } else {
        member_offset(address, structs, global_bindings)?
    };
    Ok((address.base.clone(), byte_offset))
}

pub(in crate::ir) fn pointer_referent_size(
    referent: Option<&str>,
    structs: &HashMap<String, StructLayout>,
) -> usize {
    referent
        .and_then(pointer_arithmetic::byte_size)
        .or_else(|| referent.and_then(|name| structs.get(name).map(|layout| layout.size)))
        .unwrap_or_else(|| scalar_size(ScalarType::Int))
}

fn linear_offset(
    referent: Option<&str>,
    index: usize,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<usize> {
    index
        .checked_mul(pointer_referent_size(referent, structs))
        .ok_or_else(|| CompileError::new("global pointer offset overflow"))
}

fn member_offset(
    address: &GlobalPointerAddress,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<usize> {
    let (mut struct_name, byte_size) = base_struct(&address.base, global_bindings)?;
    let mut byte_offset = address
        .index
        .checked_mul(byte_size)
        .ok_or_else(|| CompileError::new("global member pointer offset overflow"))?;
    for (position, field_name) in address.fields.iter().enumerate() {
        let layout = structs
            .get(&struct_name)
            .ok_or_else(|| CompileError::new(format!("unknown struct type: {struct_name}")))?;
        let field = layout
            .fields
            .iter()
            .find(|field| field.name == *field_name)
            .ok_or_else(|| CompileError::new(format!("unknown struct field: {field_name}")))?;
        byte_offset = byte_offset
            .checked_add(field.offset)
            .ok_or_else(|| CompileError::new("global member pointer offset overflow"))?;
        if position + 1 == address.fields.len() {
            if let Some(element_index) = address.element_index {
                byte_offset = byte_offset
                    .checked_add(array_element_offset(&field.field_type, element_index)?)
                    .ok_or_else(|| CompileError::new("global member pointer offset overflow"))?;
            }
        } else {
            let FieldType::Struct(next_struct) = &field.field_type else {
                return Err(CompileError::new(
                    "global member pointer path crosses non-struct field",
                ));
            };
            struct_name = next_struct.clone();
        }
    }
    Ok(byte_offset)
}

fn array_element_offset(field_type: &FieldType, index: usize) -> CompileResult<usize> {
    let FieldType::Array {
        element_size,
        length,
        ..
    } = field_type
    else {
        return Err(CompileError::new(
            "global member pointer subscript requires an array field",
        ));
    };
    if index >= *length {
        return Err(CompileError::new(
            "global member pointer array index is out of bounds",
        ));
    }
    index
        .checked_mul(*element_size)
        .ok_or_else(|| CompileError::new("global member pointer offset overflow"))
}

fn base_struct(
    base: &str,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<(String, usize)> {
    match global_bindings.get(base) {
        Some(
            GlobalBinding::StructObject {
                struct_name,
                byte_size,
            }
            | GlobalBinding::StructArray {
                struct_name,
                byte_size,
                ..
            },
        ) => Ok((struct_name.clone(), *byte_size)),
        Some(_) => Err(CompileError::new(
            "global member pointer base is not a struct object",
        )),
        None => Err(CompileError::new(format!(
            "unknown global member pointer base: {base}"
        ))),
    }
}
