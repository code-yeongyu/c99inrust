use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_struct_initializer::parse_value;
use super::{Constant, FieldType, GlobalStructInitializerValue, StructLayout};

pub(super) fn write_array_field_designator(
    values: &mut Vec<GlobalStructInitializerValue>,
    known_structs: &[StructLayout],
    struct_name: &str,
    field_index: usize,
    element_index: usize,
    value_tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<()> {
    let layout = global_struct_layout(known_structs, struct_name)?;
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

pub(super) fn write_index_path_value(
    values: &mut Vec<GlobalStructInitializerValue>,
    known_structs: &[StructLayout],
    struct_name: &str,
    index_path: &[usize],
    value_tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<()> {
    let Some((field_index, nested_path)) = index_path.split_first() else {
        return Err(CompileError::new(
            "expected nested global struct field designator",
        ));
    };
    let layout = global_struct_layout(known_structs, struct_name)?;
    if values.len() <= *field_index {
        values.resize(*field_index + 1, GlobalStructInitializerValue::Integer(0));
    }
    if nested_path.is_empty() {
        values[*field_index] = parse_value(value_tokens, known_structs, constants)?;
        return Ok(());
    }
    let Some(FieldType::Struct(nested_struct_name)) = layout
        .fields
        .get(*field_index)
        .map(|field| &field.field_type)
    else {
        return Err(CompileError::new(
            "nested global struct field designator requires struct field",
        ));
    };
    if !matches!(
        values[*field_index],
        GlobalStructInitializerValue::Nested(_)
    ) {
        values[*field_index] = GlobalStructInitializerValue::Nested(Vec::new());
    }
    let GlobalStructInitializerValue::Nested(nested_values) = &mut values[*field_index] else {
        return Err(CompileError::new(
            "nested global struct field designator requires nested field value",
        ));
    };
    write_index_path_value(
        nested_values,
        known_structs,
        nested_struct_name,
        nested_path,
        value_tokens,
        constants,
    )
}

pub(super) fn global_struct_layout<'a>(
    known_structs: &'a [StructLayout],
    struct_name: &str,
) -> CompileResult<&'a StructLayout> {
    known_structs
        .iter()
        .find(|layout| layout.name == struct_name)
        .ok_or_else(|| CompileError::new(format!("unknown struct type: {struct_name}")))
}
