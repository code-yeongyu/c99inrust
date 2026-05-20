use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Token, TokenKind};

use super::matchers::token_is_punctuator;

pub(in crate::parser) fn top_level_comma_ranges(tokens: &[Token]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, ",")
        {
            ranges.push((start, index));
            start = index + 1;
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    ranges.push((start, tokens.len()));
    ranges
}

pub(in crate::parser) fn top_level_punctuator_index(
    tokens: &[Token],
    expected: &str,
) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, expected)
        {
            return Some(index);
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    None
}

pub(in crate::parser) fn matching_top_level_bracket(
    tokens: &[Token],
    open_bracket: usize,
) -> Option<usize> {
    let mut bracket_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_bracket) {
        if token_is_punctuator(token, "[") {
            bracket_depth += 1;
            continue;
        }
        if token_is_punctuator(token, "]") {
            bracket_depth = bracket_depth.checked_sub(1)?;
            if bracket_depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

pub(in crate::parser) fn matching_top_level_paren(
    tokens: &[Token],
    open_paren: usize,
) -> Option<usize> {
    let mut paren_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_paren) {
        if token_is_punctuator(token, "(") {
            paren_depth += 1;
            continue;
        }
        if token_is_punctuator(token, ")") {
            paren_depth = paren_depth.checked_sub(1)?;
            if paren_depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

pub(in crate::parser) fn matching_top_level_brace(
    tokens: &[Token],
    open_brace: usize,
) -> Option<usize> {
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_brace) {
        if token_is_punctuator(token, "{") {
            brace_depth += 1;
            continue;
        }
        if token_is_punctuator(token, "}") {
            brace_depth = brace_depth.checked_sub(1)?;
            if brace_depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

pub(in crate::parser) fn update_depths(
    token: &Token,
    paren_depth: &mut usize,
    bracket_depth: &mut usize,
    brace_depth: &mut usize,
) {
    match &token.kind {
        TokenKind::Punctuator(value) if value == "(" => *paren_depth += 1,
        TokenKind::Punctuator(value) if value == ")" && *paren_depth > 0 => *paren_depth -= 1,
        TokenKind::Punctuator(value) if value == "[" => *bracket_depth += 1,
        TokenKind::Punctuator(value) if value == "]" && *bracket_depth > 0 => *bracket_depth -= 1,
        TokenKind::Punctuator(value) if value == "{" => *brace_depth += 1,
        TokenKind::Punctuator(value) if value == "}" && *brace_depth > 0 => *brace_depth -= 1,
        _ => {}
    }
}

pub(in crate::parser) fn decrease_depth(
    depth: &mut usize,
    token: &Token,
    delimiter: &str,
) -> CompileResult<()> {
    let Some(next_depth) = depth.checked_sub(1) else {
        return Err(CompileError::new(format!("unmatched closing {delimiter}"))
            .at(token.line, token.column));
    };
    *depth = next_depth;
    Ok(())
}
