use std::collections::HashMap;

use super::{GlobalBinding, LoweredGlobalInitializer, global_address_offsets};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{GlobalInitializer, StructLayout};

pub(in crate::ir) fn lower_pointer_scalar_global_initializer(
    initializer: &GlobalInitializer,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<Option<(LoweredGlobalInitializer, GlobalBinding)>> {
    let lowered = match initializer {
        GlobalInitializer::PointerNull { referent } => (
            LoweredGlobalInitializer::PointerNull,
            GlobalBinding::Pointer {
                referent: referent.clone(),
            },
        ),
        GlobalInitializer::PointerString {
            referent,
            value,
            byte_offset,
        } => (
            LoweredGlobalInitializer::PointerString(value.clone(), *byte_offset),
            GlobalBinding::Pointer {
                referent: referent.clone(),
            },
        ),
        GlobalInitializer::PointerName { referent, value } => (
            LoweredGlobalInitializer::PointerGlobalOffset {
                base: value.clone(),
                byte_offset: 0,
            },
            GlobalBinding::Pointer {
                referent: referent.clone(),
            },
        ),
        GlobalInitializer::PointerSubscriptAddress {
            referent,
            base,
            index,
        } => {
            return lower_global_pointer_subscript_address(
                referent.as_deref(),
                base,
                *index,
                structs,
            );
        }
        GlobalInitializer::PointerMemberAddress { referent, address } => {
            let (base, byte_offset) = global_address_offsets::resolve(
                referent.as_deref(),
                address,
                structs,
                global_bindings,
            )?;
            return Ok(Some((
                LoweredGlobalInitializer::PointerGlobalOffset { base, byte_offset },
                GlobalBinding::Pointer {
                    referent: referent.clone(),
                },
            )));
        }
        _ => return Ok(None),
    };
    Ok(Some(lowered))
}

fn lower_global_pointer_subscript_address(
    referent: Option<&str>,
    base: &str,
    index: usize,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<Option<(LoweredGlobalInitializer, GlobalBinding)>> {
    let stride = global_address_offsets::pointer_referent_size(referent, structs);
    let byte_offset = index
        .checked_mul(stride)
        .ok_or_else(|| CompileError::new("global pointer offset overflow"))?;
    Ok(Some((
        LoweredGlobalInitializer::PointerGlobalOffset {
            base: base.to_owned(),
            byte_offset,
        },
        GlobalBinding::Pointer {
            referent: referent.map(ToOwned::to_owned),
        },
    )))
}
