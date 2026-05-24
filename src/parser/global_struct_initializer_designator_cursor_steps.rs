use crate::diagnostics::{CompileError, CompileResult};

use super::global_struct_initializer_designator_cursor::{
    GlobalStructDesignatorCursor, GlobalStructDesignatorWrite,
};
use super::global_struct_initializer_designator_paths::parent_struct_name;
use super::global_struct_initializer_designators::global_struct_layout;
use super::{FieldType, StructLayout};

pub(super) fn next_array_field_cursor(
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
    let next_element = element_index
        .checked_add(1)
        .ok_or_else(|| CompileError::new("global array field designator index overflow"))?;
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

pub(super) fn next_struct_array_path_cursor(
    known_structs: &[StructLayout],
    struct_name: &str,
    index_path: &[usize],
    element_index: usize,
) -> CompileResult<GlobalStructDesignatorWrite> {
    let Some(field_index) = index_path.last().copied() else {
        return Err(CompileError::new(
            "expected nested global struct array field designator",
        ));
    };
    let parent_struct_name = parent_struct_name(known_structs, struct_name, index_path)?;
    let parent_layout = global_struct_layout(known_structs, &parent_struct_name)?;
    let Some(FieldType::Array { length, .. }) = parent_layout
        .fields
        .get(field_index)
        .map(|field| &field.field_type)
    else {
        return Err(CompileError::new(
            "nested global struct array field designator requires array field",
        ));
    };
    let next_element = element_index
        .checked_add(1)
        .ok_or_else(|| CompileError::new("nested global array field designator index overflow"))?;
    if next_element < *length {
        return Ok(GlobalStructDesignatorWrite {
            next_index: index_path[0] + 1,
            cursor: Some(GlobalStructDesignatorCursor::ArrayPath {
                index_path: index_path.to_vec(),
                element_index: next_element,
            }),
        });
    }
    next_struct_field_cursor(known_structs, struct_name, index_path)
}

pub(super) fn next_struct_field_cursor(
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

pub(super) fn next_struct_field_path(
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
