use super::{GlobalBinding, LoweredGlobalInitializer, pointer_arithmetic, scalar_size};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{GlobalInitializer, ScalarType};

pub(in crate::ir) fn lower_pointer_scalar_global_initializer(
    initializer: &GlobalInitializer,
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
        } => return lower_global_pointer_subscript_address(referent.as_deref(), base, *index),
        _ => return Ok(None),
    };
    Ok(Some(lowered))
}

fn lower_global_pointer_subscript_address(
    referent: Option<&str>,
    base: &str,
    index: usize,
) -> CompileResult<Option<(LoweredGlobalInitializer, GlobalBinding)>> {
    let stride = referent
        .and_then(pointer_arithmetic::byte_size)
        .unwrap_or_else(|| scalar_size(ScalarType::Int));
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
