use super::{
    GlobalBinding, LocalBinding, LoweredExpr, LoweringContext, local_char_matrix_byte_size,
    local_int_array_byte_size, local_pointer_array_byte_size, local_short_array_byte_size,
    scalar_size,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::ScalarType;

impl LoweringContext {
    pub(in crate::ir) fn lower_identifier_expr(&self, name: &str) -> CompileResult<LoweredExpr> {
        if let Some(binding) = self.local_binding(name) {
            return self.lower_local_identifier_expr(&binding);
        }
        if let Some(scalar_type) = self
            .global_bindings
            .get(name)
            .and_then(GlobalBinding::scalar_type)
        {
            return Ok(LoweredExpr::Global {
                name: name.to_owned(),
                scalar_type,
            });
        }
        if self
            .global_bindings
            .get(name)
            .is_some_and(GlobalBinding::is_addressable_array)
        {
            return Ok(LoweredExpr::GlobalAddress {
                name: name.to_owned(),
            });
        }
        if let Some(value) = self.constants.get(name) {
            return Ok(LoweredExpr::Integer(*value));
        }
        if self.function_names.contains(name) {
            return Ok(LoweredExpr::GlobalAddress {
                name: name.to_owned(),
            });
        }
        Err(CompileError::new(format!(
            "unknown local or global: {name}"
        )))
    }

    pub(in crate::ir) fn lower_local_identifier_expr(
        &self,
        binding: &LocalBinding,
    ) -> CompileResult<LoweredExpr> {
        match binding {
            LocalBinding::Scalar {
                slot, scalar_type, ..
            } => Ok(LoweredExpr::Local {
                offset: self.local_offset(*slot)?,
                scalar_type: *scalar_type,
            }),
            LocalBinding::StaticScalar {
                global_name,
                scalar_type,
                ..
            } => Ok(LoweredExpr::Global {
                name: global_name.clone(),
                scalar_type: *scalar_type,
            }),
            LocalBinding::CharArray { slot, length } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: *length,
            }),
            LocalBinding::CharMatrix {
                slot,
                rows,
                columns,
            } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: local_char_matrix_byte_size(*rows, *columns)?,
            }),
            LocalBinding::IntArray { slot, length } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: local_int_array_byte_size(*length)?,
            }),
            LocalBinding::ShortArray { slot, length, .. } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: local_short_array_byte_size(*length)?,
            }),
            LocalBinding::PointerArray { slot, length } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: local_pointer_array_byte_size(*length)?,
            }),
            LocalBinding::StructObject {
                slot, byte_size, ..
            } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: *byte_size,
            }),
            LocalBinding::VaList { slot } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: scalar_size(ScalarType::VaList),
            }),
        }
    }
}
