use crate::diagnostics::{CompileError, CompileResult};

use super::scalar_layout::scalar_size_for_layout;
use super::{Global, GlobalInitializer, ScalarType, StructLayout};

pub(super) fn global_sizeof_symbols(
    globals: &[Global],
    known_structs: &[StructLayout],
) -> CompileResult<Vec<(String, usize)>> {
    let mut symbols = known_structs
        .iter()
        .map(|layout| (layout.name.clone(), layout.size))
        .collect::<Vec<_>>();
    for global in globals {
        if let Some(size) = global_initializer_sizeof_bytes(&global.initializer, known_structs)? {
            symbols.push((global.name.clone(), size));
        }
    }
    Ok(symbols)
}

fn global_initializer_sizeof_bytes(
    initializer: &GlobalInitializer,
    known_structs: &[StructLayout],
) -> CompileResult<Option<usize>> {
    let size = match initializer {
        GlobalInitializer::Int(_) | GlobalInitializer::IntConstant(_) => {
            scalar_size_for_layout(ScalarType::Int)
        }
        GlobalInitializer::LongLong(_) => scalar_size_for_layout(ScalarType::LongLong),
        GlobalInitializer::Double(_) => scalar_size_for_layout(ScalarType::Double),
        GlobalInitializer::ComplexReal { scalar_type, .. }
        | GlobalInitializer::ScalarZero(scalar_type) => scalar_size_for_layout(*scalar_type),
        GlobalInitializer::PointerNull { .. }
        | GlobalInitializer::PointerString { .. }
        | GlobalInitializer::PointerName { .. }
        | GlobalInitializer::PointerSubscriptAddress { .. } => {
            scalar_size_for_layout(ScalarType::Pointer)
        }
        GlobalInitializer::IntArray(values) => values
            .len()
            .checked_mul(scalar_size_for_layout(ScalarType::Int))
            .ok_or_else(|| CompileError::new("global int array sizeof overflow"))?,
        GlobalInitializer::ShortArray { values, .. } => values
            .len()
            .checked_mul(2)
            .ok_or_else(|| CompileError::new("global short array sizeof overflow"))?,
        GlobalInitializer::IntMatrix { values, .. } => values
            .len()
            .checked_mul(scalar_size_for_layout(ScalarType::Int))
            .ok_or_else(|| CompileError::new("global int matrix sizeof overflow"))?,
        GlobalInitializer::DoubleArray { length } => length
            .checked_mul(scalar_size_for_layout(ScalarType::Double))
            .ok_or_else(|| CompileError::new("global double array sizeof overflow"))?,
        GlobalInitializer::PointerArray { length, .. } => length
            .checked_mul(scalar_size_for_layout(ScalarType::Pointer))
            .ok_or_else(|| CompileError::new("global pointer array sizeof overflow"))?,
        GlobalInitializer::PointerStringArray { length, .. } => length
            .checked_mul(scalar_size_for_layout(ScalarType::Pointer))
            .ok_or_else(|| CompileError::new("global pointer string array sizeof overflow"))?,
        GlobalInitializer::PointerNameArray { length, .. } => length
            .checked_mul(scalar_size_for_layout(ScalarType::Pointer))
            .ok_or_else(|| CompileError::new("global pointer name array sizeof overflow"))?,
        GlobalInitializer::StructObject { struct_name, .. } => {
            struct_size_for_initializer(struct_name, known_structs)?
        }
        GlobalInitializer::StructArray {
            struct_name,
            length,
            ..
        } => length
            .checked_mul(struct_size_for_initializer(struct_name, known_structs)?)
            .ok_or_else(|| CompileError::new("global struct array sizeof overflow"))?,
        GlobalInitializer::UnsignedCharArray { values, .. }
        | GlobalInitializer::UnsignedCharMatrix { values, .. } => values.len(),
        GlobalInitializer::Extern(_)
        | GlobalInitializer::ExternPointer { .. }
        | GlobalInitializer::ExternIntArray
        | GlobalInitializer::ExternShortArray { .. }
        | GlobalInitializer::ExternPointerArray { .. }
        | GlobalInitializer::ExternUnsignedCharArray { .. }
        | GlobalInitializer::ExternUnsignedCharMatrix { .. }
        | GlobalInitializer::ExternStructArray { .. }
        | GlobalInitializer::ExternStructObject { .. } => return Ok(None),
    };
    Ok(Some(size))
}

fn struct_size_for_initializer(
    struct_name: &str,
    known_structs: &[StructLayout],
) -> CompileResult<usize> {
    known_structs
        .iter()
        .find(|layout| layout.name == struct_name)
        .map(|layout| layout.size)
        .ok_or_else(|| CompileError::new(format!("unknown struct sizeof type: {struct_name}")))
}
