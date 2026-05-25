use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_array_compound_literals::{
    GlobalArrayCompoundLiteralBacking, global_array_compound_literal_initializer,
};
use super::global_byte_declarations::parse_unsigned_char_array_length;
use super::global_floatlike_declarations::{
    global_floatlike_scalar_type, parse_global_real_initializer,
};
use super::integer_initializer::parse_integer_initializer_with_constants;
use super::token_scan::{
    matching_top_level_brace, matching_top_level_bracket, previous_identifier_index,
    token_identifier, token_is_punctuator, top_level_punctuator_index,
};
use super::{Constant, Expr, Global, GlobalInitializer, Parser, ScalarType};

pub(super) fn parse_global_double_array(
    tokens: &[Token],
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
    let Some(scalar_type) = global_floatlike_scalar_type(&declaration[..name_index], false) else {
        return Ok(None);
    };
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global double-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global double-array name"))?
        .to_owned();
    let length_tokens = &declaration[open_bracket + 1..close_bracket];
    let initializer = if let Some(assign_index) =
        top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    {
        parse_global_double_array_initializer(
            &declaration[close_bracket + assign_index + 2..],
            scalar_type,
            length_tokens,
            constants,
        )?
    } else {
        let length = parse_unsigned_char_array_length(length_tokens, constants)?;
        if scalar_type == ScalarType::Double {
            GlobalInitializer::DoubleArray { length }
        } else {
            GlobalInitializer::ScalarArray {
                scalar_type,
                length,
            }
        }
    };
    Ok(Some(Global::new(name, initializer)))
}

fn parse_global_double_array_initializer(
    tokens: &[Token],
    scalar_type: ScalarType,
    length_tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    if tokens
        .first()
        .is_some_and(|token| token_is_punctuator(token, "{"))
    {
        let values = parse_global_real_array_initializer(tokens, constants)?;
        let length = global_double_array_length(length_tokens, values.len(), constants)?;
        if values.len() > length {
            return Err(CompileError::new(
                "too many global scalar-array initializers",
            ));
        }
        return Ok(GlobalInitializer::ScalarArrayValues {
            scalar_type,
            length,
            values,
        });
    }
    parse_global_compound_array_initializer(tokens, scalar_type, length_tokens, constants)
}

fn global_double_array_length(
    length_tokens: &[Token],
    inferred_length: usize,
    constants: &[Constant],
) -> CompileResult<usize> {
    if length_tokens.is_empty() {
        Ok(inferred_length)
    } else {
        parse_unsigned_char_array_length(length_tokens, constants)
    }
}

fn parse_global_compound_array_initializer(
    tokens: &[Token],
    scalar_type: ScalarType,
    length_tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs: &[],
        known_constants: constants,
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
        known_function_pointer_typedefs: &[],
    };
    let expr = parser.expression()?;
    if parser.peek().is_some() {
        return Err(CompileError::new(
            "unsupported global scalar-array initializer",
        ));
    }
    let Expr::ArrayCompoundLiteral {
        element_type,
        element_byte_size,
        element_unsigned,
        length,
        values,
    } = expr
    else {
        return Err(CompileError::new(
            "expected global scalar-array initializer",
        ));
    };
    if element_type != scalar_type {
        return Err(CompileError::new(
            "global scalar-array compound literal type mismatch",
        ));
    }
    let declared_length = global_double_array_length(length_tokens, length, constants)?;
    if declared_length != length {
        return Err(CompileError::new(
            "global scalar-array compound literal length mismatch",
        ));
    }
    global_array_compound_literal_initializer(
        GlobalArrayCompoundLiteralBacking {
            element_type,
            element_byte_size,
            element_unsigned,
            length,
        },
        &values,
        constants,
    )
}

fn parse_global_real_array_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Vec<String>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new(
            "expected global scalar-array initializer",
        ));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global scalar-array initializer")
                .at(first.line, first.column),
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global scalar-array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global scalar-array initializer")
                .at(token.line, token.column),
        );
    }

    let mut values = Vec::new();
    let mut start = 1usize;
    let mut next_index = 0usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global scalar-array initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        let item = &item[..item_len];
        let (index, value_tokens) = if token_is_punctuator(&item[0], "[") {
            let close = matching_top_level_bracket(item, 0)
                .ok_or_else(|| CompileError::new("unterminated global scalar-array designator"))?;
            if !item
                .get(close + 1)
                .is_some_and(|token| token_is_punctuator(token, "="))
            {
                return Err(CompileError::new(
                    "expected global scalar-array designator assignment",
                ));
            }
            let index = parse_integer_initializer_with_constants(&item[1..close], constants)?;
            let index = usize::try_from(index)
                .map_err(|_| CompileError::new("global scalar-array designator is negative"))?;
            next_index = index
                .checked_add(1)
                .ok_or_else(|| CompileError::new("global scalar-array designator is too large"))?;
            (index, &item[close + 2..])
        } else {
            let index = next_index;
            next_index = next_index
                .checked_add(1)
                .ok_or_else(|| CompileError::new("too many global scalar-array initializers"))?;
            (index, item)
        };
        if values.len() <= index {
            values.resize(index + 1, "0".to_owned());
        }
        values[index] = parse_global_real_initializer(value_tokens, constants, &[])?;
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    Ok(values)
}
