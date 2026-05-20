use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::token_scan::{
    matching_top_level_brace, token_identifier, token_is_punctuator, top_level_punctuator_index,
};
use super::{Expr, Parser};

pub(super) fn parse_string_array_initializer(tokens: &[Token]) -> CompileResult<Vec<String>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new(
            "expected global pointer-array initializer",
        ));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global pointer-array initializer")
                .at(first.line, first.column),
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global pointer-array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global pointer-array initializer")
                .at(token.line, token.column),
        );
    }

    let mut values = Vec::new();
    let mut start = 1usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global pointer-array initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        values.push(parse_string_initializer(&item[..item_len])?);
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    Ok(values)
}

pub(super) fn parse_identifier_array_initializer(tokens: &[Token]) -> CompileResult<Vec<String>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new(
            "expected global pointer-array initializer",
        ));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global pointer-array initializer")
                .at(first.line, first.column),
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global pointer-array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global pointer-array initializer")
                .at(token.line, token.column),
        );
    }

    let mut values = Vec::new();
    let mut start = 1usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global pointer-array initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        let [token] = &item[..item_len] else {
            return Err(
                CompileError::new("unsupported global pointer-array initializer")
                    .at(tokens[start].line, tokens[start].column),
            );
        };
        let Some(name) = token_identifier(token) else {
            return Err(
                CompileError::new("expected global pointer-array initializer name")
                    .at(token.line, token.column),
            );
        };
        values.push(name.to_owned());
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    Ok(values)
}

pub(super) fn parse_string_initializer(tokens: &[Token]) -> CompileResult<String> {
    if tokens.is_empty() {
        return Err(CompileError::new("expected global string initializer"));
    }
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs: &[],
        known_constants: &[],
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
    };
    let expr = parser.expression()?;
    if let Some(token) = parser.peek() {
        return Err(
            CompileError::new("unsupported global string initializer").at(token.line, token.column)
        );
    }
    let Expr::StringLiteral(value) = expr else {
        return Err(CompileError::new("expected global string initializer"));
    };
    Ok(value)
}
