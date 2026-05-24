use super::{GlobalBinding, LoweredGlobalInitializer, scalar_size};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Global, GlobalInitializer, ScalarType};

pub(in crate::ir) fn lower_array_global_initializer(
    global: &Global,
) -> CompileResult<Option<(LoweredGlobalInitializer, GlobalBinding)>> {
    if let Some(lowered) = lower_unsigned_char_global_initializer(&global.initializer) {
        return Ok(Some(lowered));
    }
    if let Some(lowered) = lower_real_array_global_initializer(&global.initializer)? {
        return Ok(Some(lowered));
    }
    lower_integer_array_global_initializer(global)
}

fn lower_integer_array_global_initializer(
    global: &Global,
) -> CompileResult<Option<(LoweredGlobalInitializer, GlobalBinding)>> {
    match &global.initializer {
        GlobalInitializer::LongLongArray(values) => Ok(Some((
            LoweredGlobalInitializer::LongLongArray(values.clone()),
            GlobalBinding::ScalarArray {
                scalar_type: ScalarType::LongLong,
                length: Some(values.len()),
            },
        ))),
        GlobalInitializer::IntArray(values) => Ok(Some((
            LoweredGlobalInitializer::IntArray(values.clone()),
            GlobalBinding::IntArray,
        ))),
        GlobalInitializer::BoolArray(values) => Ok(Some((
            LoweredGlobalInitializer::UnsignedCharArray(values.clone()),
            GlobalBinding::ScalarArray {
                scalar_type: ScalarType::Bool,
                length: Some(values.len()),
            },
        ))),
        GlobalInitializer::ShortArray { .. } => {
            lower_short_array_global_initializer(global).map(Some)
        }
        GlobalInitializer::IntMatrix { values, columns } => Ok(Some((
            LoweredGlobalInitializer::IntArray(values.clone()),
            GlobalBinding::IntMatrix { columns: *columns },
        ))),
        _ => Ok(None),
    }
}

fn lower_real_array_global_initializer(
    initializer: &GlobalInitializer,
) -> CompileResult<Option<(LoweredGlobalInitializer, GlobalBinding)>> {
    match initializer {
        GlobalInitializer::DoubleArray { length } => {
            let byte_len = length
                .checked_mul(scalar_size(ScalarType::Double))
                .ok_or_else(|| CompileError::new("global double-array size overflow"))?;
            Ok(Some((
                LoweredGlobalInitializer::ZeroBytes(byte_len),
                GlobalBinding::DoubleArray,
            )))
        }
        GlobalInitializer::ScalarArray {
            scalar_type,
            length,
        } => {
            let byte_len = length
                .checked_mul(scalar_size(*scalar_type))
                .ok_or_else(|| CompileError::new("global scalar-array size overflow"))?;
            Ok(Some((
                LoweredGlobalInitializer::ZeroBytes(byte_len),
                GlobalBinding::ScalarArray {
                    scalar_type: *scalar_type,
                    length: Some(*length),
                },
            )))
        }
        GlobalInitializer::ScalarArrayValues {
            scalar_type,
            length,
            values,
        } => Ok(Some((
            LoweredGlobalInitializer::RealArray {
                scalar_type: *scalar_type,
                length: *length,
                values: values.clone(),
            },
            GlobalBinding::ScalarArray {
                scalar_type: *scalar_type,
                length: Some(*length),
            },
        ))),
        _ => Ok(None),
    }
}

fn lower_unsigned_char_global_initializer(
    initializer: &GlobalInitializer,
) -> Option<(LoweredGlobalInitializer, GlobalBinding)> {
    match initializer {
        GlobalInitializer::UnsignedCharArray {
            values,
            is_unsigned,
        } => Some((
            LoweredGlobalInitializer::UnsignedCharArray(values.clone()),
            GlobalBinding::UnsignedCharArray {
                is_unsigned: *is_unsigned,
            },
        )),
        GlobalInitializer::UnsignedCharMatrix {
            values,
            columns,
            is_unsigned,
        } => Some((
            LoweredGlobalInitializer::UnsignedCharArray(values.clone()),
            GlobalBinding::UnsignedCharMatrix {
                columns: *columns,
                is_unsigned: *is_unsigned,
            },
        )),
        _ => None,
    }
}

fn lower_short_array_global_initializer(
    global: &Global,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    let GlobalInitializer::ShortArray {
        values,
        is_unsigned,
        columns,
    } = &global.initializer
    else {
        return Err(CompileError::new("expected short-array global initializer"));
    };
    Ok((
        LoweredGlobalInitializer::ShortArray(values.clone()),
        GlobalBinding::ShortArray {
            is_unsigned: *is_unsigned,
            columns: *columns,
        },
    ))
}
