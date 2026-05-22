use super::{
    GlobalBinding, LocalBinding, LoweredExpr, LoweringContext, builtin_calls, call_args,
    complex_truth_expr, lowered_expr_scalar_type,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, ScalarType};

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
                return_type: self.indirect_call_return_type_for_name(callee),
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
        } else if let Some(return_type) = self.function_return_types.get(callee) {
            *return_type
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
        let return_type = self.indirect_call_return_type(callee);
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
            return_type,
        })
    }

    fn indirect_call_return_type(&self, callee: &Expr) -> ScalarType {
        match callee {
            Expr::Identifier(name) => self.indirect_call_return_type_for_name(name),
            Expr::Subscript { array, .. } => self
                .function_pointer_array_return_type(array)
                .unwrap_or(ScalarType::Int),
            _ => ScalarType::Int,
        }
    }

    fn indirect_call_return_type_for_name(&self, name: &str) -> ScalarType {
        self.local_binding(name)
            .and_then(|binding| function_pointer_return_type(&binding))
            .or_else(|| {
                self.global_bindings
                    .get(name)
                    .and_then(global_function_pointer_return_type)
            })
            .unwrap_or(ScalarType::Int)
    }

    fn function_pointer_array_return_type(&self, array: &Expr) -> Option<ScalarType> {
        let Expr::Identifier(name) = array else {
            return None;
        };
        self.local_binding(name)
            .and_then(|binding| function_pointer_return_type(&binding))
            .or_else(|| {
                self.global_bindings
                    .get(name)
                    .and_then(global_function_pointer_return_type)
            })
    }

    pub(in crate::ir) fn lower_cast_expr(
        &self,
        target: ScalarType,
        expr: &Expr,
    ) -> CompileResult<LoweredExpr> {
        if target == ScalarType::Bool {
            return self.lower_bool_cast_expr(expr);
        }
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

    pub(in crate::ir) fn lower_bool_cast_expr(&self, expr: &Expr) -> CompileResult<LoweredExpr> {
        Ok(LoweredExpr::Binary {
            op: BinaryOp::NotEqual,
            left: Box::new(self.lower_condition_expr(expr)?),
            right: Box::new(LoweredExpr::Integer(0)),
        })
    }

    pub(in crate::ir) fn lower_condition_expr(&self, expr: &Expr) -> CompileResult<LoweredExpr> {
        let value = self.lower_expr(expr)?;
        Ok(Self::complex_truth_for_lowered(&value).unwrap_or(value))
    }

    pub(in crate::ir) fn complex_truth_for_lowered(value: &LoweredExpr) -> Option<LoweredExpr> {
        complex_truth_expr(value)
    }
}

fn function_pointer_return_type(binding: &LocalBinding) -> Option<ScalarType> {
    let referent = match binding {
        LocalBinding::Scalar {
            referent: Some(referent),
            ..
        }
        | LocalBinding::StaticScalar {
            referent: Some(referent),
            ..
        }
        | LocalBinding::PointerArray {
            referent: Some(referent),
            ..
        } => referent.as_str(),
        _ => return None,
    };
    function_return_type_for_referent(referent)
}

fn global_function_pointer_return_type(binding: &GlobalBinding) -> Option<ScalarType> {
    let referent = match binding {
        GlobalBinding::Pointer {
            referent: Some(referent),
        }
        | GlobalBinding::PointerArray {
            referent: Some(referent),
            ..
        } => referent.as_str(),
        _ => return None,
    };
    function_return_type_for_referent(referent)
}

fn function_return_type_for_referent(referent: &str) -> Option<ScalarType> {
    match referent {
        "function double" => Some(ScalarType::Double),
        "function long double" => Some(ScalarType::LongDouble),
        "function pointer" => Some(ScalarType::Pointer),
        "function int" => Some(ScalarType::Int),
        _ => None,
    }
}
