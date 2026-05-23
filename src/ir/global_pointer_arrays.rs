use std::collections::HashMap;

use crate::diagnostics::CompileResult;
use crate::parser::{GlobalInitializer, GlobalPointerAddress, StructLayout};

use super::{GlobalBinding, LoweredGlobalInitializer, global_address_offsets};

pub(in crate::ir) fn lower_pointer_array_initializer(
    initializer: &GlobalInitializer,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
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
        GlobalInitializer::PointerStringArray {
            referent,
            values,
            length,
        } => Some(Ok((
            LoweredGlobalInitializer::PointerStringArray {
                values: values.clone(),
                length: *length,
            },
            GlobalBinding::PointerArray {
                referent: referent.clone(),
                length: Some(*length),
                columns: None,
            },
        ))),
        GlobalInitializer::PointerNameArray {
            referent,
            values,
            length,
        } => Some(
            lower_pointer_name_array_values(referent.as_deref(), values, structs, global_bindings)
                .map(|values| {
                    (
                        LoweredGlobalInitializer::PointerNameArray {
                            values,
                            length: *length,
                        },
                        GlobalBinding::PointerArray {
                            referent: referent.clone(),
                            length: Some(*length),
                            columns: None,
                        },
                    )
                }),
        ),
        _ => None,
    }
}

fn lower_pointer_name_array_values(
    referent: Option<&str>,
    values: &[Option<GlobalPointerAddress>],
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<Vec<Option<(String, usize)>>> {
    values
        .iter()
        .map(|value| {
            value
                .as_ref()
                .map(|address| {
                    global_address_offsets::resolve(referent, address, structs, global_bindings)
                })
                .transpose()
        })
        .collect()
}
