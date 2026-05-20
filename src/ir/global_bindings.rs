use super::GlobalBinding;
use crate::diagnostics::{CompileError, CompileResult};
use std::collections::HashMap;

pub(in crate::ir) fn insert_builtin_libc_bindings(bindings: &mut HashMap<String, GlobalBinding>) {
    bindings
        .entry("errno".to_owned())
        .or_insert(GlobalBinding::Int);
    for name in ["stdin", "stdout", "stderr"] {
        bindings
            .entry(name.to_owned())
            .or_insert(GlobalBinding::Pointer { referent: None });
    }
}

pub(in crate::ir) fn insert_global_binding(
    bindings: &mut HashMap<String, GlobalBinding>,
    name: &str,
    binding: GlobalBinding,
) -> CompileResult<()> {
    if let Some(existing) = bindings.get(name) {
        let Some(merged) = merge_global_binding(existing, &binding) else {
            return Err(CompileError::new(format!(
                "conflicting global declaration: {name}"
            )));
        };
        bindings.insert(name.to_owned(), merged);
        return Ok(());
    }
    bindings.insert(name.to_owned(), binding);
    Ok(())
}

pub(in crate::ir) fn merge_global_binding(
    existing: &GlobalBinding,
    incoming: &GlobalBinding,
) -> Option<GlobalBinding> {
    if existing == incoming {
        return Some(existing.clone());
    }
    match (existing, incoming) {
        (
            GlobalBinding::PointerArray { referent, columns },
            GlobalBinding::PointerArray {
                referent: incoming_referent,
                columns: incoming_columns,
            },
        ) if referent == incoming_referent => {
            let OptionalUsizeMerge::Compatible(merged_columns) =
                merge_optional_usize(*columns, *incoming_columns)
            else {
                return None;
            };
            Some(GlobalBinding::PointerArray {
                referent: referent.clone(),
                columns: merged_columns,
            })
        }
        (
            GlobalBinding::StructArray {
                struct_name,
                byte_size,
                length,
                columns,
            },
            GlobalBinding::StructArray {
                struct_name: incoming_name,
                byte_size: incoming_byte_size,
                length: incoming_length,
                columns: incoming_columns,
            },
        ) if struct_name == incoming_name && byte_size == incoming_byte_size => {
            let OptionalUsizeMerge::Compatible(merged_length) =
                merge_optional_usize(*length, *incoming_length)
            else {
                return None;
            };
            let OptionalUsizeMerge::Compatible(merged_columns) =
                merge_optional_usize(*columns, *incoming_columns)
            else {
                return None;
            };
            Some(GlobalBinding::StructArray {
                struct_name: struct_name.clone(),
                byte_size: *byte_size,
                length: merged_length,
                columns: merged_columns,
            })
        }
        _ => None,
    }
}

pub(in crate::ir) enum OptionalUsizeMerge {
    Compatible(Option<usize>),
    Conflict,
}

pub(in crate::ir) const fn merge_optional_usize(
    existing: Option<usize>,
    incoming: Option<usize>,
) -> OptionalUsizeMerge {
    match (existing, incoming) {
        (Some(existing), Some(new)) => {
            if existing == new {
                OptionalUsizeMerge::Compatible(Some(existing))
            } else {
                OptionalUsizeMerge::Conflict
            }
        }
        (Some(existing), None) | (None, Some(existing)) => {
            OptionalUsizeMerge::Compatible(Some(existing))
        }
        (None, None) => OptionalUsizeMerge::Compatible(None),
    }
}
