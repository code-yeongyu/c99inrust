use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_floatlike_declarations::global_floatlike_scalar_type;
use super::global_int_initializers::parse_global_int_initializer;
use super::global_specifiers::{
    global_specifiers_are_extern_int, global_specifiers_are_extern_long_long,
    global_specifiers_are_extern_pointer, global_specifiers_are_int,
    global_specifiers_are_long_long, global_struct_specifier_name,
};
use super::pointer_referent_from_specifiers;
use super::token_scan::{
    previous_identifier_index, token_identifier, token_is_punctuator, top_level_comma_ranges,
    top_level_punctuator_index,
};
use super::{
    Constant, Global, GlobalInitializer, ScalarType, StructLayout,
    parse_integer_initializer_with_context,
};

pub(super) fn parse_global_extern_scalar(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    if top_level_punctuator_index(declaration, "=").is_some()
        || top_level_punctuator_index(declaration, "[").is_some()
    {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, declaration.len()) else {
        return Ok(None);
    };
    let specifiers = &declaration[..name_index];
    let initializer = if global_specifiers_are_extern_pointer(specifiers) {
        GlobalInitializer::ExternPointer {
            referent: pointer_referent_from_specifiers(specifiers),
        }
    } else if let Some(struct_name) = global_struct_specifier_name(specifiers, known_structs) {
        GlobalInitializer::ExternStructObject { struct_name }
    } else if let Some(scalar_type) = global_floatlike_scalar_type(specifiers, true) {
        GlobalInitializer::Extern(scalar_type)
    } else if global_specifiers_are_extern_int(specifiers) {
        GlobalInitializer::Extern(ScalarType::Int)
    } else if global_specifiers_are_extern_long_long(specifiers) {
        GlobalInitializer::Extern(ScalarType::LongLong)
    } else {
        return Ok(None);
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected extern global name"))?
        .to_owned();
    Ok(Some(Global::new(name, initializer)))
}

pub(super) fn parse_global_int(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let end_index = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    if top_level_punctuator_index(&declaration[..end_index], "[").is_some() {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, end_index) else {
        return Ok(None);
    };
    let specifiers = &declaration[..name_index];
    let is_long_long = global_specifiers_are_long_long(specifiers);
    if !is_long_long && !global_specifiers_are_int(specifiers, known_structs) {
        return Ok(None);
    }
    if declaration
        .get(end_index + 1)
        .is_some_and(|token| token_is_punctuator(token, "{"))
    {
        return Ok(None);
    }
    let initializer = if is_long_long {
        let value = if end_index == declaration.len() {
            0
        } else {
            parse_integer_initializer_with_context(
                &declaration[end_index + 1..],
                constants,
                sizeof_symbols,
            )?
        };
        GlobalInitializer::LongLong(value)
    } else if end_index == declaration.len() {
        GlobalInitializer::Int(0)
    } else {
        parse_global_int_initializer(&declaration[end_index + 1..], constants, sizeof_symbols)?
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global int name"))?
        .to_owned();
    Ok(Some(Global::new(name, initializer)))
}

pub(super) fn parse_global_int_declarator_list(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Option<Vec<Global>>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let ranges = top_level_comma_ranges(declaration);
    if ranges.len() <= 1 {
        return Ok(None);
    }
    let Some((first_start, first_end)) = ranges.first().copied() else {
        return Ok(None);
    };
    let first = &declaration[first_start..first_end];
    let first_end_index = top_level_punctuator_index(first, "=").unwrap_or(first.len());
    let Some(first_name_index) = previous_identifier_index(first, first_end_index) else {
        return Ok(None);
    };
    let base_specifiers = &first[..first_name_index];
    if !global_specifiers_are_int(base_specifiers, known_structs) {
        return Ok(None);
    }
    let mut globals = Vec::with_capacity(ranges.len());
    for (range_index, (start, end)) in ranges.iter().copied().enumerate() {
        let segment = &declaration[start..end];
        let end_index = top_level_punctuator_index(segment, "=").unwrap_or(segment.len());
        if top_level_punctuator_index(&segment[..end_index], "[").is_some() {
            return Ok(None);
        }
        let Some(name_index) = previous_identifier_index(segment, end_index) else {
            return Ok(None);
        };
        if range_index > 0 && !segment[..name_index].is_empty() {
            return Ok(None);
        }
        let initializer = if end_index == segment.len() {
            GlobalInitializer::Int(0)
        } else {
            parse_global_int_initializer(&segment[end_index + 1..], constants, &[])?
        };
        let name = token_identifier(&segment[name_index])
            .ok_or_else(|| CompileError::new("expected global int name"))?
            .to_owned();
        globals.push(Global::new(name, initializer));
    }
    Ok(Some(globals))
}
