use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_struct_initializer_designators::{
    global_struct_layout, write_array_field_designator, write_index_path_value,
};
use super::{
    Constant, FieldType, GlobalStructInitializerValue, Parser, StructLayout, struct_field_index,
    struct_field_path_designator,
};

pub(super) struct GlobalStructDesignatorWrite {
    pub(super) next_index: usize,
    pub(super) cursor: Option<GlobalStructDesignatorCursor>,
}

pub(super) enum GlobalStructDesignatorCursor {
    ArrayField {
        field_index: usize,
        element_index: usize,
    },
    FieldPath(Vec<usize>),
}

pub(super) fn write_designator(
    values: &mut Vec<GlobalStructInitializerValue>,
    designator_parser: &Parser<'_>,
    known_structs: &[StructLayout],
    struct_name: &str,
    item: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<GlobalStructDesignatorWrite>> {
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
        return next_array_field_cursor(known_structs, struct_name, index, element_index).map(Some);
    }
    if let Some((field_path, value_tokens)) = struct_field_path_designator(item)? {
        let index_path = global_struct_field_index_path(
            known_structs,
            struct_name,
            field_path[0],
            &field_path[1..],
        )?;
        write_index_path_value(
            values,
            known_structs,
            struct_name,
            &index_path,
            value_tokens,
            constants,
        )?;
        return next_struct_field_cursor(known_structs, struct_name, &index_path).map(Some);
    }
    Ok(None)
}

pub(super) fn write_cursor_value(
    values: &mut Vec<GlobalStructInitializerValue>,
    known_structs: &[StructLayout],
    struct_name: &str,
    cursor: GlobalStructDesignatorCursor,
    value_tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<GlobalStructDesignatorWrite> {
    match cursor {
        GlobalStructDesignatorCursor::ArrayField {
            field_index,
            element_index,
        } => {
            write_array_field_designator(
                values,
                known_structs,
                struct_name,
                field_index,
                element_index,
                value_tokens,
                constants,
            )?;
            next_array_field_cursor(known_structs, struct_name, field_index, element_index)
        }
        GlobalStructDesignatorCursor::FieldPath(index_path) => {
            write_index_path_value(
                values,
                known_structs,
                struct_name,
                &index_path,
                value_tokens,
                constants,
            )?;
            next_struct_field_cursor(known_structs, struct_name, &index_path)
        }
    }
}

fn next_array_field_cursor(
    known_structs: &[StructLayout],
    struct_name: &str,
    field_index: usize,
    element_index: usize,
) -> CompileResult<GlobalStructDesignatorWrite> {
    let layout = global_struct_layout(known_structs, struct_name)?;
    let Some(FieldType::Array { length, .. }) = layout
        .fields
        .get(field_index)
        .map(|field| &field.field_type)
    else {
        return Err(CompileError::new(
            "global struct array field designator requires array field",
        ));
    };
    let next_element = element_index + 1;
    let cursor = if next_element < *length {
        Some(GlobalStructDesignatorCursor::ArrayField {
            field_index,
            element_index: next_element,
        })
    } else {
        None
    };
    Ok(GlobalStructDesignatorWrite {
        next_index: field_index + 1,
        cursor,
    })
}

fn next_struct_field_cursor(
    known_structs: &[StructLayout],
    struct_name: &str,
    index_path: &[usize],
) -> CompileResult<GlobalStructDesignatorWrite> {
    let next_path = next_struct_field_path(known_structs, struct_name, index_path)?;
    Ok(match next_path {
        Some(path) if path.len() > 1 => GlobalStructDesignatorWrite {
            next_index: path[0] + 1,
            cursor: Some(GlobalStructDesignatorCursor::FieldPath(path)),
        },
        Some(path) => GlobalStructDesignatorWrite {
            next_index: path[0],
            cursor: None,
        },
        None => GlobalStructDesignatorWrite {
            next_index: global_struct_layout(known_structs, struct_name)?
                .fields
                .len(),
            cursor: None,
        },
    })
}

fn next_struct_field_path(
    known_structs: &[StructLayout],
    struct_name: &str,
    index_path: &[usize],
) -> CompileResult<Option<Vec<usize>>> {
    let mut path = index_path.to_vec();
    while let Some(last_index) = path.last().copied() {
        let parent_struct_name = parent_struct_name(known_structs, struct_name, &path)?;
        let parent_layout = global_struct_layout(known_structs, &parent_struct_name)?;
        if last_index + 1 < parent_layout.fields.len() {
            let Some(last_path_index) = path.last_mut() else {
                return Err(CompileError::new(
                    "expected nested global struct field designator",
                ));
            };
            *last_path_index = last_index + 1;
            return Ok(Some(path));
        }
        path.pop();
    }
    Ok(None)
}

fn parent_struct_name(
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

fn global_struct_field_index_path(
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
