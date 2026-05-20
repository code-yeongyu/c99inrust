use super::{
    GlobalBinding, LoweredGlobal, insert_builtin_libc_bindings, insert_global_binding,
    lower_defined_global_initializer,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Global, GlobalInitializer, StructLayout};
use std::collections::{HashMap, HashSet};

pub(in crate::ir) fn lower_globals(
    globals: &[Global],
    constants: &HashMap<String, i64>,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<(Vec<LoweredGlobal>, HashMap<String, GlobalBinding>)> {
    let mut lowered = Vec::with_capacity(globals.len());
    let mut bindings = HashMap::with_capacity(globals.len());
    let mut definitions = HashSet::with_capacity(globals.len());
    for global in globals {
        if let Some(binding) = lower_extern_global_binding(&global.initializer, structs)? {
            insert_global_binding(&mut bindings, &global.name, binding)?;
            continue;
        }
        let (initializer, binding) =
            lower_defined_global_initializer(global, constants, structs, &bindings)?;
        if !definitions.insert(global.name.clone()) {
            return Err(CompileError::new(format!(
                "duplicate global declaration: {}",
                global.name
            )));
        }
        insert_global_binding(&mut bindings, &global.name, binding)?;
        lowered.push(LoweredGlobal {
            name: global.name.clone(),
            is_static: global.is_static,
            initializer,
        });
    }
    insert_builtin_libc_bindings(&mut bindings);
    Ok((lowered, bindings))
}

pub(in crate::ir) fn lower_extern_global_binding(
    initializer: &GlobalInitializer,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<Option<GlobalBinding>> {
    let binding = match initializer {
        GlobalInitializer::Extern(scalar_type) => GlobalBinding::from_scalar_type(*scalar_type)?,
        GlobalInitializer::ExternPointer { referent } => GlobalBinding::Pointer {
            referent: referent.clone(),
        },
        GlobalInitializer::ExternIntArray => GlobalBinding::IntArray,
        GlobalInitializer::ExternShortArray {
            is_unsigned,
            columns,
        } => GlobalBinding::ShortArray {
            is_unsigned: *is_unsigned,
            columns: *columns,
        },
        GlobalInitializer::ExternPointerArray { referent, columns } => {
            GlobalBinding::PointerArray {
                referent: referent.clone(),
                columns: *columns,
            }
        }
        GlobalInitializer::ExternUnsignedCharArray { is_unsigned } => {
            GlobalBinding::UnsignedCharArray {
                is_unsigned: *is_unsigned,
            }
        }
        GlobalInitializer::ExternUnsignedCharMatrix {
            columns,
            is_unsigned,
        } => GlobalBinding::UnsignedCharMatrix {
            columns: *columns,
            is_unsigned: *is_unsigned,
        },
        GlobalInitializer::ExternStructArray { struct_name } => {
            let layout = structs.get(struct_name).ok_or_else(|| {
                CompileError::new(format!("unknown struct-array type: {struct_name}"))
            })?;
            GlobalBinding::StructArray {
                struct_name: struct_name.clone(),
                byte_size: layout.size,
                length: None,
                columns: None,
            }
        }
        GlobalInitializer::ExternStructObject { struct_name } => {
            let layout = structs.get(struct_name).ok_or_else(|| {
                CompileError::new(format!("unknown struct object type: {struct_name}"))
            })?;
            GlobalBinding::StructObject {
                struct_name: struct_name.clone(),
                byte_size: layout.size,
            }
        }
        GlobalInitializer::Int(_)
        | GlobalInitializer::IntArray(_)
        | GlobalInitializer::ShortArray { .. }
        | GlobalInitializer::IntMatrix { .. }
        | GlobalInitializer::DoubleArray { .. }
        | GlobalInitializer::IntConstant(_)
        | GlobalInitializer::PointerNull { .. }
        | GlobalInitializer::PointerString { .. }
        | GlobalInitializer::PointerSubscriptAddress { .. }
        | GlobalInitializer::PointerArray { .. }
        | GlobalInitializer::PointerStringArray { .. }
        | GlobalInitializer::PointerNameArray { .. }
        | GlobalInitializer::StructObject { .. }
        | GlobalInitializer::StructArray { .. }
        | GlobalInitializer::UnsignedCharArray { .. }
        | GlobalInitializer::UnsignedCharMatrix { .. } => return Ok(None),
    };
    Ok(Some(binding))
}
