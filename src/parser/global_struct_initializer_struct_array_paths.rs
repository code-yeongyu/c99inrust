use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_struct_initializer::parse_value;
use super::global_struct_initializer_designators::global_struct_layout;
use super::{Constant, FieldType, GlobalStructInitializerValue, StructLayout};

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
            "expected global struct-array element field path",
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
            "global struct-array element designator requires struct field",
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
            "global struct-array element designator requires nested field value",
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
