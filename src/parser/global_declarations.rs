use crate::diagnostics::CompileResult;
use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::external_declarations::{
    classify_external_item, function_pointer_name, top_level_function_open_paren,
};
use super::global_byte_declarations::parse_global_unsigned_char_array;
use super::global_double_declarations::parse_global_double_array;
use super::global_int_arrays::parse_global_int_array;
use super::global_pointer_arrays::{
    parse_global_extern_pointer_array, parse_global_pointer_array, parse_global_pointer_name_array,
    parse_global_pointer_string_array,
};
use super::global_pointer_scalars::parse_global_pointer;
use super::global_scalar_declarations::{
    parse_global_extern_scalar, parse_global_int, parse_global_int_declarator_list,
};
use super::global_short_declarations::{parse_global_extern_int_array, parse_global_short_array};
use super::global_specifiers::global_specifiers_are_static_const_char;
use super::global_struct_arrays::parse_global_struct_array;
use super::token_scan::{
    last_token_is_punctuator, matching_top_level_bracket, previous_identifier_index,
    token_has_keyword, top_level_punctuator_index,
};
use super::{
    Constant, ExternalItem, Global, GlobalInitializer, StructLayout, global_struct_object,
};

pub(super) fn parse_supported_global_declaration(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    if last_token_is_punctuator(tokens, "}") || !last_token_is_punctuator(tokens, ";") {
        return Ok(None);
    }
    if let Some(global) = parse_global_function_pointer(tokens) {
        return Ok(Some(global));
    }
    if top_level_function_open_paren(tokens).is_some() {
        return Ok(None);
    }
    if let Some(global) = parse_global_unsigned_char_array(tokens, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer_string_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer_name_array(tokens, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer_array(tokens, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_extern_pointer_array(tokens, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_struct_array(tokens, known_structs, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = global_struct_object::parse(tokens, known_structs, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_short_array(tokens, constants, sizeof_symbols)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_extern_int_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_double_array(tokens, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_int_array(tokens, known_structs, constants, sizeof_symbols)?
    {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_extern_scalar(tokens, known_structs)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer(tokens, constants)? {
        return Ok(Some(global));
    }
    parse_global_int(tokens, known_structs, constants, sizeof_symbols)
}

pub(super) fn parse_supported_global_declarations(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Vec<Global>> {
    if let Some(globals) = parse_global_int_declarator_list(tokens, known_structs, constants)? {
        return Ok(with_global_linkage(tokens, globals));
    }
    parse_supported_global_declaration(tokens, known_structs, constants, sizeof_symbols).map(
        |global| {
            let globals = global.map_or_else(Vec::new, |global| vec![global]);
            with_global_linkage(tokens, globals)
        },
    )
}

fn with_global_linkage(tokens: &[Token], mut globals: Vec<Global>) -> Vec<Global> {
    let is_static = token_has_keyword(tokens, Keyword::Static);
    for global in &mut globals {
        global.is_static = is_static;
    }
    globals
}

fn parse_global_function_pointer(tokens: &[Token]) -> Option<Global> {
    let declaration = tokens.get(..tokens.len().checked_sub(1)?)?;
    let name = function_pointer_name(declaration)?;
    let initializer = if token_has_keyword(declaration, Keyword::Extern) {
        GlobalInitializer::ExternPointer { referent: None }
    } else {
        GlobalInitializer::PointerNull { referent: None }
    };
    Some(Global::new(name, initializer))
}

pub(super) fn unsupported_data_declaration_blocks_empty_unit(tokens: &[Token]) -> bool {
    if ignorable_static_const_char_array(tokens) {
        return false;
    }
    if declaration_only_extern(tokens) {
        return false;
    }
    matches!(
        classify_external_item(tokens),
        Some(ExternalItem::Declaration { .. })
    )
}

fn declaration_only_extern(tokens: &[Token]) -> bool {
    token_has_keyword(tokens, Keyword::Extern) && top_level_punctuator_index(tokens, "=").is_none()
}

fn ignorable_static_const_char_array(tokens: &[Token]) -> bool {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return false;
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return false;
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return false;
    };
    if !global_specifiers_are_static_const_char(&declaration[..name_index]) {
        return false;
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return false;
    };
    let Some(assign_index) = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    else {
        return false;
    };
    let initializer = &declaration[close_bracket + 2 + assign_index..];
    matches!(
        initializer,
        [Token {
            kind: TokenKind::StringLiteral(_),
            ..
        }]
    )
}
