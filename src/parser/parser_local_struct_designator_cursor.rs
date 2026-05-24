use crate::front_end::lexer::Token;

use super::{
    CompileError, CompileResult, FieldType, LocalStructInitializerValue, Parser, StructLayout,
    field_type_at, resize_values_for_index, struct_field_index, struct_field_path_designator,
};

pub(super) struct LocalStructDesignatorWrite {
    pub(super) next_index: usize,
    pub(super) cursor: Option<LocalStructDesignatorCursor>,
}

pub(super) enum LocalStructDesignatorCursor {
    ArrayField {
        field_index: usize,
        element_index: usize,
    },
    ArrayPath {
        index_path: Vec<usize>,
        element_index: usize,
    },
    StructArrayFieldPath {
        array_path: Vec<usize>,
        element_index: usize,
        field_path: Vec<usize>,
    },
    FieldPath(Vec<usize>),
}

impl Parser<'_> {
    pub(super) fn write_local_struct_designator_item(
        &self,
        layout: &StructLayout,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        item: &[Token],
    ) -> CompileResult<Option<LocalStructDesignatorWrite>> {
        if let Some((field_name, element_index, value_tokens)) =
            self.struct_array_field_designator(item)?
        {
            let index = struct_field_index(self.known_structs, struct_name, field_name)?;
            resize_values_for_index(values, layout, index);
            self.write_local_array_field_designator(
                layout,
                values,
                index,
                element_index,
                value_tokens,
            )?;
            return Self::next_local_array_field_cursor(layout, index, element_index).map(Some);
        }
        if let Some(write) = self.write_local_struct_array_element_designator_item(
            layout,
            struct_name,
            values,
            item,
        )? {
            return Ok(Some(write));
        }
        if let Some((field_path, element_index, value_tokens)) =
            self.struct_array_field_path_designator(item)?
        {
            let index = struct_field_index(self.known_structs, struct_name, field_path[0])?;
            let index_path = self.local_struct_field_index_path(layout, index, &field_path[1..])?;
            self.write_local_struct_array_index_path_value(
                struct_name,
                values,
                &index_path,
                element_index,
                value_tokens,
            )?;
            return self
                .next_local_struct_array_path_cursor(struct_name, &index_path, element_index)
                .map(Some);
        }
        if let Some((field_path, value_tokens)) = struct_field_path_designator(item)? {
            let index = struct_field_index(self.known_structs, struct_name, field_path[0])?;
            let index_path = self.local_struct_field_index_path(layout, index, &field_path[1..])?;
            self.write_local_struct_index_path_value(
                struct_name,
                values,
                &index_path,
                value_tokens,
            )?;
            return self
                .next_local_struct_field_cursor(struct_name, &index_path)
                .map(Some);
        }
        Ok(None)
    }

    pub(super) fn write_local_struct_cursor_value(
        &self,
        layout: &StructLayout,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        cursor: LocalStructDesignatorCursor,
        value_tokens: &[Token],
    ) -> CompileResult<LocalStructDesignatorWrite> {
        match cursor {
            LocalStructDesignatorCursor::ArrayField {
                field_index,
                element_index,
            } => {
                resize_values_for_index(values, layout, field_index);
                self.write_local_array_field_designator(
                    layout,
                    values,
                    field_index,
                    element_index,
                    value_tokens,
                )?;
                Self::next_local_array_field_cursor(layout, field_index, element_index)
            }
            LocalStructDesignatorCursor::ArrayPath {
                index_path,
                element_index,
            } => {
                self.write_local_struct_array_index_path_value(
                    struct_name,
                    values,
                    &index_path,
                    element_index,
                    value_tokens,
                )?;
                self.next_local_struct_array_path_cursor(struct_name, &index_path, element_index)
            }
            LocalStructDesignatorCursor::StructArrayFieldPath {
                array_path,
                element_index,
                field_path,
            } => self.write_local_struct_array_element_cursor_value(
                struct_name,
                values,
                &array_path,
                element_index,
                &field_path,
                value_tokens,
            ),
            LocalStructDesignatorCursor::FieldPath(index_path) => {
                self.write_local_struct_index_path_value(
                    struct_name,
                    values,
                    &index_path,
                    value_tokens,
                )?;
                self.next_local_struct_field_cursor(struct_name, &index_path)
            }
        }
    }

    fn next_local_array_field_cursor(
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

    fn next_local_struct_array_path_cursor(
        &self,
        struct_name: &str,
        index_path: &[usize],
        element_index: usize,
    ) -> CompileResult<LocalStructDesignatorWrite> {
        let Some(field_index) = index_path.last().copied() else {
            return Err(CompileError::new(
                "expected nested struct array field designator",
            ));
        };
        let parent_struct_name = self.local_parent_struct_name(struct_name, index_path)?;
        let parent_layout = self.local_struct_layout(&parent_struct_name)?;
        let Some(FieldType::Array { length, .. }) = field_type_at(parent_layout, field_index)
        else {
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
        self.next_local_struct_field_cursor(struct_name, index_path)
    }

    pub(super) fn next_local_struct_field_cursor(
        &self,
        struct_name: &str,
        index_path: &[usize],
    ) -> CompileResult<LocalStructDesignatorWrite> {
        let next_path = self.next_local_struct_field_path(struct_name, index_path)?;
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
                next_index: self.local_struct_layout(struct_name)?.fields.len(),
                cursor: None,
            },
        })
    }

    pub(super) fn next_local_struct_field_path(
        &self,
        struct_name: &str,
        index_path: &[usize],
    ) -> CompileResult<Option<Vec<usize>>> {
        let mut path = index_path.to_vec();
        while let Some(last_index) = path.last().copied() {
            let parent_struct_name = self.local_parent_struct_name(struct_name, &path)?;
            let parent_layout = self.local_struct_layout(&parent_struct_name)?;
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
}
