use crate::front_end::lexer::{Keyword, Token};

use super::function_pointer_declarators::{function_pointer_variable, pointer_return_declarator};
use super::pointer_referent_from_specifiers;
use super::token_scan::{
    array_declarator_name, last_token_is_punctuator, last_top_level_identifier,
    previous_identifier, previous_identifier_index, token_has_keyword, token_identifier,
    token_is_keyword, token_is_punctuator, update_depths,
};
use super::type_recognition::supported_return_type;
use super::{ExternalItem, PointerReturnFunction, ReturnType};

pub(super) fn classify_external_item(tokens: &[Token]) -> Option<ExternalItem> {
    if token_has_keyword(tokens, Keyword::Typedef) {
        return typedef_name(tokens).map(|name| ExternalItem::Typedef { name });
    }
    if let Some(name) = struct_forward_name(tokens) {
        return Some(ExternalItem::StructForward { name });
    }
    if let Some(name) = function_pointer_name(tokens) {
        return Some(ExternalItem::Declaration { name });
    }
    if let Some(name) = normal_function_name(tokens) {
        if last_token_is_punctuator(tokens, "}") {
            return Some(ExternalItem::FunctionDefinition { name });
        }
        return Some(ExternalItem::Prototype { name });
    }
    declaration_name(tokens).map(|name| ExternalItem::Declaration { name })
}

pub(super) fn function_definition_name(tokens: &[Token]) -> Option<String> {
    if last_token_is_punctuator(tokens, "}") {
        return pointer_return_declarator(tokens)
            .map(|declarator| declarator.name)
            .or_else(|| normal_function_name(tokens));
    }
    None
}

pub(super) fn function_prototype_name(tokens: &[Token]) -> Option<String> {
    if last_token_is_punctuator(tokens, "}") || function_pointer_name(tokens).is_some() {
        return None;
    }
    let open_index = top_level_function_open_paren(tokens)?;
    let name_index = previous_identifier_index(tokens, open_index)?;
    supported_return_type(&tokens[..name_index])?;
    token_identifier(&tokens[name_index]).map(ToOwned::to_owned)
}

pub(super) fn pointer_return_function(tokens: &[Token]) -> Option<PointerReturnFunction> {
    if let Some(declarator) = pointer_return_declarator(tokens) {
        supported_return_type(&tokens[..declarator.specifier_end])?;
        return Some(PointerReturnFunction {
            name: declarator.name,
            referent: None,
        });
    }
    let open_index = top_level_function_open_paren(tokens)?;
    let name_index = previous_identifier_index(tokens, open_index)?;
    if supported_return_type(&tokens[..name_index]) != Some(ReturnType::Pointer) {
        return None;
    }
    let name = token_identifier(&tokens[name_index])?.to_owned();
    Some(PointerReturnFunction {
        name,
        referent: pointer_referent_from_specifiers(&tokens[..name_index]),
    })
}

pub(super) fn function_definition_has_supported_signature(tokens: &[Token]) -> bool {
    if let Some(declarator) = pointer_return_declarator(tokens) {
        if supported_return_type(&tokens[..declarator.specifier_end]).is_none() {
            return false;
        }
        return function_body_follows(tokens, declarator.suffix_end);
    }
    let Some(open_index) = top_level_function_open_paren(tokens) else {
        return false;
    };
    let Some(name_index) = previous_identifier_index(tokens, open_index) else {
        return false;
    };
    if supported_return_type(&tokens[..name_index]).is_none() {
        return false;
    }
    let mut paren_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        if token_is_punctuator(token, "(") {
            paren_depth += 1;
            continue;
        }
        if token_is_punctuator(token, ")") {
            if paren_depth == 0 {
                return false;
            }
            paren_depth -= 1;
            if paren_depth == 0 {
                return function_body_follows(tokens, index + 1);
            }
        }
    }
    false
}

fn function_body_follows(tokens: &[Token], start: usize) -> bool {
    if tokens
        .get(start)
        .is_some_and(|next| token_is_punctuator(next, "{"))
    {
        return true;
    }
    let mut saw_parameter_declaration = false;
    for token in &tokens[start..] {
        if token_is_punctuator(token, ";") {
            saw_parameter_declaration = true;
            continue;
        }
        if token_is_punctuator(token, "{") {
            return saw_parameter_declaration;
        }
    }
    false
}

fn typedef_name(tokens: &[Token]) -> Option<String> {
    function_pointer_name(tokens).or_else(|| last_top_level_identifier(tokens))
}

pub(super) fn function_pointer_typedef_name(tokens: &[Token]) -> Option<String> {
    token_has_keyword(tokens, Keyword::Typedef)
        .then(|| function_pointer_name(tokens))
        .flatten()
}

fn struct_forward_name(tokens: &[Token]) -> Option<String> {
    let meaningful = tokens
        .iter()
        .filter(|token| !token_is_punctuator(token, ";"))
        .collect::<Vec<_>>();
    if meaningful.len() != 2 {
        return None;
    }
    if !token_is_keyword(meaningful[0], Keyword::Struct) {
        return None;
    }
    token_identifier(meaningful[1]).map(ToOwned::to_owned)
}

fn declaration_name(tokens: &[Token]) -> Option<String> {
    function_pointer_name(tokens)
        .or_else(|| array_declarator_name(tokens))
        .or_else(|| last_top_level_identifier(tokens))
}

pub(super) fn function_pointer_cast_type(tokens: &[Token]) -> bool {
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
            && tokens
                .get(index + 2)
                .is_some_and(|next| token_is_punctuator(next, ")"))
        {
            return true;
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    false
}

pub(super) fn function_pointer_name(tokens: &[Token]) -> Option<String> {
    function_pointer_variable(tokens).map(|declarator| declarator.name)
}

fn normal_function_name(tokens: &[Token]) -> Option<String> {
    let open_index = top_level_function_open_paren(tokens)?;
    previous_identifier(tokens, open_index).map(ToOwned::to_owned)
}

pub(super) fn top_level_function_open_paren(tokens: &[Token]) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut saw_assignment = false;
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 {
            if token_is_punctuator(token, "=") {
                saw_assignment = true;
            }
            if !saw_assignment && token_is_punctuator(token, "(") {
                return Some(index);
            }
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
