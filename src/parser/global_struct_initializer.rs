use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_struct_initializer_addresses::{address_from_expr, string_pointer_from_expr};
use super::global_struct_initializer_designators::write_array_field_designator;
use super::token_scan::{matching_top_level_brace, token_is_punctuator, top_level_comma_ranges};
use super::{
    Constant, Expr, GlobalStructInitializerValue, Parser, StructLayout,
    eval_integer_initializer_expr_with_constants, struct_field_designator, struct_field_index,
};

pub(super) fn parse(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Vec<Vec<GlobalStructInitializerValue>>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new(
            "expected global struct-array initializer",
        ));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global struct-array initializer")
                .at(first.line, first.column),
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global struct-array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global struct-array initializer")
                .at(token.line, token.column),
        );
    }

    let mut rows = Vec::new();
    for (start, end) in top_level_comma_ranges(&tokens[1..close_brace]) {
        if start == end {
            continue;
        }
        let row = &tokens[(start + 1)..=end];
        rows.push(parse_row(row, known_structs, constants)?);
    }
    Ok(rows)
}

pub(super) fn parse_object(
    tokens: &[Token],
    struct_name: &str,
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Vec<GlobalStructInitializerValue>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global struct initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global struct initializer").at(first.line, first.column)
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(CompileError::new("unterminated global struct initializer")
            .at(first.line, first.column));
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global struct initializer").at(token.line, token.column)
        );
    }

    parse_values_for_struct(
        &tokens[1..close_brace],
        struct_name,
        known_structs,
        constants,
    )
}

fn parse_row(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Vec<GlobalStructInitializerValue>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global struct initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global struct initializer").at(first.line, first.column)
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(CompileError::new("unterminated global struct initializer")
            .at(first.line, first.column));
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global struct initializer").at(token.line, token.column)
        );
    }

    parse_values(&tokens[1..close_brace], known_structs, constants)
}

fn parse_values(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Vec<GlobalStructInitializerValue>> {
    let mut values = Vec::new();
    for (start, end) in top_level_comma_ranges(tokens) {
        if start == end {
            continue;
        }
        values.push(parse_value(&tokens[start..end], known_structs, constants)?);
    }
    Ok(values)
}

fn parse_values_for_struct(
    tokens: &[Token],
    struct_name: &str,
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Vec<GlobalStructInitializerValue>> {
    let mut values = Vec::new();
    let mut next_index = 0usize;
    let designator_parser = Parser {
        tokens: &[],
        index: 0,
        known_structs,
        known_constants: constants,
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
        known_function_pointer_typedefs: &[],
    };
    for (start, end) in top_level_comma_ranges(tokens) {
        if start == end {
            continue;
        }
        let item = &tokens[start..end];
        if let Some((field_name, element_index, value_tokens)) =
            designator_parser.struct_array_field_designator(item)?
        {
            let index = struct_field_index(known_structs, struct_name, field_name)?;
            next_index = index + 1;
            write_array_field_designator(
                &mut values,
                known_structs,
                struct_name,
                index,
                element_index,
                value_tokens,
                constants,
            )?;
            continue;
        }
        let (index, value_tokens) =
            if let Some((field_name, value_tokens)) = struct_field_designator(item)? {
                let index = struct_field_index(known_structs, struct_name, field_name)?;
                next_index = index + 1;
                (index, value_tokens)
            } else {
                let index = next_index;
                next_index += 1;
                (index, item)
            };
        if values.len() <= index {
            values.resize(index + 1, GlobalStructInitializerValue::Integer(0));
        }
        values[index] = parse_value(value_tokens, known_structs, constants)?;
    }
    Ok(values)
}

pub(super) fn parse_value(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<GlobalStructInitializerValue> {
    if token_is_punctuator(&tokens[0], "{") {
        let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
            return Err(CompileError::new(
                "unterminated global struct initializer value",
            ));
        };
        if close_brace + 1 == tokens.len() {
            let values = parse_values(&tokens[1..close_brace], known_structs, constants)?;
            return Ok(GlobalStructInitializerValue::Nested(values));
        }
    }
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
    if let Some(token) = parser.peek() {
        return Err(
            CompileError::new("unsupported global struct initializer value")
                .at(token.line, token.column),
        );
    }
    value_from_expr(&expr, constants)
}

fn value_from_expr(
    expr: &Expr,
    constants: &[Constant],
) -> CompileResult<GlobalStructInitializerValue> {
    match expr {
        Expr::StringLiteral(value) => Ok(GlobalStructInitializerValue::String(value.clone())),
        _ => string_pointer_from_expr(expr, constants)?.map_or_else(
            || {
                address_from_expr(expr, constants)?.map_or_else(
                    || {
                        eval_integer_initializer_expr_with_constants(expr, constants)
                            .and_then(super::InitializerNumber::to_i64_trunc)
                            .map(GlobalStructInitializerValue::Integer)
                    },
                    |address| Ok(GlobalStructInitializerValue::Address(address)),
                )
            },
            |(value, byte_offset, cast_target)| {
                Ok(GlobalStructInitializerValue::StringPointer {
                    value,
                    byte_offset,
                    cast_target,
                })
            },
        ),
    }
}
