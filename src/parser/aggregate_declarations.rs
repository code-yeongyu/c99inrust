use crate::diagnostics::CompileResult;
use crate::front_end::lexer::{Keyword, Token};

use super::doom_layout;
use super::token_scan::{
    last_top_level_identifier, matching_top_level_brace, token_has_keyword, token_identifier,
    token_is_keyword, top_level_punctuator_index,
};
use super::{Constant, StructLayout, StructParseContext, parse_struct_fields};

pub(super) fn parse_struct_typedef(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
    pointer_typedefs: &[String],
) -> CompileResult<Option<Vec<StructLayout>>> {
    if !token_has_keyword(tokens, Keyword::Typedef) {
        return Ok(None);
    }
    if !token_has_keyword(tokens, Keyword::Struct) && !token_has_keyword(tokens, Keyword::Union) {
        return Ok(parse_struct_alias_typedef(tokens, known_structs).map(|layout| vec![layout]));
    }
    let Some(open_brace) = top_level_punctuator_index(tokens, "{") else {
        return Ok(None);
    };
    let is_union = tokens[..open_brace]
        .iter()
        .any(|token| token_is_keyword(token, Keyword::Union));
    let Some(close_brace) = matching_top_level_brace(tokens, open_brace) else {
        return Ok(None);
    };
    let Some(name) = last_top_level_identifier(tokens) else {
        return Ok(None);
    };
    let mut available_structs = known_structs.to_vec();
    let mut layouts = Vec::new();
    let mut context = StructParseContext {
        parent_name: &name,
        available_structs: &mut available_structs,
        constants,
        pointer_typedefs,
        nested_layouts: &mut layouts,
    };
    let Some((fields, size)) =
        parse_struct_fields(&tokens[open_brace + 1..close_brace], is_union, &mut context)?
    else {
        return Ok(None);
    };
    layouts.push(doom_layout::typedef_layout(name, fields, size));
    Ok(Some(layouts))
}

pub(super) fn parse_struct_definition(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
    pointer_typedefs: &[String],
) -> CompileResult<Option<Vec<StructLayout>>> {
    if token_has_keyword(tokens, Keyword::Typedef) {
        return Ok(None);
    }
    if !token_has_keyword(tokens, Keyword::Struct) && !token_has_keyword(tokens, Keyword::Union) {
        return Ok(None);
    }
    let Some(open_brace) = top_level_punctuator_index(tokens, "{") else {
        return Ok(None);
    };
    let is_union = tokens[..open_brace]
        .iter()
        .any(|token| token_is_keyword(token, Keyword::Union));
    let Some(close_brace) = matching_top_level_brace(tokens, open_brace) else {
        return Ok(None);
    };
    let Some(name) = aggregate_tag_name(tokens) else {
        return Ok(None);
    };
    let mut available_structs = known_structs.to_vec();
    let mut layouts = Vec::new();
    let mut context = StructParseContext {
        parent_name: &name,
        available_structs: &mut available_structs,
        constants,
        pointer_typedefs,
        nested_layouts: &mut layouts,
    };
    let Some((fields, size)) =
        parse_struct_fields(&tokens[open_brace + 1..close_brace], is_union, &mut context)?
    else {
        return Ok(None);
    };
    layouts.push(StructLayout { name, fields, size });
    Ok(Some(layouts))
}

pub(super) fn struct_forward_typedef_alias(tokens: &[Token]) -> Option<(String, String)> {
    if !token_has_keyword(tokens, Keyword::Typedef) || !token_has_keyword(tokens, Keyword::Struct) {
        return None;
    }
    if top_level_punctuator_index(tokens, "{").is_some() {
        return None;
    }
    let names = tokens
        .iter()
        .filter_map(token_identifier)
        .collect::<Vec<_>>();
    let [tag, alias] = names.as_slice() else {
        return None;
    };
    Some(((*tag).to_owned(), (*alias).to_owned()))
}

pub(super) fn struct_alias_layouts(
    layout: &StructLayout,
    aliases: &[(String, String)],
) -> Vec<StructLayout> {
    aliases
        .iter()
        .filter(|(tag, alias)| tag == &layout.name && alias != &layout.name)
        .map(|(_tag, alias)| StructLayout {
            name: alias.clone(),
            fields: layout.fields.clone(),
            size: layout.size,
        })
        .collect()
}

fn parse_struct_alias_typedef(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> Option<StructLayout> {
    let names = tokens
        .iter()
        .filter_map(token_identifier)
        .collect::<Vec<_>>();
    let [source, alias] = names.as_slice() else {
        return None;
    };
    known_structs
        .iter()
        .find(|layout| layout.name == *source)
        .map(|layout| StructLayout {
            name: (*alias).to_owned(),
            fields: layout.fields.clone(),
            size: layout.size,
        })
}

pub(super) fn aggregate_tag_name(tokens: &[Token]) -> Option<String> {
    let open_brace = top_level_punctuator_index(tokens, "{")?;
    let aggregate_index = tokens[..open_brace].iter().position(|token| {
        token_is_keyword(token, Keyword::Struct) || token_is_keyword(token, Keyword::Union)
    })?;
    tokens
        .get(aggregate_index + 1)
        .and_then(token_identifier)
        .map(ToOwned::to_owned)
}
