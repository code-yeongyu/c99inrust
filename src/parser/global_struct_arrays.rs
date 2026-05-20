use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::global_byte_declarations::parse_unsigned_char_array_length;
use super::global_specifiers::global_struct_specifier_name;
use super::token_scan::{
    matching_top_level_brace, matching_top_level_bracket, previous_identifier_index,
    token_has_keyword, token_identifier, token_is_punctuator, top_level_punctuator_index,
};
use super::{
    Constant, Global, GlobalInitializer, GlobalStructInitializerValue, StructLayout,
    global_struct_initializer,
};

pub(super) fn parse_global_struct_array(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
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
    let Some(struct_name) = global_struct_specifier_name(&declaration[..name_index], known_structs)
    else {
        return Ok(None);
    };
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global struct-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global struct-array name"))?
        .to_owned();
    let is_extern = token_has_keyword(&declaration[..name_index], Keyword::Extern);
    let columns =
        parse_global_struct_array_columns(declaration, close_bracket, is_extern, constants)?;
    let assign_index = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
        .map(|offset| close_bracket + 1 + offset);
    let initializer = if is_extern {
        GlobalInitializer::ExternStructArray { struct_name }
    } else {
        let length = parse_global_struct_array_length(
            declaration,
            open_bracket,
            close_bracket,
            columns,
            assign_index,
            constants,
        )?;
        let values = parse_global_struct_array_values(
            declaration,
            columns,
            assign_index,
            known_structs,
            constants,
        );
        if values.len() > length {
            return Err(CompileError::new(
                "too many global struct-array initializers",
            ));
        }
        GlobalInitializer::StructArray {
            struct_name,
            length,
            columns,
            values,
        }
    };
    Ok(Some(Global::new(name, initializer)))
}

fn parse_global_struct_array_columns(
    declaration: &[Token],
    close_bracket: usize,
    is_extern: bool,
    constants: &[Constant],
) -> CompileResult<Option<usize>> {
    if !declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        return Ok(None);
    }
    let second_open = close_bracket + 1;
    let Some(second_close) = matching_top_level_bracket(declaration, second_open) else {
        return Err(
            CompileError::new("unterminated global struct-matrix declarator").at(
                declaration[second_open].line,
                declaration[second_open].column,
            ),
        );
    };
    let tokens = &declaration[second_open + 1..second_close];
    if is_extern {
        parse_optional_symbolic_array_length(tokens, constants)
    } else {
        parse_unsigned_char_array_length(tokens, constants).map(Some)
    }
}

fn parse_global_struct_array_length(
    declaration: &[Token],
    open_bracket: usize,
    close_bracket: usize,
    columns: Option<usize>,
    assign_index: Option<usize>,
    constants: &[Constant],
) -> CompileResult<usize> {
    if let Some(columns) = columns {
        if open_bracket + 1 == close_bracket {
            return Err(CompileError::new("expected global struct-matrix row count"));
        }
        let rows = parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?;
        return rows
            .checked_mul(columns)
            .ok_or_else(|| CompileError::new("global struct-matrix size overflow"));
    }
    if let Some(assign_index) = assign_index {
        let initializer = &declaration[assign_index + 1..];
        return aggregate_initializer_length(initializer).ok_or_else(|| {
            CompileError::new("expected global struct-array initializer").at(
                declaration[assign_index].line,
                declaration[assign_index].column,
            )
        });
    }
    if open_bracket + 1 == close_bracket {
        return Err(CompileError::new("expected global struct-array length"));
    }
    parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket], constants)
}

fn parse_global_struct_array_values(
    declaration: &[Token],
    columns: Option<usize>,
    assign_index: Option<usize>,
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> Vec<Vec<GlobalStructInitializerValue>> {
    if columns.is_some() {
        return Vec::new();
    }
    let Some(assign_index) = assign_index else {
        return Vec::new();
    };
    global_struct_initializer::parse(&declaration[assign_index + 1..], known_structs, constants)
        .unwrap_or_default()
}

fn parse_optional_symbolic_array_length(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<usize>> {
    if let [
        Token {
            kind: TokenKind::Identifier(name),
            ..
        },
    ] = tokens
        && !constants.iter().any(|constant| constant.name == *name)
    {
        return Ok(None);
    }
    parse_unsigned_char_array_length(tokens, constants).map(Some)
}

fn aggregate_initializer_length(tokens: &[Token]) -> Option<usize> {
    if !tokens
        .first()
        .is_some_and(|token| token_is_punctuator(token, "{"))
    {
        return None;
    }
    let close = matching_top_level_brace(tokens, 0)?;
    let values = &tokens[1..close];
    if values.is_empty() {
        return Some(0);
    }
    let mut depth = 0usize;
    let mut count = 1usize;
    for token in values {
        if token_is_punctuator(token, "{") {
            depth += 1;
        } else if token_is_punctuator(token, "}") {
            depth = depth.checked_sub(1)?;
        } else if depth == 0 && token_is_punctuator(token, ",") {
            count += 1;
        }
    }
    Some(count)
}
