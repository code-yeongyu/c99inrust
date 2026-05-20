use super::{
    GlobalBinding, LoweredGlobalInitializer, pointer_arithmetic, scalar_size, struct_initializer,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Global, GlobalInitializer, ScalarType, StructLayout};
use std::collections::HashMap;

pub(in crate::ir) fn lower_defined_global_initializer(
    global: &Global,
    constants: &HashMap<String, i64>,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    if let Some(lowered) = lower_pointer_array_initializer(&global.initializer) {
        return lowered;
    }
    if let Some(lowered) = lower_unsigned_char_global_initializer(&global.initializer) {
        return Ok(lowered);
    }
    match &global.initializer {
        GlobalInitializer::Int(value) => Ok((
            LoweredGlobalInitializer::Int(i32::try_from(*value).map_err(|_| {
                CompileError::new(format!(
                    "global int initializer does not fit i32: {}",
                    global.name
                ))
            })?),
            GlobalBinding::Int,
        )),
        GlobalInitializer::LongLong(value) => Ok(lower_long_long_global_initializer(*value)),
        GlobalInitializer::IntArray(values) => Ok((
            LoweredGlobalInitializer::IntArray(values.clone()),
            GlobalBinding::IntArray,
        )),
        GlobalInitializer::ShortArray { .. } => lower_short_array_global_initializer(global),
        GlobalInitializer::IntMatrix { values, columns } => Ok((
            LoweredGlobalInitializer::IntArray(values.clone()),
            GlobalBinding::IntMatrix { columns: *columns },
        )),
        GlobalInitializer::DoubleArray { length } => {
            let byte_len = length
                .checked_mul(scalar_size(ScalarType::Double))
                .ok_or_else(|| CompileError::new("global double-array size overflow"))?;
            Ok((
                LoweredGlobalInitializer::ZeroBytes(byte_len),
                GlobalBinding::DoubleArray,
            ))
        }
        GlobalInitializer::IntConstant(name) => {
            lower_int_constant_global(name, &global.name, constants)
        }
        GlobalInitializer::PointerNull { referent } => Ok((
            LoweredGlobalInitializer::PointerNull,
            GlobalBinding::Pointer {
                referent: referent.clone(),
            },
        )),
        GlobalInitializer::PointerString { referent, value } => Ok((
            LoweredGlobalInitializer::PointerString(value.clone()),
            GlobalBinding::Pointer {
                referent: referent.clone(),
            },
        )),
        GlobalInitializer::PointerSubscriptAddress {
            referent,
            base,
            index,
        } => lower_global_pointer_subscript_address(referent.as_deref(), base, *index),
        GlobalInitializer::PointerArray { .. }
        | GlobalInitializer::PointerStringArray { .. }
        | GlobalInitializer::PointerNameArray { .. } => Err(CompileError::new(
            "internal error: pointer array global reached fallback lowering",
        )),
        GlobalInitializer::StructObject {
            struct_name,
            values,
        } => struct_initializer::lower_struct_object_global(
            struct_name,
            values,
            structs,
            global_bindings,
        ),
        GlobalInitializer::StructArray {
            struct_name,
            length,
            columns,
            values,
        } => struct_initializer::lower_struct_array_global(
            struct_name,
            *length,
            *columns,
            values,
            structs,
            global_bindings,
        ),
        GlobalInitializer::UnsignedCharArray { .. }
        | GlobalInitializer::UnsignedCharMatrix { .. } => Err(CompileError::new(
            "internal error: byte global reached fallback lowering",
        )),
        GlobalInitializer::Extern(_)
        | GlobalInitializer::ExternPointer { .. }
        | GlobalInitializer::ExternIntArray
        | GlobalInitializer::ExternShortArray { .. }
        | GlobalInitializer::ExternPointerArray { .. }
        | GlobalInitializer::ExternUnsignedCharArray { .. }
        | GlobalInitializer::ExternUnsignedCharMatrix { .. }
        | GlobalInitializer::ExternStructArray { .. }
        | GlobalInitializer::ExternStructObject { .. } => Err(CompileError::new(
            "internal error: extern global reached definition lowering",
        )),
    }
}

const fn lower_long_long_global_initializer(
    value: i64,
) -> (LoweredGlobalInitializer, GlobalBinding) {
    (
        LoweredGlobalInitializer::LongLong(value),
        GlobalBinding::LongLong,
    )
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

pub(in crate::ir) fn lower_short_array_global_initializer(
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

pub(in crate::ir) fn lower_pointer_array_initializer(
    initializer: &GlobalInitializer,
) -> Option<CompileResult<(LoweredGlobalInitializer, GlobalBinding)>> {
    match initializer {
        GlobalInitializer::PointerArray {
            referent,
            length,
            columns,
        } => Some(Ok((
            LoweredGlobalInitializer::PointerArray(*length),
            GlobalBinding::PointerArray {
                referent: referent.clone(),
                columns: *columns,
            },
        ))),
        GlobalInitializer::PointerStringArray { referent, values } => Some(Ok((
            LoweredGlobalInitializer::PointerStringArray(values.clone()),
            GlobalBinding::PointerArray {
                referent: referent.clone(),
                columns: None,
            },
        ))),
        GlobalInitializer::PointerNameArray {
            referent,
            values,
            length,
        } => Some(Ok((
            LoweredGlobalInitializer::PointerNameArray {
                values: values.clone(),
                length: *length,
            },
            GlobalBinding::PointerArray {
                referent: referent.clone(),
                columns: None,
            },
        ))),
        _ => None,
    }
}

pub(in crate::ir) fn lower_global_pointer_subscript_address(
    referent: Option<&str>,
    base: &str,
    index: usize,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    let stride = referent
        .and_then(pointer_arithmetic::byte_size)
        .unwrap_or_else(|| scalar_size(ScalarType::Int));
    let byte_offset = index
        .checked_mul(stride)
        .ok_or_else(|| CompileError::new("global pointer offset overflow"))?;
    Ok((
        LoweredGlobalInitializer::PointerGlobalOffset {
            base: base.to_owned(),
            byte_offset,
        },
        GlobalBinding::Pointer {
            referent: referent.map(ToOwned::to_owned),
        },
    ))
}

pub(in crate::ir) fn lower_int_constant_global(
    name: &str,
    global_name: &str,
    constants: &HashMap<String, i64>,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    let Some(value) = constants.get(name) else {
        return Err(CompileError::new(format!(
            "unknown global initializer constant: {name}"
        )));
    };
    Ok((
        LoweredGlobalInitializer::Int(i32::try_from(*value).map_err(|_| {
            CompileError::new(format!(
                "global int initializer does not fit i32: {global_name}"
            ))
        })?),
        GlobalBinding::Int,
    ))
}
