use super::{
    GlobalBinding, LocalBinding, LoweredExpr, LoweringContext, StructAddress, pointer_arithmetic,
    pointer_referent,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, FieldType};

impl LoweringContext {
    pub(in crate::ir) fn resolve_struct_address(
        &self,
        expr: &Expr,
    ) -> CompileResult<StructAddress> {
        if let Expr::Identifier(name) = expr
            && let Some(LocalBinding::StructObject {
                slot,
                struct_name,
                byte_size,
            }) = self.local_binding(name)
        {
            return Ok(StructAddress {
                pointer: LoweredExpr::LocalAddress {
                    offset: self.local_offset(slot)?,
                    byte_size,
                },
                offset: 0,
                struct_name,
            });
        }
        if let Expr::Identifier(name) = expr
            && let Some(GlobalBinding::StructObject {
                struct_name,
                byte_size: _,
            }) = self.global_bindings.get(name)
        {
            return Ok(StructAddress {
                pointer: LoweredExpr::GlobalAddress { name: name.clone() },
                offset: 0,
                struct_name: struct_name.clone(),
            });
        }
        if let Expr::Member {
            base,
            field,
            dereference,
        } = expr
        {
            let member = self.resolve_member_access(base, field, *dereference)?;
            let FieldType::Struct(struct_name) = member.field_type else {
                return Err(CompileError::new("member base is not a struct"));
            };
            return Ok(StructAddress {
                pointer: member.pointer,
                offset: member.offset,
                struct_name,
            });
        }
        if let Expr::Dereference { pointer } = expr {
            let struct_name = self.pointer_referent_for_expr(pointer)?;
            return Ok(StructAddress {
                pointer: self.lower_expr(pointer)?,
                offset: 0,
                struct_name,
            });
        }
        if let Expr::Subscript { array, index } = expr {
            if let Some(address) = self.resolve_global_struct_subscript_address(array, index)? {
                return Ok(address);
            }
            if let Some(address) =
                self.resolve_struct_array_field_subscript_address(array, index)?
            {
                return Ok(address);
            }
            return self.resolve_pointer_struct_subscript_address(array, index);
        }
        Err(CompileError::new("member access requires a struct base"))
    }

    pub(in crate::ir) fn resolve_global_struct_subscript_address(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<StructAddress>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::StructArray {
            struct_name,
            byte_size,
            columns,
            ..
        }) = self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        if columns.is_some() {
            return Ok(None);
        }
        Ok(Some(StructAddress {
            pointer: LoweredExpr::PointerOffset {
                pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
                index: Box::new(self.lower_expr(index)?),
                byte_size: *byte_size,
            },
            offset: 0,
            struct_name: struct_name.clone(),
        }))
    }

    pub(in crate::ir) fn resolve_pointer_struct_subscript_address(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<StructAddress> {
        let struct_name = self.pointer_referent_for_expr(array)?;
        let byte_size = self.struct_layout(&struct_name)?.size;
        Ok(StructAddress {
            pointer: LoweredExpr::PointerOffset {
                pointer: Box::new(self.lower_expr(array)?),
                index: Box::new(self.lower_expr(index)?),
                byte_size,
            },
            offset: 0,
            struct_name,
        })
    }

    pub(in crate::ir) fn pointer_referent_for_identifier(&self, name: &str) -> Option<String> {
        if name == "__func__" {
            return Some("char".to_owned());
        }
        if let Some(binding) = self.local_binding(name) {
            return match binding {
                LocalBinding::Scalar {
                    referent: Some(referent),
                    ..
                }
                | LocalBinding::StaticScalar {
                    referent: Some(referent),
                    ..
                } => Some(referent),
                LocalBinding::ShortArray { .. } => Some("short".to_owned()),
                _ => None,
            };
        }
        if let Some(GlobalBinding::Pointer {
            referent: Some(referent),
        }) = self.global_bindings.get(name)
        {
            return Some(referent.clone());
        }
        if let Some(GlobalBinding::StructArray { struct_name, .. }) = self.global_bindings.get(name)
        {
            return Some(struct_name.clone());
        }
        if matches!(
            self.global_bindings.get(name),
            Some(GlobalBinding::ShortArray { .. })
        ) {
            return Some("short".to_owned());
        }
        None
    }

    pub(in crate::ir) fn pointer_referent_for_expr(&self, expr: &Expr) -> CompileResult<String> {
        pointer_referent::for_expr(self, expr)
    }

    pub(in crate::ir) fn pointer_referent_stride(&self, referent: &str) -> CompileResult<usize> {
        pointer_arithmetic::byte_size(referent)
            .or_else(|| self.structs.get(referent).map(|layout| layout.size))
            .ok_or_else(|| CompileError::new("unknown pointer arithmetic referent"))
    }

    pub(in crate::ir) fn pointer_difference_stride(
        &self,
        left_referent: &str,
        right_referent: &str,
    ) -> CompileResult<usize> {
        pointer_arithmetic::difference_stride(
            self.pointer_referent_stride(left_referent)?,
            self.pointer_referent_stride(right_referent)?,
        )
    }

    pub(in crate::ir) fn expr_is_pointer_return_call(&self, expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::Call { callee, .. } if self.pointer_return_functions.contains_key(callee)
        )
    }
}
