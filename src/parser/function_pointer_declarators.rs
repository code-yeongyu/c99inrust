use crate::front_end::lexer::Token;

use super::token_scan::{
    matching_top_level_paren, token_identifier, token_is_punctuator, update_depths,
};
use super::{ReturnType, ScalarType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FunctionPointerVariableDeclarator {
    pub(super) name: String,
    pub(super) pointer_depth: usize,
    pub(super) consumed: usize,
    pub(super) specifier_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PointerReturnDeclarator {
    pub(super) name: String,
    pub(super) parameter_open: usize,
    pub(super) suffix_end: usize,
    pub(super) specifier_end: usize,
}

pub(super) fn function_pointer_variable(
    tokens: &[Token],
) -> Option<FunctionPointerVariableDeclarator> {
    let open = top_level_pointer_wrapper_open(tokens)?;
    let (name, pointer_depth, after_name) = pointer_name_after_stars(tokens, open)?;
    if !tokens
        .get(after_name)
        .is_some_and(|token| token_is_punctuator(token, ")"))
    {
        return None;
    }
    let return_params_open = after_name + 1;
    let return_params_close = matching_return_params(tokens, return_params_open)?;
    Some(FunctionPointerVariableDeclarator {
        name,
        pointer_depth,
        consumed: return_params_close + 1,
        specifier_end: open,
    })
}

pub(super) const fn function_referent_for_return(return_type: ReturnType) -> &'static str {
    match return_type {
        ReturnType::Double => "function double",
        ReturnType::LongDouble => "function long double",
        ReturnType::Pointer => "function pointer",
        ReturnType::Int | ReturnType::Void => "function int",
    }
}

pub(super) const fn function_referent_for_scalar(return_type: ScalarType) -> &'static str {
    match return_type {
        ScalarType::Double => "function double",
        ScalarType::LongDouble => "function long double",
        ScalarType::Pointer => "function pointer",
        _ => "function int",
    }
}

pub(super) fn pointer_return_declarator(tokens: &[Token]) -> Option<PointerReturnDeclarator> {
    let open = top_level_pointer_wrapper_open(tokens)?;
    let (name, _pointer_depth, parameter_open) = pointer_name_after_stars(tokens, open)?;
    let parameter_close = matching_top_level_paren(tokens, parameter_open)?;
    if !tokens
        .get(parameter_close + 1)
        .is_some_and(|token| token_is_punctuator(token, ")"))
    {
        return None;
    }
    let return_params_open = parameter_close + 2;
    let return_params_close = matching_return_params(tokens, return_params_open)?;
    Some(PointerReturnDeclarator {
        name,
        parameter_open,
        suffix_end: return_params_close + 1,
        specifier_end: open,
    })
}

fn top_level_pointer_wrapper_open(tokens: &[Token]) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, "(")
            && tokens
                .get(index + 1)
                .is_some_and(|next| token_is_punctuator(next, "*"))
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

fn pointer_name_after_stars(tokens: &[Token], open: usize) -> Option<(String, usize, usize)> {
    let mut index = open + 1;
    let mut pointer_depth = 0usize;
    while tokens
        .get(index)
        .is_some_and(|token| token_is_punctuator(token, "*"))
    {
        pointer_depth += 1;
        index += 1;
    }
    let name = tokens.get(index).and_then(token_identifier)?.to_owned();
    Some((name, pointer_depth, index + 1))
}

fn matching_return_params(tokens: &[Token], open: usize) -> Option<usize> {
    if !tokens
        .get(open)
        .is_some_and(|token| token_is_punctuator(token, "("))
    {
        return None;
    }
    matching_top_level_paren(tokens, open)
}
