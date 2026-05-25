use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token};

use super::global_pointer_compound_literals::global_struct_value;
use super::global_specifiers::global_struct_specifier_name;
use super::token_scan::{
    previous_identifier_index, token_has_keyword, token_identifier, token_is_punctuator,
    top_level_punctuator_index,
};
use super::{
    Constant, Expr, Global, GlobalInitializer, GlobalStructInitializerValue, Parser, StructLayout,
    global_struct_initializer,
};

pub(super) fn parse(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let end_index = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    if top_level_punctuator_index(&declaration[..end_index], "[").is_some() {
        return Ok(None);
    }
    if token_has_keyword(declaration, Keyword::Extern) {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, end_index) else {
        return Ok(None);
    };
    let Some(struct_name) = global_struct_specifier_name(&declaration[..name_index], known_structs)
    else {
        return Ok(None);
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global struct object name"))?
        .to_owned();
    let Some(values) = initializer_values(
        declaration,
        end_index,
        &struct_name,
        known_structs,
        constants,
    )?
    else {
        return Ok(None);
    };
    Ok(Some(Global::new(
        name,
        GlobalInitializer::StructObject {
            struct_name,
            values,
        },
    )))
}

fn initializer_values(
    declaration: &[Token],
    end_index: usize,
    struct_name: &str,
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Option<Vec<GlobalStructInitializerValue>>> {
    if end_index == declaration.len() {
        return Ok(Some(Vec::new()));
    }
    let initializer = &declaration[end_index + 1..];
    if initializer
        .first()
        .is_some_and(|token| token_is_punctuator(token, "{"))
    {
        return global_struct_initializer::parse_object(
            initializer,
            struct_name,
            known_structs,
            constants,
        )
        .map(Some);
    }
    compound_literal_initializer_values(initializer, struct_name, known_structs, constants)
}

fn compound_literal_initializer_values(
    tokens: &[Token],
    struct_name: &str,
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Option<Vec<GlobalStructInitializerValue>>> {
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs,
        known_constants: constants,
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
        known_function_pointer_typedefs: &[],
    };
    let expr = parser.expression()?;
    if parser.peek().is_some() {
        return Ok(None);
    }
    let Expr::StructCompoundLiteral {
        struct_name: literal_struct_name,
        values,
    } = expr
    else {
        return Ok(None);
    };
    if literal_struct_name != struct_name {
        return Err(CompileError::new(
            "global struct compound literal type mismatch",
        ));
    }
    values
        .iter()
        .map(|value| global_struct_value(value, constants))
        .collect::<CompileResult<Vec<_>>>()
        .map(Some)
}
