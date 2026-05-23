use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::GlobalStructInitializerValue;

use super::LoweredStructInitializerScalar;

pub(super) fn lower(
    value: &GlobalStructInitializerValue,
    element_size: usize,
    length: usize,
) -> CompileResult<LoweredStructInitializerScalar> {
    if let Some(bytes) = string_bytes(value, element_size, length)? {
        return Ok(bytes);
    }
    let GlobalStructInitializerValue::Nested(values) = value else {
        return Err(CompileError::new(
            "unsupported global struct array field initializer",
        ));
    };
    lower_numeric_values(values, element_size, length)
}

fn string_bytes(
    value: &GlobalStructInitializerValue,
    element_size: usize,
    length: usize,
) -> CompileResult<Option<LoweredStructInitializerScalar>> {
    let GlobalStructInitializerValue::String(value) = value else {
        return Ok(None);
    };
    let byte_len = byte_len(element_size, length)?;
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
    Ok(Some(LoweredStructInitializerScalar::Bytes {
        values,
        byte_len,
    }))
}

fn lower_numeric_values(
    values: &[GlobalStructInitializerValue],
    element_size: usize,
    length: usize,
) -> CompileResult<LoweredStructInitializerScalar> {
    if element_size > 8 {
        return Err(CompileError::new(
            "unsupported global struct array field element size",
        ));
    }
    let byte_len = byte_len(element_size, length)?;
    let mut bytes = vec![0; byte_len];
    let mut flattened = Vec::new();
    flatten_values(values, &mut flattened)?;
    if flattened.len() > length {
        return Err(CompileError::new(
            "too many global struct array field initializer values",
        ));
    }
    for (index, value) in flattened.iter().enumerate() {
        let start = index
            .checked_mul(element_size)
            .ok_or_else(|| CompileError::new("global struct array field offset overflow"))?;
        let end = start
            .checked_add(element_size)
            .ok_or_else(|| CompileError::new("global struct array field offset overflow"))?;
        bytes[start..end].copy_from_slice(&value.to_le_bytes()[..element_size]);
    }
    Ok(LoweredStructInitializerScalar::Bytes {
        values: bytes,
        byte_len,
    })
}

fn flatten_values(
    values: &[GlobalStructInitializerValue],
    flattened: &mut Vec<i64>,
) -> CompileResult<()> {
    for value in values {
        match value {
            GlobalStructInitializerValue::Integer(value) => flattened.push(*value),
            GlobalStructInitializerValue::Nested(values) => flatten_values(values, flattened)?,
            GlobalStructInitializerValue::String(_)
            | GlobalStructInitializerValue::StringPointer { .. }
            | GlobalStructInitializerValue::Address(_) => {
                return Err(CompileError::new(
                    "unsupported global struct array field initializer",
                ));
            }
        }
    }
    Ok(())
}

fn byte_len(element_size: usize, length: usize) -> CompileResult<usize> {
    length
        .checked_mul(element_size)
        .ok_or_else(|| CompileError::new("global struct array field size overflow"))
}
