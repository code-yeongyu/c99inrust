use super::{LoweredExpr, LoweredLValue, LoweringContext, pointer_arithmetic, scalar_size};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, LValue, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn is_scalar_compound_address(initializer: &Expr) -> bool {
        scalar_compound_address(initializer).is_ok()
    }

    pub(in crate::ir) fn lower_scalar_compound_pointer_initializer(
        &mut self,
        pointer_slot: usize,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let target = LoweredLValue::Local {
            slot: pointer_slot,
            offset: self.local_offset(pointer_slot)?,
            scalar_type: ScalarType::Pointer,
        };
        self.lower_scalar_compound_pointer_assignment(target, initializer)
    }

    pub(in crate::ir) fn lower_scalar_compound_pointer_assignment(
        &mut self,
        target: LoweredLValue,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let pointer = self.lower_scalar_compound_pointer(initializer)?;
        self.push_store(target, pointer)
    }

    fn lower_scalar_compound_pointer(&mut self, initializer: &Expr) -> CompileResult<LoweredExpr> {
        let (scalar_type, referent, value) = scalar_compound_address(initializer)?;
        let byte_size = scalar_compound_byte_size(scalar_type, referent);
        let slot = self.declare_anonymous_slot(scalar_type, byte_size, byte_size)?;
        let pointer = LoweredExpr::LocalAddress {
            offset: self.local_offset(slot)?,
            byte_size,
        };
        let target = LoweredLValue::PointerSubscript {
            pointer: Box::new(pointer.clone()),
            index: Box::new(LoweredExpr::Integer(0)),
            element_type: scalar_type,
            element_byte_size: byte_size,
            element_unsigned: referent == Some("byte"),
        };
        let value = self.lower_expr(value)?;
        self.push_store(target, value)?;
        Ok(pointer)
    }

    pub(in crate::ir) fn lower_scalar_compound_assignment_expr(
        &self,
        target: &LValue,
        value: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let LValue::ScalarCompoundLiteral {
            scalar_type,
            referent,
            value: initializer,
        } = target
        else {
            return Ok(None);
        };
        Ok(Some(LoweredExpr::Comma {
            left: Box::new(self.lower_scalar_compound_value(
                *scalar_type,
                referent.as_deref(),
                initializer,
            )?),
            right: Box::new(self.lower_scalar_compound_value(
                *scalar_type,
                referent.as_deref(),
                value,
            )?),
        }))
    }

    fn lower_scalar_compound_value(
        &self,
        scalar_type: ScalarType,
        referent: Option<&str>,
        value: &Expr,
    ) -> CompileResult<LoweredExpr> {
        if scalar_type == ScalarType::Int {
            match referent {
                Some("byte") => return Ok(masked_integer(self.lower_expr(value)?, 255)),
                Some("char") => {
                    return Ok(signed_narrow_integer(
                        self.lower_expr(value)?,
                        255,
                        128,
                        256,
                    ));
                }
                Some("short") => {
                    return Ok(signed_narrow_integer(
                        self.lower_expr(value)?,
                        65_535,
                        32_768,
                        65_536,
                    ));
                }
                _ => {}
            }
        }
        self.lower_cast_expr(scalar_type, value)
    }
}

fn scalar_compound_address(initializer: &Expr) -> CompileResult<(ScalarType, Option<&str>, &Expr)> {
    let Expr::AddressOf {
        target:
            LValue::ScalarCompoundLiteral {
                scalar_type,
                referent,
                value,
            },
    } = initializer
    else {
        return Err(CompileError::new(
            "expected address of scalar compound literal",
        ));
    };
    Ok((*scalar_type, referent.as_deref(), value))
}

fn scalar_compound_byte_size(scalar_type: ScalarType, referent: Option<&str>) -> usize {
    if let Some(referent) = referent
        && let Some(byte_size) = pointer_arithmetic::byte_size(referent)
    {
        return byte_size;
    }
    scalar_size(scalar_type)
}

fn signed_narrow_integer(expr: LoweredExpr, mask: i64, sign_bit: i64, range: i64) -> LoweredExpr {
    let masked = masked_integer(expr, mask);
    LoweredExpr::Conditional {
        condition: Box::new(LoweredExpr::Binary {
            op: BinaryOp::GreaterEqual,
            left: Box::new(masked.clone()),
            right: Box::new(LoweredExpr::Integer(sign_bit)),
        }),
        then_expr: Box::new(LoweredExpr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(masked.clone()),
            right: Box::new(LoweredExpr::Integer(range)),
        }),
        else_expr: Box::new(masked),
    }
}

fn masked_integer(expr: LoweredExpr, mask: i64) -> LoweredExpr {
    LoweredExpr::Binary {
        op: BinaryOp::BitAnd,
        left: Box::new(expr),
        right: Box::new(LoweredExpr::Integer(mask)),
    }
}
