use super::LoweredExpr;
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{LocalCharArrayInitializer, ScalarType, StructLayout};

pub(in crate::ir) fn zero_expr_for(scalar_type: ScalarType) -> LoweredExpr {
    match scalar_type {
        ScalarType::Double | ScalarType::LongDouble => {
            LoweredExpr::DoubleLiteral("0.0".to_string())
        }
        ScalarType::ComplexFloat | ScalarType::ComplexDouble | ScalarType::ComplexLongDouble => {
            LoweredExpr::Integer(0)
        }
        ScalarType::Bool
        | ScalarType::Int
        | ScalarType::LongLong
        | ScalarType::Pointer
        | ScalarType::VaList => LoweredExpr::Integer(0),
    }
}

pub(in crate::ir) fn local_char_array_initializer_values(
    initializer: &LocalCharArrayInitializer,
    length: usize,
) -> CompileResult<Vec<u8>> {
    match initializer {
        LocalCharArrayInitializer::StringLiteral(value) => {
            local_char_array_string_initializer_values(value, length)
        }
        LocalCharArrayInitializer::Bytes(values) => {
            local_char_array_braced_initializer_values(values, length)
        }
    }
}

pub(in crate::ir) fn local_char_array_string_initializer_values(
    value: &str,
    length: usize,
) -> CompileResult<Vec<u8>> {
    if value.len() > length {
        return Err(CompileError::new(
            "local char array initializer is too large",
        ));
    }
    let mut values = Vec::with_capacity(length);
    values.extend_from_slice(value.as_bytes());
    if values.len() < length {
        values.push(0);
    }
    values.resize(length, 0);
    Ok(values)
}

pub(in crate::ir) fn local_char_array_braced_initializer_values(
    values: &[u8],
    length: usize,
) -> CompileResult<Vec<u8>> {
    if values.len() > length {
        return Err(CompileError::new(
            "local char array initializer is too large",
        ));
    }
    let mut values = values.to_vec();
    values.resize(length, 0);
    Ok(values)
}

pub(in crate::ir) fn local_char_matrix_initializer_values(
    values: &[String],
    rows: usize,
    columns: usize,
) -> CompileResult<Vec<u8>> {
    if values.len() > rows {
        return Err(CompileError::new(
            "local char matrix initializer has too many rows",
        ));
    }
    let mut bytes = Vec::with_capacity(local_char_matrix_byte_size(rows, columns)?);
    for value in values {
        bytes.extend(local_char_array_string_initializer_values(value, columns)?);
    }
    bytes.resize(local_char_matrix_byte_size(rows, columns)?, 0);
    Ok(bytes)
}

pub(in crate::ir) fn local_char_matrix_byte_size(
    rows: usize,
    columns: usize,
) -> CompileResult<usize> {
    rows.checked_mul(columns)
        .ok_or_else(|| CompileError::new("local char matrix size overflow"))
}

pub(in crate::ir) fn local_int_array_byte_size(length: usize) -> CompileResult<usize> {
    length
        .checked_mul(scalar_size(ScalarType::Int))
        .ok_or_else(|| CompileError::new("local int array size overflow"))
}

pub(in crate::ir) fn local_int_matrix_byte_size(
    rows: usize,
    columns: usize,
) -> CompileResult<usize> {
    rows.checked_mul(columns)
        .ok_or_else(|| CompileError::new("local int matrix size overflow"))
        .and_then(local_int_array_byte_size)
}

pub(in crate::ir) fn local_short_array_byte_size(length: usize) -> CompileResult<usize> {
    length
        .checked_mul(2)
        .ok_or_else(|| CompileError::new("local short array size overflow"))
}

pub(in crate::ir) fn local_pointer_array_byte_size(length: usize) -> CompileResult<usize> {
    length
        .checked_mul(scalar_size(ScalarType::Pointer))
        .ok_or_else(|| CompileError::new("local pointer array size overflow"))
}

pub(in crate::ir) fn local_scalar_array_byte_size(
    scalar_type: ScalarType,
    length: usize,
) -> CompileResult<usize> {
    length
        .checked_mul(scalar_size(scalar_type))
        .ok_or_else(|| CompileError::new("local scalar array size overflow"))
}

pub(in crate::ir) fn struct_alignment(layout: &StructLayout) -> usize {
    layout.size.clamp(1, 8)
}

pub(in crate::ir) const fn scalar_size(scalar_type: ScalarType) -> usize {
    match scalar_type {
        ScalarType::Bool | ScalarType::Int => 4,
        ScalarType::LongLong
        | ScalarType::ComplexFloat
        | ScalarType::Double
        | ScalarType::Pointer => 8,
        ScalarType::ComplexDouble => 16,
        ScalarType::LongDouble => long_double_size(),
        ScalarType::ComplexLongDouble => 2 * long_double_size(),
        ScalarType::VaList => 24,
    }
}

const fn long_double_size() -> usize {
    if cfg!(all(target_arch = "x86_64", not(target_os = "macos"))) {
        16
    } else {
        8
    }
}

pub(in crate::ir) const fn align_to(value: usize, alignment: usize) -> usize {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}
