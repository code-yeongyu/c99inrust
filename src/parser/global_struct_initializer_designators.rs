use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_struct_initializer::parse_value;
use super::{
    Constant, FieldType, GlobalStructInitializerValue, Parser, StructLayout, struct_field_index,
    struct_field_path_designator,
};

pub(super) fn write_designator(
    values: &mut Vec<GlobalStructInitializerValue>,
    designator_parser: &Parser<'_>,
    known_structs: &[StructLayout],
    struct_name: &str,
    item: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<usize>> {
    if let Some((field_name, element_index, value_tokens)) =
        designator_parser.struct_array_field_designator(item)?
    {
        let index = struct_field_index(known_structs, struct_name, field_name)?;
        write_array_field_designator(
            values,
            known_structs,
            struct_name,
            index,
            element_index,
            value_tokens,
            constants,
        )?;
        return Ok(Some(index + 1));
    }
    if let Some((field_path, value_tokens)) = struct_field_path_designator(item)? {
        let index = write_field_path_designator(
            values,
            known_structs,
            struct_name,
            &field_path,
            value_tokens,
            constants,
        )?;
        return Ok(Some(index + 1));
    }
    Ok(None)
}

fn write_array_field_designator(
    values: &mut Vec<GlobalStructInitializerValue>,
    known_structs: &[StructLayout],
    struct_name: &str,
    field_index: usize,
    element_index: usize,
    value_tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<()> {
    let layout = known_structs
        .iter()
        .find(|layout| layout.name == struct_name)
        .ok_or_else(|| CompileError::new(format!("unknown struct type: {struct_name}")))?;
    let Some(FieldType::Array { .. }) = layout
        .fields
        .get(field_index)
        .map(|field| &field.field_type)
    else {
        return Err(CompileError::new(
            "global struct array field designator requires array field",
        ));
    };
    if values.len() <= field_index {
        values.resize(field_index + 1, GlobalStructInitializerValue::Integer(0));
    }
    if !matches!(values[field_index], GlobalStructInitializerValue::Nested(_)) {
        values[field_index] = GlobalStructInitializerValue::Nested(Vec::new());
    }
    let GlobalStructInitializerValue::Nested(nested_values) = &mut values[field_index] else {
        return Err(CompileError::new(
            "global struct array field designator requires nested field value",
        ));
    };
    if nested_values.len() <= element_index {
        nested_values.resize(element_index + 1, GlobalStructInitializerValue::Integer(0));
    }
    nested_values[element_index] = parse_value(value_tokens, known_structs, constants)?;
    Ok(())
}

fn write_field_path_designator(
    values: &mut Vec<GlobalStructInitializerValue>,
    known_structs: &[StructLayout],
    struct_name: &str,
    field_path: &[&str],
    value_tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<usize> {
    let Some(field_name) = field_path.first() else {
        return Err(CompileError::new(
            "expected nested global struct field designator",
        ));
    };
    let field_index = struct_field_index(known_structs, struct_name, field_name)?;
    write_nested_field_value(
        values,
        known_structs,
        struct_name,
        field_path,
        value_tokens,
        constants,
    )?;
    Ok(field_index)
}

fn write_nested_field_value(
    values: &mut Vec<GlobalStructInitializerValue>,
    known_structs: &[StructLayout],
    struct_name: &str,
    field_path: &[&str],
    value_tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<()> {
    let Some(field_name) = field_path.first() else {
        return Err(CompileError::new(
            "expected nested global struct field designator",
        ));
    };
    let layout = known_structs
        .iter()
        .find(|layout| layout.name == struct_name)
        .ok_or_else(|| CompileError::new(format!("unknown struct type: {struct_name}")))?;
    let field_index = struct_field_index(known_structs, struct_name, field_name)?;
    if values.len() <= field_index {
        values.resize(field_index + 1, GlobalStructInitializerValue::Integer(0));
    }
    if field_path.len() == 1 {
        values[field_index] = parse_value(value_tokens, known_structs, constants)?;
        return Ok(());
    }
    let Some(FieldType::Struct(nested_struct_name)) = layout
        .fields
        .get(field_index)
        .map(|field| &field.field_type)
    else {
        return Err(CompileError::new(
            "nested global struct field designator requires struct field",
        ));
    };
    if !matches!(values[field_index], GlobalStructInitializerValue::Nested(_)) {
        values[field_index] = GlobalStructInitializerValue::Nested(Vec::new());
    }
    let GlobalStructInitializerValue::Nested(nested_values) = &mut values[field_index] else {
        return Err(CompileError::new(
            "nested global struct field designator requires nested field value",
        ));
    };
    write_nested_field_value(
        nested_values,
        known_structs,
        nested_struct_name,
        &field_path[1..],
        value_tokens,
        constants,
    )
}
