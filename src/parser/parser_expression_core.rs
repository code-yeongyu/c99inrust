use super::{
    AssignmentOperator, BinaryOp, CompileResult, Expr, Keyword, Parser, ScalarType, UnaryOp,
    local_scalar_initializer, lvalue_from_expr, prefix_update_expr, token_is_punctuator,
};

impl Parser<'_> {
    pub(super) fn expression(&mut self) -> CompileResult<Expr> {
        let mut expr = self.assignment()?;
        while self.check_punctuator(",") {
            self.advance();
            expr = Expr::Comma {
                left: Box::new(expr),
                right: Box::new(self.assignment()?),
            };
        }
        Ok(expr)
    }

    pub(super) fn assignment(&mut self) -> CompileResult<Expr> {
        let target = self.conditional()?;
        let Some(op) = self.assignment_operator_at_current() else {
            return Ok(target);
        };
        self.advance();
        let lvalue = lvalue_from_expr(target.clone())?;
        let value = self.assignment()?;
        let value = match op {
            AssignmentOperator::Simple => value,
            AssignmentOperator::Compound(op) => Expr::Binary {
                op,
                left: Box::new(target),
                right: Box::new(value),
            },
        };
        Ok(Expr::Assignment {
            target: lvalue,
            value: Box::new(value),
        })
    }

    pub(super) fn conditional(&mut self) -> CompileResult<Expr> {
        let condition = self.logical_or()?;
        if !self.check_punctuator("?") {
            return Ok(condition);
        }
        self.advance();
        let then_expr = self.expression()?;
        self.expect_punctuator(":")?;
        let else_expr = self.conditional()?;
        Ok(Expr::Conditional {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
        })
    }

    pub(super) fn logical_or(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::logical_and, &[("||", BinaryOp::LogicalOr)])
    }

    pub(super) fn logical_and(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::bit_or, &[("&&", BinaryOp::LogicalAnd)])
    }

    pub(super) fn bit_or(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::bit_xor, &[("|", BinaryOp::BitOr)])
    }

    pub(super) fn bit_xor(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::bit_and, &[("^", BinaryOp::BitXor)])
    }

    pub(super) fn bit_and(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::equality, &[("&", BinaryOp::BitAnd)])
    }

    pub(super) fn equality(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::relational,
            &[("==", BinaryOp::Equal), ("!=", BinaryOp::NotEqual)],
        )
    }

    pub(super) fn relational(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::shift,
            &[
                ("<", BinaryOp::Less),
                ("<=", BinaryOp::LessEqual),
                (">", BinaryOp::Greater),
                (">=", BinaryOp::GreaterEqual),
            ],
        )
    }

    pub(super) fn shift(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::additive,
            &[("<<", BinaryOp::ShiftLeft), (">>", BinaryOp::ShiftRight)],
        )
    }

    pub(super) fn additive(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::multiplicative,
            &[("+", BinaryOp::Add), ("-", BinaryOp::Sub)],
        )
    }

    pub(super) fn multiplicative(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::unary,
            &[
                ("*", BinaryOp::Mul),
                ("/", BinaryOp::Div),
                ("%", BinaryOp::Mod),
            ],
        )
    }

    pub(super) fn left_associative(
        &mut self,
        next: fn(&mut Self) -> CompileResult<Expr>,
        ops: &[(&str, BinaryOp)],
    ) -> CompileResult<Expr> {
        let mut expr = next(self)?;
        loop {
            let Some((punctuator, op)) = ops
                .iter()
                .find(|(punctuator, _op)| self.check_punctuator(punctuator))
                .copied()
            else {
                return Ok(expr);
            };
            self.expect_punctuator(punctuator)?;
            let right = next(self)?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
    }

    pub(super) fn unary(&mut self) -> CompileResult<Expr> {
        if let Some(expr) = self.compound_literal_at_current()? {
            return self.postfix_suffixes(expr);
        }
        if let Some((target, referent, next_index)) = self.cast_type_at_current() {
            if self
                .tokens
                .get(next_index)
                .is_some_and(|token| token_is_punctuator(token, "{"))
            {
                self.index = next_index;
                let expr = self.scalar_compound_literal(target, referent)?;
                return self.postfix_suffixes(expr);
            }
            self.index = next_index;
            return Ok(Expr::Cast {
                target,
                referent,
                expr: Box::new(self.unary()?),
            });
        }
        if self.check_keyword(Keyword::Sizeof) {
            return self.sizeof_expr();
        }
        if self.check_punctuator("++") {
            self.advance();
            return prefix_update_expr(self.unary()?, false);
        }
        if self.check_punctuator("--") {
            self.advance();
            return prefix_update_expr(self.unary()?, true);
        }
        if self.check_punctuator("&") {
            self.advance();
            let target = lvalue_from_expr(self.unary()?)?;
            return Ok(Expr::AddressOf { target });
        }
        if self.check_punctuator("*") {
            self.advance();
            return Ok(Expr::Dereference {
                pointer: Box::new(self.unary()?),
            });
        }
        let op = if self.check_punctuator("+") {
            Some(UnaryOp::Plus)
        } else if self.check_punctuator("-") {
            Some(UnaryOp::Minus)
        } else if self.check_punctuator("~") {
            Some(UnaryOp::BitNot)
        } else if self.check_punctuator("!") {
            Some(UnaryOp::LogicalNot)
        } else {
            None
        };
        if let Some(op) = op {
            self.advance();
            return Ok(Expr::Unary {
                op,
                expr: Box::new(self.unary()?),
            });
        }
        self.postfix()
    }

    fn scalar_compound_literal(
        &mut self,
        target: ScalarType,
        referent: Option<String>,
    ) -> CompileResult<Expr> {
        self.expect_punctuator("{")?;
        let expr = self.expression()?;
        if self.check_punctuator(",") {
            self.advance();
        }
        self.expect_punctuator("}")?;
        let value = scalar_compound_value(target, referent.as_deref(), expr);
        Ok(Expr::ScalarCompoundLiteral {
            scalar_type: target,
            referent,
            value: Box::new(value),
        })
    }
}

fn scalar_compound_value(target: ScalarType, referent: Option<&str>, expr: Expr) -> Expr {
    match referent {
        Some("byte") => local_scalar_initializer(target, true, false, true, expr),
        Some("char") => local_scalar_initializer(target, true, false, false, expr),
        Some("unsigned short") => local_scalar_initializer(target, false, true, true, expr),
        Some("short") => local_scalar_initializer(target, false, true, false, expr),
        _ => expr,
    }
}
