use super::{
    LocalBinding, LocalSlot, LoweringContext, align_to, local_char_matrix_byte_size,
    local_int_array_byte_size, local_pointer_array_byte_size, local_short_array_byte_size,
    scalar_size, struct_alignment,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::ScalarType;

impl LoweringContext {
    pub(in crate::ir) fn declare_local(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        referent: Option<String>,
    ) -> CompileResult<usize> {
        if scalar_type == ScalarType::VaList {
            return self.declare_slot(
                name,
                scalar_type,
                scalar_size(scalar_type),
                scalar_size(ScalarType::Pointer),
                LocalBinding::VaList {
                    slot: self.local_slots.len(),
                },
            );
        }
        self.declare_slot(
            name,
            scalar_type,
            scalar_size(scalar_type),
            scalar_size(scalar_type),
            LocalBinding::Scalar {
                slot: self.local_slots.len(),
                scalar_type,
                referent,
            },
        )
    }

    pub(in crate::ir) fn declare_static_scalar(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        referent: Option<String>,
    ) -> CompileResult<String> {
        self.ensure_scope_name_available(name)?;
        let global_name = format!("{}__static__{name}", self.function_name);
        self.insert_scope_binding(
            name,
            LocalBinding::StaticScalar {
                global_name: global_name.clone(),
                scalar_type,
                referent,
            },
        )?;
        Ok(global_name)
    }

    pub(in crate::ir) fn declare_anonymous_slot(
        &mut self,
        scalar_type: ScalarType,
        byte_size: usize,
        alignment: usize,
    ) -> CompileResult<usize> {
        let slot = self.local_slots.len();
        let offset = align_to(self.next_local_offset, alignment);
        self.next_local_offset = offset
            .checked_add(byte_size)
            .ok_or_else(|| CompileError::new("local stack size overflow"))?;
        self.local_slots.push(LocalSlot {
            offset,
            scalar_type,
            byte_size,
        });
        Ok(slot)
    }

    pub(in crate::ir) fn declare_char_array(
        &mut self,
        name: &str,
        length: usize,
    ) -> CompileResult<usize> {
        self.declare_slot(
            name,
            ScalarType::Int,
            length,
            1,
            LocalBinding::CharArray {
                slot: self.local_slots.len(),
                length,
            },
        )
    }

    pub(in crate::ir) fn declare_char_matrix(
        &mut self,
        name: &str,
        rows: usize,
        columns: usize,
    ) -> CompileResult<usize> {
        let byte_size = local_char_matrix_byte_size(rows, columns)?;
        self.declare_slot(
            name,
            ScalarType::Int,
            byte_size,
            1,
            LocalBinding::CharMatrix {
                slot: self.local_slots.len(),
                rows,
                columns,
            },
        )
    }

    pub(in crate::ir) fn declare_int_array(
        &mut self,
        name: &str,
        length: usize,
    ) -> CompileResult<usize> {
        let byte_size = local_int_array_byte_size(length)?;
        self.declare_slot(
            name,
            ScalarType::Int,
            byte_size,
            scalar_size(ScalarType::Int),
            LocalBinding::IntArray {
                slot: self.local_slots.len(),
                length,
            },
        )
    }

    pub(in crate::ir) fn declare_short_array(
        &mut self,
        name: &str,
        length: usize,
        is_unsigned: bool,
    ) -> CompileResult<usize> {
        let byte_size = local_short_array_byte_size(length)?;
        self.declare_slot(
            name,
            ScalarType::Int,
            byte_size,
            2,
            LocalBinding::ShortArray {
                slot: self.local_slots.len(),
                length,
                is_unsigned,
            },
        )
    }

    pub(in crate::ir) fn declare_pointer_array(
        &mut self,
        name: &str,
        length: usize,
    ) -> CompileResult<usize> {
        let byte_size = local_pointer_array_byte_size(length)?;
        self.declare_slot(
            name,
            ScalarType::Pointer,
            byte_size,
            scalar_size(ScalarType::Pointer),
            LocalBinding::PointerArray {
                slot: self.local_slots.len(),
                length,
            },
        )
    }

    pub(in crate::ir) fn declare_struct_object(
        &mut self,
        name: &str,
        struct_name: &str,
    ) -> CompileResult<usize> {
        let layout = self.struct_layout(struct_name)?.clone();
        self.declare_slot(
            name,
            ScalarType::Pointer,
            layout.size,
            struct_alignment(&layout),
            LocalBinding::StructObject {
                slot: self.local_slots.len(),
                struct_name: struct_name.to_owned(),
                byte_size: layout.size,
            },
        )
    }

    pub(in crate::ir) fn declare_slot(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        byte_size: usize,
        alignment: usize,
        binding: LocalBinding,
    ) -> CompileResult<usize> {
        self.ensure_scope_name_available(name)?;
        let slot = self.local_slots.len();
        let offset = align_to(self.next_local_offset, alignment);
        self.next_local_offset = offset + byte_size;
        self.local_slots.push(LocalSlot {
            offset,
            scalar_type,
            byte_size,
        });
        self.insert_scope_binding(name, binding)?;
        Ok(slot)
    }

    pub(in crate::ir) fn ensure_scope_name_available(&self, name: &str) -> CompileResult<()> {
        let Some(scope) = self.scopes.last() else {
            return Err(CompileError::new("internal error: no local scope"));
        };
        if scope.contains_key(name) {
            return Err(CompileError::new(format!(
                "duplicate local declaration: {name}"
            )));
        }
        Ok(())
    }
}
