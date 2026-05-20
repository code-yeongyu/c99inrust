use crate::diagnostics::{CompileError, CompileResult};

use super::struct_layout_helpers::{
    StructFieldOutput, align_struct_offset, field_type_alignment, field_type_size,
};
use super::{FieldType, StructField, StructLayout};

pub(super) fn push_anonymous_aggregate_fields(
    layout: &StructLayout,
    is_parent_union: bool,
    known_structs: &[StructLayout],
    output: &mut StructFieldOutput<'_>,
) -> CompileResult<()> {
    let field_type = FieldType::Struct(layout.name.clone());
    let size = field_type_size(&field_type, known_structs)?;
    let alignment = field_type_alignment(&field_type, known_structs)?;
    *output.max_alignment = (*output.max_alignment).max(alignment);
    let base_offset = if is_parent_union {
        *output.offset = (*output.offset).max(size);
        0
    } else {
        *output.offset = align_struct_offset(*output.offset, alignment)?;
        let base_offset = *output.offset;
        *output.offset = (*output.offset)
            .checked_add(size)
            .ok_or_else(|| CompileError::new("struct size overflow"))?;
        base_offset
    };
    for field in &layout.fields {
        let offset = base_offset
            .checked_add(field.offset)
            .ok_or_else(|| CompileError::new("struct member offset overflow"))?;
        output.fields.push(StructField {
            name: field.name.clone(),
            field_type: field.field_type.clone(),
            offset,
        });
    }
    Ok(())
}
