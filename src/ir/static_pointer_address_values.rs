use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::StructLayout;

use super::{LoweredGlobalInitializer, global_address_offsets};

pub(in crate::ir) enum AddressConstant {
    Global { base: String, byte_offset: i64 },
    String { value: String, byte_offset: i64 },
}

pub(in crate::ir) fn global(base: &str, byte_offset: i64) -> AddressConstant {
    AddressConstant::Global {
        base: base.to_owned(),
        byte_offset,
    }
}

pub(in crate::ir) fn string(value: &str, byte_offset: i64) -> AddressConstant {
    AddressConstant::String {
        value: value.to_owned(),
        byte_offset,
    }
}

pub(in crate::ir) fn offset(
    address: AddressConstant,
    offset: i64,
    referent: Option<&str>,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<AddressConstant> {
    Ok(match address {
        AddressConstant::Global { base, byte_offset } => AddressConstant::Global {
            base,
            byte_offset: byte_offset
                .checked_add(scaled_offset(offset, referent, structs)?)
                .ok_or_else(|| CompileError::new("static local pointer offset overflow"))?,
        },
        AddressConstant::String { value, byte_offset } => AddressConstant::String {
            value,
            byte_offset: byte_offset
                .checked_add(offset)
                .ok_or_else(|| CompileError::new("static local pointer offset overflow"))?,
        },
    })
}

pub(in crate::ir) fn initializer(
    address: AddressConstant,
) -> CompileResult<LoweredGlobalInitializer> {
    match address {
        AddressConstant::Global { base, byte_offset } => global_offset(base, byte_offset),
        AddressConstant::String { value, byte_offset } => string_offset(value, byte_offset),
    }
}

pub(in crate::ir) fn scaled_offset(
    offset: i64,
    referent: Option<&str>,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<i64> {
    offset
        .checked_mul(
            i64::try_from(global_address_offsets::pointer_referent_size(
                referent, structs,
            ))
            .map_err(|_| CompileError::new("static local pointer stride is too large"))?,
        )
        .ok_or_else(|| CompileError::new("static local pointer offset overflow"))
}

fn string_offset(value: String, byte_offset: i64) -> CompileResult<LoweredGlobalInitializer> {
    usize::try_from(byte_offset)
        .map(|byte_offset| LoweredGlobalInitializer::PointerString(value, byte_offset))
        .map_err(|_| CompileError::new("static local string pointer offset must be nonnegative"))
}

fn global_offset(base: String, byte_offset: i64) -> CompileResult<LoweredGlobalInitializer> {
    Ok(LoweredGlobalInitializer::PointerGlobalOffset {
        base,
        byte_offset: usize::try_from(byte_offset)
            .map_err(|_| CompileError::new("static local pointer offset must be nonnegative"))?,
    })
}
