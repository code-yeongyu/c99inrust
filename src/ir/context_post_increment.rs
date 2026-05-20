use super::{
    LoweredExpr, LoweredLValue, LoweringContext, ensure_post_increment_scalar,
    lowered_lvalue_scalar_type, lowered_lvalue_to_expr,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, FieldType, LValue, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_post_increment_statement(
        &mut self,
        target: &LValue,
        decrement: bool,
    ) -> CompileResult<()> {
        let lowered = self.lower_lvalue(target)?;
        ensure_post_increment_scalar(&lowered)?;
        let increment = self.post_increment_amount(target, &lowered, decrement)?;
        let current = lowered_lvalue_to_expr(&lowered);
        self.push_store(
            lowered,
            LoweredExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(current),
                right: Box::new(LoweredExpr::Integer(increment)),
            },
        );
        Ok(())
    }

    pub(in crate::ir) fn lower_post_increment_expr(
        &self,
        target: &LValue,
        decrement: bool,
    ) -> CompileResult<LoweredExpr> {
        let lowered = self.lower_lvalue(target)?;
        ensure_post_increment_scalar(&lowered)?;
        let increment = self.post_increment_amount(target, &lowered, decrement)?;
        Ok(LoweredExpr::PostIncrement {
            target: lowered,
            increment,
        })
    }

    pub(in crate::ir) fn post_increment_amount(
        &self,
        target: &LValue,
        lowered: &LoweredLValue,
        decrement: bool,
    ) -> CompileResult<i64> {
        let amount = if lowered_lvalue_scalar_type(lowered) == ScalarType::Pointer {
            self.pointer_referent_for_lvalue(target)?
                .map_or(Ok(1), |referent| {
                    let stride = self.pointer_referent_stride(&referent)?;
                    i64::try_from(stride)
                        .map_err(|_| CompileError::new("pointer stride does not fit i64"))
                })?
        } else {
            1
        };
        Ok(if decrement { -amount } else { amount })
    }

    pub(in crate::ir) fn pointer_referent_for_lvalue(
        &self,
        target: &LValue,
    ) -> CompileResult<Option<String>> {
        match target {
            LValue::Identifier(name) => Ok(self.pointer_referent_for_identifier(name)),
            LValue::Member {
                base,
                field,
                dereference,
            } => {
                let member = self.resolve_member_access(base, field, *dereference)?;
                if let FieldType::Pointer { referent } = member.field_type {
                    Ok(referent)
                } else {
                    Ok(None)
                }
            }
            LValue::Subscript { .. } => Ok(None),
        }
    }
}
