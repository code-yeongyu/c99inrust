use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token};

use super::global_byte_declarations::parse_unsigned_char_array_length;
use super::global_int_initializers::{
    parse_int_array_initializer, parse_int_matrix_initializer, parse_short_initializer_values,
};
use super::global_specifiers::{global_specifiers_are_extern_int, global_specifiers_are_short};
use super::token_scan::{
    matching_top_level_bracket, previous_identifier_index, token_has_keyword, token_identifier,
    token_is_punctuator, top_level_punctuator_index,
};
use super::{Constant, Global, GlobalInitializer};

pub(super) fn parse_global_extern_int_array(tokens: &[Token]) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_extern_int(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated extern global int-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    if top_level_punctuator_index(&declaration[close_bracket + 1..], "=").is_some() {
        return Ok(None);
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected extern global int-array name"))?
        .to_owned();
    Ok(Some(Global::new(name, GlobalInitializer::ExternIntArray)))
}

pub(super) fn parse_global_short_array(
    tokens: &[Token],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    let specifiers = &declaration[..name_index];
    if !global_specifiers_are_short(specifiers) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global short-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global short-array name"))?
        .to_owned();
    let is_extern = token_has_keyword(specifiers, Keyword::Extern);
    let is_unsigned = token_has_keyword(specifiers, Keyword::Unsigned);
    if declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        return parse_global_short_matrix(
            ShortArrayDeclarator {
                declaration,
                open_bracket,
                close_bracket,
                name: &name,
                is_extern,
                is_unsigned,
            },
            constants,
            sizeof_symbols,
        );
    }
    if is_extern {
        return Ok(Some(Global::new(
            name,
            GlobalInitializer::ExternShortArray {
                is_unsigned,
                columns: None,
            },
        )));
    }
    let explicit_length = if open_bracket + 1 == close_bracket {
        None
    } else {
        Some(parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?)
    };
    let assign_index = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
        .map(|offset| close_bracket + 1 + offset);
    let values = if let Some(assign_index) = assign_index {
        parse_short_initializer_values(
            parse_int_array_initializer(
                &declaration[assign_index + 1..],
                explicit_length,
                constants,
                sizeof_symbols,
            )?,
            is_unsigned,
        )?
    } else {
        let Some(length) = explicit_length else {
            return Err(CompileError::new("expected short array length"));
        };
        vec![0; length]
    };
    Ok(Some(Global::new(
        name,
        GlobalInitializer::ShortArray {
            values,
            is_unsigned,
            columns: None,
        },
    )))
}

fn parse_global_short_matrix(
    spec: ShortArrayDeclarator<'_>,
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let second_open = spec.close_bracket + 1;
    let Some(second_close) = matching_top_level_bracket(spec.declaration, second_open) else {
        return Err(
            CompileError::new("unterminated global short-matrix declarator").at(
                spec.declaration[second_open].line,
                spec.declaration[second_open].column,
            ),
        );
    };
    let columns = parse_unsigned_char_array_length(
        &spec.declaration[second_open + 1..second_close],
        constants,
    )?;
    if spec.is_extern {
        return Ok(Some(Global::new(
            spec.name.to_owned(),
            GlobalInitializer::ExternShortArray {
                is_unsigned: spec.is_unsigned,
                columns: Some(columns),
            },
        )));
    }
    let rows = parse_unsigned_char_array_length(
        &spec.declaration[spec.open_bracket + 1..spec.close_bracket],
        constants,
    )?;
    let length = rows
        .checked_mul(columns)
        .ok_or_else(|| CompileError::new("global short matrix size overflow"))?;
    let values = if let Some(assign_index) =
        top_level_punctuator_index(&spec.declaration[second_close + 1..], "=")
    {
        let assign_index = second_close + 1 + assign_index;
        parse_short_initializer_values(
            parse_int_matrix_initializer(
                &spec.declaration[assign_index + 1..],
                rows,
                columns,
                constants,
                sizeof_symbols,
            )?,
            spec.is_unsigned,
        )?
    } else {
        vec![0; length]
    };
    Ok(Some(Global::new(
        spec.name.to_owned(),
        GlobalInitializer::ShortArray {
            values,
            is_unsigned: spec.is_unsigned,
            columns: Some(columns),
        },
    )))
}

#[derive(Clone, Copy)]
struct ShortArrayDeclarator<'a> {
    declaration: &'a [Token],
    open_bracket: usize,
    close_bracket: usize,
    name: &'a str,
    is_extern: bool,
    is_unsigned: bool,
}
