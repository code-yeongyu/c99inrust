use crate::diagnostics::CompileResult;
use crate::front_end::lexer::{Keyword, Token};

use super::external_declarations::function_pointer_name;
use super::struct_bitfields::{BitFieldState, bit_field_width, push_bit_field};
use super::struct_layout_helpers::{
    StructFieldOutput, align_struct_offset, declarator_field_type, declarator_name_index,
    field_type_alignment, field_type_size, push_struct_field, struct_field_type,
};
use super::token_scan::{
    matching_top_level_brace, token_identifier, token_is_keyword, token_is_punctuator,
    top_level_comma_ranges, top_level_punctuator_index, update_depths,
};
use super::{Constant, FieldType, StructField, StructLayout};

pub(super) struct StructParseContext<'a> {
    pub(super) parent_name: &'a str,
    pub(super) available_structs: &'a mut Vec<StructLayout>,
    pub(super) constants: &'a [Constant],
    pub(super) pointer_typedefs: &'a [String],
    pub(super) nested_layouts: &'a mut Vec<StructLayout>,
}

pub(super) fn parse_struct_fields(
    tokens: &[Token],
    is_union: bool,
    context: &mut StructParseContext<'_>,
) -> CompileResult<Option<(Vec<StructField>, usize)>> {
    let mut fields = Vec::new();
    let mut offset = 0usize;
    let mut max_alignment = 1usize;
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bit_fields = BitFieldState::new();
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth != 0
            || bracket_depth != 0
            || brace_depth != 0
            || !token_is_punctuator(token, ";")
        {
            update_depths(
                token,
                &mut paren_depth,
                &mut bracket_depth,
                &mut brace_depth,
            );
            continue;
        }
        let declaration = &tokens[start..index];
        if !declaration.is_empty()
            && !parse_struct_field_declaration(
                declaration,
                is_union,
                context,
                &mut StructFieldOutput {
                    fields: &mut fields,
                    offset: &mut offset,
                    max_alignment: &mut max_alignment,
                },
                &mut bit_fields,
            )?
        {
            return Ok(None);
        }
        start = index + 1;
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    if start < tokens.len() {
        return Ok(None);
    }
    let size = align_struct_offset(offset, max_alignment)?;
    Ok(Some((fields, size)))
}

fn parse_struct_field_declaration(
    tokens: &[Token],
    is_union: bool,
    context: &mut StructParseContext<'_>,
    output: &mut StructFieldOutput<'_>,
    bit_fields: &mut BitFieldState,
) -> CompileResult<bool> {
    if parse_nested_aggregate_field_declaration(tokens, is_union, context, output)? {
        bit_fields.clear();
        return Ok(true);
    }
    if let Some(name) = function_pointer_name(tokens) {
        bit_fields.clear();
        push_struct_field(
            &name,
            FieldType::Pointer { referent: None },
            is_union,
            context.available_structs.as_slice(),
            output,
        )?;
        return Ok(true);
    }
    let ranges = top_level_comma_ranges(tokens);
    let Some((first_start, first_end)) = ranges.first().copied() else {
        return Ok(false);
    };
    let first = &tokens[first_start..first_end];
    let Some(first_name_index) = declarator_name_index(first) else {
        for (start, end) in ranges {
            let segment = &tokens[start..end];
            let Some(width) = bit_field_width(segment) else {
                return Ok(false);
            };
            push_bit_field(width, is_union, output, bit_fields)?;
        }
        return Ok(true);
    };
    let base_specifiers = &first[..first_name_index];
    let Some(base_type) = struct_field_type(
        base_specifiers,
        context.available_structs.as_slice(),
        context.pointer_typedefs,
    ) else {
        return Ok(false);
    };
    for (range_index, (start, end)) in ranges.iter().copied().enumerate() {
        let segment = &tokens[start..end];
        if let Some(width) = bit_field_width(segment) {
            push_bit_field(width, is_union, output, bit_fields)?;
            continue;
        }
        bit_fields.clear();
        let Some(name_index) = declarator_name_index(segment) else {
            return Ok(false);
        };
        let Some(name) = token_identifier(&segment[name_index]) else {
            return Ok(false);
        };
        let field_type = declarator_field_type(
            segment,
            name_index,
            range_index,
            &base_type,
            context.constants,
        );
        push_struct_field(
            name,
            field_type,
            is_union,
            context.available_structs.as_slice(),
            output,
        )?;
    }
    Ok(true)
}

fn parse_nested_aggregate_field_declaration(
    tokens: &[Token],
    is_parent_union: bool,
    context: &mut StructParseContext<'_>,
    output: &mut StructFieldOutput<'_>,
) -> CompileResult<bool> {
    let Some(first) = tokens.first() else {
        return Ok(false);
    };
    let is_union = if token_is_keyword(first, Keyword::Union) {
        true
    } else if token_is_keyword(first, Keyword::Struct) {
        false
    } else {
        return Ok(false);
    };
    let Some(open_brace) = top_level_punctuator_index(tokens, "{") else {
        return Ok(false);
    };
    let Some(close_brace) = matching_top_level_brace(tokens, open_brace) else {
        return Ok(false);
    };
    let field_name = tokens.get(close_brace + 1).and_then(token_identifier);
    if (field_name.is_some() && tokens.get(close_brace + 2).is_some())
        || (field_name.is_none() && tokens.get(close_brace + 1).is_some())
    {
        return Ok(false);
    }
    let struct_name = field_name.map_or_else(
        || {
            format!(
                "{}.__anonymous{}",
                context.parent_name,
                context.nested_layouts.len()
            )
        },
        |name| format!("{}.{}", context.parent_name, name),
    );
    let nested_fields = {
        let mut nested_context = StructParseContext {
            parent_name: &struct_name,
            available_structs: &mut *context.available_structs,
            constants: context.constants,
            pointer_typedefs: context.pointer_typedefs,
            nested_layouts: &mut *context.nested_layouts,
        };
        parse_struct_fields(
            &tokens[open_brace + 1..close_brace],
            is_union,
            &mut nested_context,
        )?
    };
    let Some((fields, size)) = nested_fields else {
        return Ok(false);
    };
    let layout = StructLayout {
        name: struct_name.clone(),
        fields,
        size,
    };
    context.available_structs.push(layout.clone());
    context.nested_layouts.push(layout.clone());
    if let Some(name) = field_name {
        push_struct_field(
            name,
            FieldType::Struct(struct_name),
            is_parent_union,
            context.available_structs.as_slice(),
            output,
        )?;
    } else {
        push_anonymous_aggregate_fields(
            &layout,
            is_parent_union,
            context.available_structs.as_slice(),
            output,
        )?;
    }
    Ok(true)
}

fn push_anonymous_aggregate_fields(
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
            .ok_or_else(|| crate::diagnostics::CompileError::new("struct size overflow"))?;
        base_offset
    };
    for field in &layout.fields {
        let offset = base_offset.checked_add(field.offset).ok_or_else(|| {
            crate::diagnostics::CompileError::new("struct member offset overflow")
        })?;
        output.fields.push(StructField {
            name: field.name.clone(),
            field_type: field.field_type.clone(),
            offset,
        });
    }
    Ok(())
}
