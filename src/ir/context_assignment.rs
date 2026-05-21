use super::{
    GlobalBinding, LocalBinding, LoweredExpr, LoweredLValue, LoweringContext, StructAddress,
    sizeof_expr,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, FieldType, LValue};

impl LoweringContext {
    pub(in crate::ir) fn lower_assignment(
        &mut self,
        target: &LValue,
        value: &Expr,
    ) -> CompileResult<()> {
        if let Some(struct_target) = self.resolve_struct_lvalue_address(target)? {
            let source = self.resolve_struct_address(value)?;
            if source.struct_name != struct_target.struct_name {
                return Err(CompileError::new("incompatible struct assignment"));
            }
            self.push_struct_copy(&struct_target, &source)?;
            return Ok(());
        }
        let target = self.lower_lvalue(target)?;
        let value = self.lower_expr(value)?;
        self.push_store(target, value);
        Ok(())
    }

    pub(in crate::ir) fn lower_lvalue(&self, target: &LValue) -> CompileResult<LoweredLValue> {
        match target {
            LValue::Identifier(name) => {
                if let Some(binding) = self.local_binding(name) {
                    return match binding {
                        LocalBinding::Scalar {
                            slot, scalar_type, ..
                        } => Ok(LoweredLValue::Local {
                            slot,
                            offset: self.local_offset(slot)?,
                            scalar_type,
                        }),
                        LocalBinding::StaticScalar {
                            global_name,
                            scalar_type,
                            ..
                        } => Ok(LoweredLValue::Global {
                            name: global_name,
                            scalar_type,
                        }),
                        LocalBinding::CharArray { .. }
                        | LocalBinding::CharMatrix { .. }
                        | LocalBinding::IntArray { .. }
                        | LocalBinding::ShortArray { .. } => Err(CompileError::new(
                            "assignment to local array is not supported",
                        )),
                        LocalBinding::PointerArray { .. } => Err(CompileError::new(
                            "assignment to local pointer array is not supported",
                        )),
                        LocalBinding::StructObject { .. } => Err(CompileError::new(
                            "direct assignment to local struct object is not supported",
                        )),
                        LocalBinding::StructArray { .. } => Err(CompileError::new(
                            "assignment to local struct array is not supported",
                        )),
                        LocalBinding::VaList { .. } => {
                            Err(CompileError::new("assignment to va_list is not supported"))
                        }
                    };
                }
                if let Some(scalar_type) = self
                    .global_bindings
                    .get(name)
                    .and_then(GlobalBinding::scalar_type)
                {
                    return Ok(LoweredLValue::Global {
                        name: name.clone(),
                        scalar_type,
                    });
                }
                Err(CompileError::new(format!(
                    "assignment to undeclared local or global: {name}"
                )))
            }
            LValue::Subscript { array, index } => self.lower_subscript_lvalue(array, index),
            LValue::Member {
                base,
                field,
                dereference,
            } => self.lower_member_lvalue(base, field, *dereference),
        }
    }

    pub(in crate::ir) fn resolve_struct_lvalue_address(
        &self,
        target: &LValue,
    ) -> CompileResult<Option<StructAddress>> {
        match target {
            LValue::Identifier(name) => {
                if let Some(LocalBinding::StructObject {
                    slot,
                    struct_name,
                    byte_size,
                }) = self.local_binding(name)
                {
                    return Ok(Some(StructAddress {
                        pointer: LoweredExpr::LocalAddress {
                            offset: self.local_offset(slot)?,
                            byte_size,
                        },
                        offset: 0,
                        struct_name,
                    }));
                }
                if let Some(GlobalBinding::StructObject { struct_name, .. }) =
                    self.global_bindings.get(name)
                {
                    return Ok(Some(StructAddress {
                        pointer: LoweredExpr::GlobalAddress { name: name.clone() },
                        offset: 0,
                        struct_name: struct_name.clone(),
                    }));
                }
                Ok(None)
            }
            LValue::Member {
                base,
                field,
                dereference,
            } => {
                let member = self.resolve_member_access(base, field, *dereference)?;
                let FieldType::Struct(struct_name) = member.field_type else {
                    return Ok(None);
                };
                Ok(Some(StructAddress {
                    pointer: member.pointer,
                    offset: member.offset,
                    struct_name,
                }))
            }
            LValue::Subscript { array, index } => {
                if let Some(address) = self.resolve_local_struct_subscript_address(array, index)? {
                    return Ok(Some(address));
                }
                if let Some(address) =
                    self.resolve_struct_array_field_subscript_address(array, index)?
                {
                    return Ok(Some(address));
                }
                if let Ok(address) = self.resolve_pointer_struct_subscript_address(array, index) {
                    return Ok(Some(address));
                }
                Ok(None)
            }
        }
    }

    pub(in crate::ir) fn lower_sizeof_expr(&self, expr: &Expr) -> CompileResult<LoweredExpr> {
        sizeof_expr::lower(self, expr)
    }
}
