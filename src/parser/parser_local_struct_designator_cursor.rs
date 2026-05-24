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
        let next_element = element_index + 1;
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

    fn next_local_struct_field_cursor(
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

    fn next_local_struct_field_path(
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

    fn local_parent_struct_name(
        &self,
        struct_name: &str,
        index_path: &[usize],
    ) -> CompileResult<String> {
        let mut current = struct_name.to_owned();
        for index in &index_path[..index_path.len().saturating_sub(1)] {
            let layout = self.local_struct_layout(&current)?;
            let Some(FieldType::Struct(next)) = field_type_at(layout, *index) else {
                return Err(CompileError::new(
                    "nested struct field designator requires struct field",
                ));
            };
            current = next.clone();
        }
        Ok(current)
    }

    fn local_struct_field_index_path(
        &self,
        layout: &StructLayout,
        field_index: usize,
        field_path: &[&str],
    ) -> CompileResult<Vec<usize>> {
        let mut index_path = vec![field_index];
        let mut current_struct = match field_type_at(layout, field_index) {
            Some(FieldType::Struct(struct_name)) => struct_name.clone(),
            Some(_) if field_path.is_empty() => return Ok(index_path),
            _ => {
                return Err(CompileError::new(
                    "nested struct field designator requires struct field",
                ));
            }
        };
        for (position, field_name) in field_path.iter().enumerate() {
            let index = struct_field_index(self.known_structs, &current_struct, field_name)?;
            index_path.push(index);
            if position + 1 < field_path.len() {
                let layout = self.local_struct_layout(&current_struct)?;
                let Some(FieldType::Struct(next_struct)) = field_type_at(layout, index) else {
                    return Err(CompileError::new(
                        "nested struct field designator requires struct field",
                    ));
                };
                current_struct = next_struct.clone();
            }
        }
        Ok(index_path)
    }

    fn write_local_struct_index_path_value(
        &self,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        index_path: &[usize],
        value_tokens: &[Token],
    ) -> CompileResult<()> {
        let Some((field_index, nested_path)) = index_path.split_first() else {
            return Err(CompileError::new("expected nested struct field designator"));
        };
        let layout = self.local_struct_layout(struct_name)?;
        resize_values_for_index(values, layout, *field_index);
        let Some(field_type) = field_type_at(layout, *field_index) else {
            return Err(CompileError::new("unknown nested struct field designator"));
        };
        if nested_path.is_empty() {
            values[*field_index] = self.parse_local_struct_field_value(field_type, value_tokens)?;
            return Ok(());
        }
        let FieldType::Struct(nested_struct_name) = field_type else {
            return Err(CompileError::new(
                "nested struct field designator requires struct field",
            ));
        };
        let LocalStructInitializerValue::Nested(nested_values) = &mut values[*field_index] else {
            return Err(CompileError::new(
                "nested struct field designator requires nested field value",
            ));
        };
        self.write_local_struct_index_path_value(
            nested_struct_name,
            nested_values,
            nested_path,
            value_tokens,
        )
    }
}
