use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::Constant;
use super::integer_initializer::parse_integer_initializer_with_constants;
use super::token_scan::{
    last_top_level_identifier, matching_top_level_brace, token_has_keyword, token_identifier,
    token_is_punctuator, top_level_punctuator_index,
};

pub(super) fn parse_enum_constants(
    tokens: &[Token],
    known_constants: &[Constant],
) -> CompileResult<Vec<Constant>> {
    if !tokens_start_enum_declaration(tokens) {
        return Ok(Vec::new());
    }
    let Some(open_brace) = top_level_punctuator_index(tokens, "{") else {
        return Ok(Vec::new());
    };
    let Some(close_brace) = matching_top_level_brace(tokens, open_brace) else {
        return Err(CompileError::new("unterminated enum declaration")
            .at(tokens[open_brace].line, tokens[open_brace].column));
    };
    parse_enum_body(&tokens[open_brace + 1..close_brace], known_constants)
}

fn parse_enum_body(tokens: &[Token], known_constants: &[Constant]) -> CompileResult<Vec<Constant>> {
    let mut constants = Vec::new();
    let mut available_constants = known_constants.to_vec();
    let mut value = 0i64;
    let mut index = 0usize;
    while index < tokens.len() {
        if token_is_punctuator(&tokens[index], ",") {
            index += 1;
            continue;
        }
        let Some(name) = token_identifier(&tokens[index]) else {
            return Ok(Vec::new());
        };
        index += 1;
        if tokens
            .get(index)
            .is_some_and(|token| token_is_punctuator(token, "="))
        {
            let initializer_start = index + 1;
            let initializer_end = next_enum_separator(tokens, initializer_start);
            value = parse_integer_initializer_with_constants(
                &tokens[initializer_start..initializer_end],
                &available_constants,
            )?;
            index = initializer_end;
        }
        let constant = Constant {
            name: name.to_owned(),
            value,
        };
        available_constants.push(constant.clone());
        constants.push(constant);
        value = value
            .checked_add(1)
            .ok_or_else(|| CompileError::new("enum constant overflow"))?;
    }
    Ok(constants)
}

fn tokens_start_enum_declaration(tokens: &[Token]) -> bool {
    matches!(
        tokens.first().map(|token| &token.kind),
        Some(TokenKind::Keyword(Keyword::Enum))
    ) || matches!(
        (
            tokens.first().map(|token| &token.kind),
            tokens.get(1).map(|token| &token.kind)
        ),
        (
            Some(TokenKind::Keyword(Keyword::Typedef)),
            Some(TokenKind::Keyword(Keyword::Enum))
        )
    )
}

pub(super) fn enum_typedef_name(tokens: &[Token]) -> Option<String> {
    if !token_has_keyword(tokens, Keyword::Typedef) || !token_has_keyword(tokens, Keyword::Enum) {
        return None;
    }
    last_top_level_identifier(tokens)
}

fn next_enum_separator(tokens: &[Token], start: usize) -> usize {
    tokens[start..]
        .iter()
        .position(|token| token_is_punctuator(token, ","))
        .map_or(tokens.len(), |offset| start + offset)
}
