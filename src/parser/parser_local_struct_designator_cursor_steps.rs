use super::parser_local_struct_designator_cursor::{
    LocalStructDesignatorCursor, LocalStructDesignatorWrite,
};
use super::{CompileError, CompileResult, FieldType, Parser, StructLayout, field_type_at};

pub(super) fn next_local_array_field_cursor(
    layout: &StructLayout,
    field_index: usize,
    element_index: usize,
) -> CompileResult<LocalStructDesignatorWrite> {
    let Some(FieldType::Array { length, .. }) = field_type_at(layout, field_index) else {
        return Err(CompileError::new(
            "struct array field designator requires array field",
        ));
    };
    let next_element = element_index
        .checked_add(1)
        .ok_or_else(|| CompileError::new("array field designator index overflow"))?;
    let cursor = if next_element < *length {
        Some(LocalStructDesignatorCursor::ArrayField {
            field_index,
            element_index: next_element,
        })
    } else {
        None
    };
    Ok(LocalStructDesignatorWrite {
        next_index: field_index + 1,
        cursor,
    })
}

pub(super) fn next_local_struct_array_path_cursor(
    parser: &Parser<'_>,
    struct_name: &str,
    index_path: &[usize],
    element_index: usize,
) -> CompileResult<LocalStructDesignatorWrite> {
    let Some(field_index) = index_path.last().copied() else {
        return Err(CompileError::new(
            "expected nested struct array field designator",
        ));
    };
    let parent_struct_name = parser.local_parent_struct_name(struct_name, index_path)?;
    let parent_layout = parser.local_struct_layout(&parent_struct_name)?;
    let Some(FieldType::Array { length, .. }) = field_type_at(parent_layout, field_index) else {
        return Err(CompileError::new(
            "nested struct array field designator requires array field",
        ));
    };
    let next_element = element_index
        .checked_add(1)
        .ok_or_else(|| CompileError::new("nested array field designator index overflow"))?;
    if next_element < *length {
        return Ok(LocalStructDesignatorWrite {
            next_index: index_path[0] + 1,
            cursor: Some(LocalStructDesignatorCursor::ArrayPath {
                index_path: index_path.to_vec(),
                element_index: next_element,
            }),
        });
    }
    next_local_struct_field_cursor(parser, struct_name, index_path)
}

pub(super) fn next_local_struct_field_cursor(
    parser: &Parser<'_>,
    struct_name: &str,
    index_path: &[usize],
) -> CompileResult<LocalStructDesignatorWrite> {
    let next_path = next_local_struct_field_path(parser, struct_name, index_path)?;
    Ok(match next_path {
        Some(path) if path.len() > 1 => LocalStructDesignatorWrite {
            next_index: path[0] + 1,
            cursor: Some(LocalStructDesignatorCursor::FieldPath(path)),
        },
        Some(path) => LocalStructDesignatorWrite {
            next_index: path[0],
            cursor: None,
        },
        None => LocalStructDesignatorWrite {
            next_index: parser.local_struct_layout(struct_name)?.fields.len(),
            cursor: None,
        },
    })
}

pub(super) fn next_local_struct_field_path(
    parser: &Parser<'_>,
    struct_name: &str,
    index_path: &[usize],
) -> CompileResult<Option<Vec<usize>>> {
    let mut path = index_path.to_vec();
    while let Some(last_index) = path.last().copied() {
        let parent_struct_name = parser.local_parent_struct_name(struct_name, &path)?;
        let parent_layout = parser.local_struct_layout(&parent_struct_name)?;
        if last_index + 1 < parent_layout.fields.len() {
            let Some(last_path_index) = path.last_mut() else {
                return Err(CompileError::new("expected nested struct field designator"));
            };
            *last_path_index = last_index + 1;
            return Ok(Some(path));
        }
        path.pop();
    }
    Ok(None)
}
