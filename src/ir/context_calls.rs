use super::{
    GlobalBinding, LocalBinding, LoweredExpr, LoweringContext, builtin_calls, call_args,
    lowered_expr_scalar_type,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_call_expr(
        &self,
        callee: &str,
        args: &[Expr],
    ) -> CompileResult<LoweredExpr> {
        if self.callee_is_pointer_binding(callee) {
            return Ok(LoweredExpr::IndirectCall {
                callee: Box::new(self.lower_identifier_expr(callee)?),
                args: args
                    .iter()
                    .map(|arg| self.lower_expr(arg))
                    .collect::<CompileResult<Vec<_>>>()?,
            });
        }
        Ok(LoweredExpr::Call {
            callee: callee.to_owned(),
            args: args
                .iter()
                .enumerate()
                .map(|(index, arg)| self.lower_call_arg(callee, index, arg))
                .collect::<CompileResult<Vec<_>>>()?,
            return_type: self.direct_call_return_type(callee),
        })
    }

    pub(in crate::ir) fn lower_call_arg(
        &self,
        callee: &str,
        index: usize,
        arg: &Expr,
    ) -> CompileResult<LoweredExpr> {
        call_args::lower(self, callee, index, arg)
    }

    pub(in crate::ir) fn direct_call_return_type(&self, callee: &str) -> ScalarType {
        if self.pointer_return_functions.contains_key(callee)
            || builtin_calls::returns_pointer(callee)
        {
            ScalarType::Pointer
        } else {
            ScalarType::Int
        }
    }

    pub(in crate::ir) fn callee_is_pointer_binding(&self, callee: &str) -> bool {
        if let Some(binding) = self.local_binding(callee) {
            return matches!(
                binding,
                LocalBinding::Scalar {
                    scalar_type: ScalarType::Pointer,
                    ..
                } | LocalBinding::StaticScalar {
                    scalar_type: ScalarType::Pointer,
                    ..
                }
            );
        }
        self.global_bindings
            .get(callee)
            .and_then(GlobalBinding::scalar_type)
            == Some(ScalarType::Pointer)
    }

    pub(in crate::ir) fn lower_indirect_call_expr(
        &self,
        callee: &Expr,
        args: &[Expr],
    ) -> CompileResult<LoweredExpr> {
        let callee = if let Expr::Dereference { pointer } = callee {
            pointer.as_ref()
        } else {
            callee
        };
        let callee = self.lower_expr(callee)?;
        if lowered_expr_scalar_type(&callee) != Some(ScalarType::Pointer) {
            return Err(CompileError::new("indirect call requires a pointer callee"));
        }
        Ok(LoweredExpr::IndirectCall {
            callee: Box::new(callee),
            args: args
                .iter()
                .map(|arg| self.lower_expr(arg))
                .collect::<CompileResult<Vec<_>>>()?,
        })
    }

    pub(in crate::ir) fn lower_cast_expr(
        &self,
        target: ScalarType,
        expr: &Expr,
    ) -> CompileResult<LoweredExpr> {
        let expr = if target == ScalarType::Pointer {
            self.lower_pointer_cast_expr(expr)?
        } else {
            self.lower_expr(expr)?
        };
        Ok(LoweredExpr::Cast {
            target,
            expr: Box::new(expr),
        })
    }

    pub(in crate::ir) fn lower_pointer_cast_expr(&self, expr: &Expr) -> CompileResult<LoweredExpr> {
        match self.lower_expr(expr) {
            Ok(lowered) => Ok(lowered),
            Err(error) => {
                if let Expr::Identifier(name) = expr {
                    return Ok(LoweredExpr::GlobalAddress { name: name.clone() });
                }
                Err(error)
            }
        }
    }
}
