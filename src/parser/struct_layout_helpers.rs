use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Token, TokenKind};

use super::declarator_types::{integer_parameter_type, pointer_referent_from_specifiers};
use super::scalar_layout::scalar_field_type;
use super::token_scan::{
    matching_top_level_bracket, previous_identifier_index, token_identifier, token_is_punctuator,
    top_level_punctuator_index,
};
use super::type_recognition::supported_typedef_scalar;
use super::{Constant, FieldType, ScalarType, StructLayout, parse_unsigned_char_array_length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ArrayShape {
    pub(super) length: usize,
    pub(super) columns: Option<usize>,
}

pub(super) fn declarator_name_index(tokens: &[Token]) -> Option<usize> {
    let before = top_level_punctuator_index(tokens, "[").unwrap_or(tokens.len());
    previous_identifier_index(tokens, before)
}

pub(super) fn struct_field_array_shape(
    tokens: &[Token],
    constants: &[Constant],
) -> Option<ArrayShape> {
    let open_bracket = top_level_punctuator_index(tokens, "[")?;
    let close_bracket = matching_top_level_bracket(tokens, open_bracket)?;
    let length_tokens = &tokens[open_bracket + 1..close_bracket];
    let rows = if let Ok(length) = parse_unsigned_char_array_length(length_tokens, constants) {
        length
    } else {
        match &tokens.get(open_bracket + 1)?.kind {
            TokenKind::Integer(value) => usize::try_from(*value).ok().filter(|length| *length > 0),
            _ => Some(1),
        }?
    };
    if close_bracket <= open_bracket {
        return None;
    }
    let columns = tokens
        .get(close_bracket + 1)
        .filter(|token| token_is_punctuator(token, "["))
        .and_then(|_token| {
            let second_open = close_bracket + 1;
            let second_close = matching_top_level_bracket(tokens, second_open)?;
            parse_unsigned_char_array_length(&tokens[second_open + 1..second_close], constants).ok()
        });
    let length = columns.map_or(Some(rows), |columns| rows.checked_mul(columns))?;
    Some(ArrayShape { length, columns })
}

pub(super) fn struct_field_type(
    tokens: &[Token],
    known_structs: &[StructLayout],
    pointer_typedefs: &[String],
) -> Option<FieldType> {
    if tokens.iter().any(|token| token_is_punctuator(token, "*")) {
        return Some(FieldType::Pointer {
            referent: pointer_referent_from_specifiers(tokens),
        });
    }
    if let Some(scalar_type) = integer_parameter_type(tokens) {
        return Some(FieldType::Scalar(scalar_field_type(tokens, scalar_type)));
    }
    let name = tokens.iter().rev().find_map(token_identifier)?;
    if pointer_typedefs.iter().any(|known| known == name) {
        return Some(FieldType::Pointer { referent: None });
    }
    if known_structs.iter().any(|layout| layout.name == name) {
        return Some(FieldType::Struct(name.to_owned()));
    }
    let scalar_type = supported_typedef_scalar(name).unwrap_or(ScalarType::Int);
    Some(FieldType::Scalar(scalar_field_type(tokens, scalar_type)))
}

pub(super) fn field_type_size(
    field_type: &FieldType,
    known_structs: &[StructLayout],
) -> CompileResult<usize> {
    match field_type {
        FieldType::Scalar(scalar_type) => Ok(scalar_type.byte_size),
        FieldType::Pointer { .. } => Ok(8),
        FieldType::Array {
            element_size,
            length,
            ..
        } => element_size
            .checked_mul(*length)
            .ok_or_else(|| CompileError::new("struct array field size overflow")),
        FieldType::StructArray {
            struct_name,
            length,
        } => known_structs
            .iter()
            .find(|layout| layout.name == *struct_name)
            .map(|layout| layout.size)
            .ok_or_else(|| CompileError::new(format!("unknown struct field type: {struct_name}")))?
            .checked_mul(*length)
            .ok_or_else(|| CompileError::new("struct array field size overflow")),
        FieldType::Struct(name) => known_structs
            .iter()
            .find(|layout| layout.name == *name)
            .map(|layout| layout.size)
            .ok_or_else(|| CompileError::new(format!("unknown struct field type: {name}"))),
    }
}

pub(super) fn field_type_alignment(
    field_type: &FieldType,
    known_structs: &[StructLayout],
) -> CompileResult<usize> {
    match field_type {
        FieldType::Array { element_size, .. } => Ok((*element_size).clamp(1, 8)),
        FieldType::StructArray { struct_name, .. } | FieldType::Struct(struct_name) => {
            let layout = known_structs
                .iter()
                .find(|layout| layout.name == *struct_name)
                .ok_or_else(|| {
                    CompileError::new(format!("unknown struct field type: {struct_name}"))
                })?;
            struct_layout_alignment(layout, known_structs)
        }
        _ => field_type_size(field_type, known_structs).map(|size| size.clamp(1, 8)),
    }
}

fn struct_layout_alignment(
    layout: &StructLayout,
    known_structs: &[StructLayout],
) -> CompileResult<usize> {
    layout.fields.iter().try_fold(1usize, |alignment, field| {
        field_type_alignment(&field.field_type, known_structs)
            .map(|field_alignment| alignment.max(field_alignment))
    })
}

pub(super) fn align_struct_offset(offset: usize, alignment: usize) -> CompileResult<usize> {
    let remainder = offset % alignment;
    if remainder == 0 {
        return Ok(offset);
    }
    offset
        .checked_add(alignment - remainder)
        .ok_or_else(|| CompileError::new("struct offset overflow"))
}
