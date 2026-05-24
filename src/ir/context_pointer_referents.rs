use super::{GlobalBinding, LocalBinding, LoweringContext, pointer_arithmetic, pointer_referent};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn pointer_referent_for_identifier(&self, name: &str) -> Option<String> {
        if name == "__func__" {
            return Some("char".to_owned());
        }
        if let Some(binding) = self.local_binding(name) {
            return local_binding_referent(binding);
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
            Some(GlobalBinding::IntArray | GlobalBinding::IntMatrix { .. })
        ) {
            return Some("int".to_owned());
        }
        if matches!(
            self.global_bindings.get(name),
            Some(GlobalBinding::ShortArray { .. })
        ) {
            return Some("short".to_owned());
        }
        if let Some(GlobalBinding::ScalarArray { scalar_type, .. }) = self.global_bindings.get(name)
        {
            return scalar_array_referent(*scalar_type);
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

fn local_binding_referent(binding: LocalBinding) -> Option<String> {
    match binding {
        LocalBinding::Scalar {
            referent: Some(referent),
            ..
        }
        | LocalBinding::StaticScalar {
            referent: Some(referent),
            ..
        } => Some(referent),
        LocalBinding::CharArray { is_unsigned, .. } => {
            Some(if is_unsigned { "byte" } else { "char" }.to_owned())
        }
        LocalBinding::IntArray { .. } | LocalBinding::IntMatrix { .. } => Some("int".to_owned()),
        LocalBinding::ShortArray { .. } => Some("short".to_owned()),
        LocalBinding::ScalarArray { scalar_type, .. } => scalar_array_referent(scalar_type),
        LocalBinding::PointerArray { .. } => Some(pointer_arithmetic::nested_referent(None)),
        LocalBinding::StructArray { struct_name, .. } => Some(struct_name),
        _ => None,
    }
}

fn scalar_array_referent(scalar_type: ScalarType) -> Option<String> {
    match scalar_type {
        ScalarType::Bool => Some("_Bool".to_owned()),
        ScalarType::LongLong => Some("long long".to_owned()),
        ScalarType::Double => Some("double".to_owned()),
        ScalarType::LongDouble => Some("long double".to_owned()),
        ScalarType::ComplexFloat => Some("float _Complex".to_owned()),
        ScalarType::ComplexDouble => Some("double _Complex".to_owned()),
        ScalarType::ComplexLongDouble => Some("long double _Complex".to_owned()),
        ScalarType::Int | ScalarType::Pointer | ScalarType::VaList => None,
    }
}
