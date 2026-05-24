use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{GlobalStructInitializerAddress, GlobalStructInitializerValue, ScalarType};

use super::super::{GlobalBinding, LoweredStructInitializerScalar, scalar_size};

pub(super) fn lower(
    value: &GlobalStructInitializerValue,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<LoweredStructInitializerScalar> {
    match value {
        GlobalStructInitializerValue::Integer(0) => Ok(LoweredStructInitializerScalar::PointerNull),
        GlobalStructInitializerValue::Integer(value) => {
            Ok(LoweredStructInitializerScalar::PointerInteger(*value))
        }
        GlobalStructInitializerValue::String(value) => Ok(
            LoweredStructInitializerScalar::PointerString(value.clone(), 0),
        ),
        GlobalStructInitializerValue::StringPointer {
            value,
            byte_offset,
            cast_target,
        } if string_pointer_can_initialize_pointer(*cast_target) => Ok(
            LoweredStructInitializerScalar::PointerString(value.clone(), *byte_offset),
        ),
        GlobalStructInitializerValue::StringPointer { .. } => Err(CompileError::new(
            "unsupported global struct pointer initializer string pointer cast",
        )),
        GlobalStructInitializerValue::Address(address) => lower_address(address, global_bindings),
        GlobalStructInitializerValue::Nested(values) if values.len() == 1 => {
            lower(&values[0], global_bindings)
        }
        GlobalStructInitializerValue::Nested(_) => Err(CompileError::new(
            "unsupported global struct pointer initializer",
        )),
    }
}

const fn string_pointer_can_initialize_pointer(cast_target: Option<ScalarType>) -> bool {
    matches!(cast_target, None | Some(ScalarType::Pointer))
}

fn lower_address(
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

fn global_binding_element_size(binding: &GlobalBinding) -> usize {
    match binding {
        GlobalBinding::Int | GlobalBinding::IntArray => scalar_size(ScalarType::Int),
        GlobalBinding::LongLong => scalar_size(ScalarType::LongLong),
        GlobalBinding::Scalar(scalar_type) | GlobalBinding::ScalarArray { scalar_type, .. } => {
            scalar_size(*scalar_type)
        }
        GlobalBinding::IntMatrix { columns } => columns * scalar_size(ScalarType::Int),
        GlobalBinding::ShortArray { columns, .. } => columns.map_or(2, |columns| columns * 2),
        GlobalBinding::DoubleArray => scalar_size(ScalarType::Double),
        GlobalBinding::Pointer { .. } => scalar_size(ScalarType::Pointer),
        GlobalBinding::PointerArray { columns, .. } => columns
            .map_or(scalar_size(ScalarType::Pointer), |columns| {
                columns * scalar_size(ScalarType::Pointer)
            }),
        GlobalBinding::StructObject { byte_size, .. }
        | GlobalBinding::StructArray { byte_size, .. } => *byte_size,
        GlobalBinding::UnsignedCharArray { .. } => 1,
        GlobalBinding::UnsignedCharMatrix { columns, .. } => *columns,
    }
}
