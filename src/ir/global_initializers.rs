use super::{
    GlobalBinding, LoweredGlobalInitializer, global_pointer_initializers,
    lower_scalar_global_initializer, scalar_size, struct_initializer,
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
    if let Some(lowered) = lower_scalar_global_initializer(global, constants)? {
        return Ok(lowered);
    }
    if let Some(lowered) =
        global_pointer_initializers::lower_pointer_scalar_global_initializer(&global.initializer)?
    {
        return Ok(lowered);
    }
    match &global.initializer {
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
        GlobalInitializer::PointerArray { .. }
        | GlobalInitializer::PointerStringArray { .. }
        | GlobalInitializer::PointerNameArray { .. } => Err(CompileError::new(
            "internal error: pointer array global reached fallback lowering",
        )),
        GlobalInitializer::PointerNull { .. }
        | GlobalInitializer::PointerString { .. }
        | GlobalInitializer::PointerName { .. }
        | GlobalInitializer::PointerSubscriptAddress { .. } => Err(CompileError::new(
            "internal error: pointer scalar global reached fallback lowering",
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
        GlobalInitializer::Int(_)
        | GlobalInitializer::LongLong(_)
        | GlobalInitializer::Double(_)
        | GlobalInitializer::ComplexReal { .. }
        | GlobalInitializer::ScalarZero(_)
        | GlobalInitializer::IntConstant(_)
        | GlobalInitializer::Extern(_)
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
                length: Some(*length),
                columns: *columns,
            },
        ))),
        GlobalInitializer::PointerStringArray { referent, values } => Some(Ok((
            LoweredGlobalInitializer::PointerStringArray(values.clone()),
            GlobalBinding::PointerArray {
                referent: referent.clone(),
                length: Some(values.len()),
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
                length: Some(*length),
                columns: None,
            },
        ))),
        _ => None,
    }
}
