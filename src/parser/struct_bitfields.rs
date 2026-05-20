use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Token, TokenKind};

use super::struct_layout_helpers::{StructFieldOutput, align_struct_offset};
use super::top_level_punctuator_index;

const BIT_FIELD_UNIT_BITS: usize = 32;
const BIT_FIELD_UNIT_BYTES: usize = 4;

pub(super) struct BitFieldState {
    bits_used: usize,
}

impl BitFieldState {
    pub(super) const fn new() -> Self {
        Self { bits_used: 0 }
    }

    pub(super) const fn clear(&mut self) {
        self.bits_used = 0;
    }
}

pub(super) fn bit_field_width(tokens: &[Token]) -> Option<usize> {
    let colon = top_level_punctuator_index(tokens, ":")?;
    let TokenKind::Integer(value) = tokens.get(colon + 1)?.kind else {
        return None;
    };
    usize::try_from(value).ok()
}

pub(super) fn push_bit_field(
    width: usize,
    is_union: bool,
    output: &mut StructFieldOutput<'_>,
    state: &mut BitFieldState,
) -> CompileResult<()> {
    *output.max_alignment = (*output.max_alignment).max(BIT_FIELD_UNIT_BYTES);
    if is_union {
        *output.offset = (*output.offset).max(BIT_FIELD_UNIT_BYTES);
        return Ok(());
    }
    if width == 0 {
        state.clear();
        *output.offset = align_struct_offset(*output.offset, BIT_FIELD_UNIT_BYTES)?;
        return Ok(());
    }
    let needs_new_unit = state.bits_used == 0
        || state
            .bits_used
            .checked_add(width)
            .is_none_or(|bits| bits > BIT_FIELD_UNIT_BITS);
    if needs_new_unit {
        *output.offset = align_struct_offset(*output.offset, BIT_FIELD_UNIT_BYTES)?;
        *output.offset = (*output.offset)
            .checked_add(BIT_FIELD_UNIT_BYTES)
            .ok_or_else(|| CompileError::new("struct bit-field size overflow"))?;
        state.clear();
    }
    state.bits_used = state
        .bits_used
        .checked_add(width)
        .ok_or_else(|| CompileError::new("struct bit-field width overflow"))?;
    Ok(())
}
