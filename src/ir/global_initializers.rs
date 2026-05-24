use super::{
    GlobalBinding, LoweredGlobalInitializer, global_array_initializers, global_pointer_arrays,
    global_pointer_initializers, lower_scalar_global_initializer, struct_initializer,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Global, GlobalInitializer, StructLayout};
use std::collections::HashMap;

pub(in crate::ir) fn lower_defined_global_initializer(
    global: &Global,
    constants: &HashMap<String, i64>,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    if let Some(lowered) = global_pointer_arrays::lower_pointer_array_initializer(
        &global.initializer,
        structs,
        global_bindings,
    ) {
        return lowered;
    }
    if let Some(lowered) = lower_scalar_global_initializer(global, constants)? {
        return Ok(lowered);
    }
    if let Some(lowered) = global_pointer_initializers::lower_pointer_scalar_global_initializer(
        &global.initializer,
        structs,
        global_bindings,
    )? {
        return Ok(lowered);
    }
    if let Some(lowered) = global_array_initializers::lower_array_global_initializer(global)? {
        return Ok(lowered);
    }
    if let Some(lowered) =
        lower_struct_global_initializer(&global.initializer, structs, global_bindings)?
    {
        return Ok(lowered);
    }
    invalid_defined_global_initializer(&global.initializer)
}

fn lower_struct_global_initializer(
    initializer: &GlobalInitializer,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<Option<(LoweredGlobalInitializer, GlobalBinding)>> {
    match initializer {
        GlobalInitializer::StructObject {
            struct_name,
            values,
        } => struct_initializer::lower_struct_object_global(
            struct_name,
            values,
            structs,
            global_bindings,
        )
        .map(Some),
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
        )
        .map(Some),
        _ => Ok(None),
    }
}

fn invalid_defined_global_initializer(
    initializer: &GlobalInitializer,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    match initializer {
        GlobalInitializer::PointerArray { .. }
        | GlobalInitializer::PointerStringArray { .. }
        | GlobalInitializer::PointerNameArray { .. } => Err(CompileError::new(
            "internal error: pointer array global reached fallback lowering",
        )),
        GlobalInitializer::PointerNull { .. }
        | GlobalInitializer::PointerString { .. }
        | GlobalInitializer::PointerName { .. }
        | GlobalInitializer::PointerSubscriptAddress { .. }
        | GlobalInitializer::PointerMemberAddress { .. } => Err(CompileError::new(
            "internal error: pointer scalar global reached fallback lowering",
        )),
        GlobalInitializer::LongLongArray(_)
        | GlobalInitializer::IntArray(_)
        | GlobalInitializer::BoolArray(_)
        | GlobalInitializer::ShortArray { .. }
        | GlobalInitializer::IntMatrix { .. }
        | GlobalInitializer::StructObject { .. }
        | GlobalInitializer::StructArray { .. } => Err(CompileError::new(
            "internal error: supported global reached fallback lowering",
        )),
        GlobalInitializer::UnsignedCharArray { .. }
        | GlobalInitializer::UnsignedCharMatrix { .. } => Err(CompileError::new(
            "internal error: byte global reached fallback lowering",
        )),
        GlobalInitializer::Int(_)
        | GlobalInitializer::Bool(_)
        | GlobalInitializer::LongLong(_)
        | GlobalInitializer::Double(_)
        | GlobalInitializer::ComplexReal { .. }
        | GlobalInitializer::DoubleArray { .. }
        | GlobalInitializer::ScalarArray { .. }
        | GlobalInitializer::ScalarArrayValues { .. }
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
