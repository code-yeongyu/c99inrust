use crate::diagnostics::{CompileError, CompileResult};

use super::global_struct_initializer_designators::global_struct_layout;
use super::{FieldType, StructLayout, struct_field_index};

pub(super) fn parent_struct_name(
    known_structs: &[StructLayout],
    struct_name: &str,
    index_path: &[usize],
) -> CompileResult<String> {
    let mut current = struct_name.to_owned();
    for index in &index_path[..index_path.len().saturating_sub(1)] {
        let layout = global_struct_layout(known_structs, &current)?;
        let Some(FieldType::Struct(next)) =
            layout.fields.get(*index).map(|field| &field.field_type)
        else {
            return Err(CompileError::new(
                "nested global struct field designator requires struct field",
            ));
        };
        current = next.clone();
    }
    Ok(current)
}

pub(super) fn global_struct_field_index_path(
    known_structs: &[StructLayout],
    struct_name: &str,
    field_name: &str,
    field_path: &[&str],
) -> CompileResult<Vec<usize>> {
    let root_index = struct_field_index(known_structs, struct_name, field_name)?;
    let root_layout = global_struct_layout(known_structs, struct_name)?;
    let mut index_path = vec![root_index];
    let mut current_struct = match root_layout
        .fields
        .get(root_index)
        .map(|field| &field.field_type)
    {
        Some(FieldType::Struct(struct_name)) => struct_name.clone(),
        Some(_) if field_path.is_empty() => return Ok(index_path),
        _ => {
            return Err(CompileError::new(
                "nested global struct field designator requires struct field",
            ));
        }
    };
    for (position, nested_field_name) in field_path.iter().enumerate() {
        let index = struct_field_index(known_structs, &current_struct, nested_field_name)?;
        index_path.push(index);
        if position + 1 < field_path.len() {
            let layout = global_struct_layout(known_structs, &current_struct)?;
            let Some(FieldType::Struct(next_struct)) =
                layout.fields.get(index).map(|field| &field.field_type)
            else {
                return Err(CompileError::new(
                    "nested global struct field designator requires struct field",
                ));
            };
            current_struct = next_struct.clone();
        }
    }
    Ok(index_path)
}
