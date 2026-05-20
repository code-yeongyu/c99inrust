use super::{
    CompileError, CompileResult, Expr, Keyword, Parser, ScalarType, Token, TokenKind,
    function_pointer_cast_type, lvalue_from_expr, matching_top_level_paren,
    pointer_referent_from_specifiers, supported_cast_type_with_typedefs,
};

impl Parser<'_> {
    pub(super) fn sizeof_expr(&mut self) -> CompileResult<Expr> {
        self.expect_keyword(Keyword::Sizeof)?;
        if self.check_punctuator("(") {
            let start = self.index + 1;
            let close = matching_top_level_paren(self.tokens, self.index);
            if let Some(close) = close
                && let Some(size) = self.sizeof_type(&self.tokens[start..close])
            {
                self.index = close + 1;
                return i64::try_from(size)
                    .map(Expr::Integer)
                    .map_err(|_| CompileError::new("sizeof result does not fit i64"));
            }
        }
        Ok(Expr::SizeOfExpr {
            expr: Box::new(self.unary()?),
        })
    }

    pub(super) fn postfix(&mut self) -> CompileResult<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.check_punctuator("[") {
                self.advance();
                let index = self.expression()?;
                self.expect_punctuator("]")?;
                expr = Expr::Subscript {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
                continue;
            }
            if self.check_punctuator(".") || self.check_punctuator("->") {
                let dereference = self.check_punctuator("->");
                self.advance();
                let field = self.expect_identifier()?;
                expr = Expr::Member {
                    base: Box::new(expr),
                    field,
                    dereference,
                };
                continue;
            }
            if self.check_punctuator("(") {
                expr = Expr::IndirectCall {
                    callee: Box::new(expr),
                    args: self.call_arguments()?,
                };
                continue;
            }
            if self.check_punctuator("++") {
                self.advance();
                expr = Expr::PostIncrement {
                    target: lvalue_from_expr(expr)?,
                    decrement: false,
                };
                continue;
            }
            if self.check_punctuator("--") {
                self.advance();
                expr = Expr::PostIncrement {
                    target: lvalue_from_expr(expr)?,
                    decrement: true,
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    pub(super) fn cast_type_at_current(&self) -> Option<(ScalarType, Option<String>, usize)> {
        if !self.check_punctuator("(") {
            return None;
        }
        let start = self.index + 1;
        let close = matching_top_level_paren(self.tokens, self.index)?;
        let cast_tokens = &self.tokens[start..close];
        if function_pointer_cast_type(cast_tokens) {
            return Some((ScalarType::Pointer, None, close + 1));
        }
        let target = supported_cast_type_with_typedefs(
            cast_tokens,
            self.known_scalar_typedefs,
            self.known_pointer_typedefs,
        )?;
        let referent = if target == ScalarType::Pointer {
            pointer_referent_from_specifiers(cast_tokens)
        } else {
            None
        };
        Some((target, referent, close + 1))
    }

    pub(super) fn primary(&mut self) -> CompileResult<Expr> {
        if let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::Integer(value) => {
                    let value = *value;
                    self.advance();
                    if self.check_punctuator(".") {
                        self.advance();
                        let fractional = if self
                            .peek()
                            .and_then(|token| token.kind.integer_value())
                            .is_some()
                        {
                            self.expect_integer()?
                        } else {
                            0
                        };
                        let exponent = self.decimal_exponent_suffix()?;
                        return Ok(Expr::DoubleLiteral(format!(
                            "{value}.{fractional}{exponent}"
                        )));
                    }
                    if let Some(exponent) = self.optional_decimal_exponent_suffix()? {
                        return Ok(Expr::DoubleLiteral(format!("{value}{exponent}")));
                    }
                    Ok(Expr::Integer(value))
                }
                TokenKind::LongInteger(value) => {
                    let value = *value;
                    self.advance();
                    Ok(Expr::LongInteger(value))
                }
                TokenKind::CharLiteral(value) => {
                    let value = i64::from(u32::from(*value));
                    self.advance();
                    Ok(Expr::Integer(value))
                }
                TokenKind::StringLiteral(value) => {
                    let mut value = value.clone();
                    self.advance();
                    while let Some(Token {
                        kind: TokenKind::StringLiteral(next),
                        ..
                    }) = self.peek()
                    {
                        value.push_str(next);
                        self.advance();
                    }
                    Ok(Expr::StringLiteral(value))
                }
                TokenKind::Identifier(value) => {
                    let value = value.clone();
                    self.advance();
                    if self.check_punctuator("(") {
                        return Ok(Expr::Call {
                            callee: value,
                            args: self.call_arguments()?,
                        });
                    }
                    Ok(Expr::Identifier(value))
                }
                TokenKind::Punctuator(value) if value == "." => {
                    self.advance();
                    let fractional = self.expect_integer()?;
                    let exponent = self.decimal_exponent_suffix()?;
                    Ok(Expr::DoubleLiteral(format!("0.{fractional}{exponent}")))
                }
                TokenKind::Punctuator(value) if value == "(" => {
                    self.advance();
                    let expr = self.expression()?;
                    self.expect_punctuator(")")?;
                    Ok(expr)
                }
                _ => Err(CompileError::new("expected expression").at(token.line, token.column)),
            }
        } else {
            Err(CompileError::new("unexpected end of token stream"))
        }
    }

    pub(super) fn call_arguments(&mut self) -> CompileResult<Vec<Expr>> {
        self.expect_punctuator("(")?;
        let mut args = Vec::new();
        if self.check_punctuator(")") {
            self.advance();
            return Ok(args);
        }
        loop {
            args.push(self.assignment()?);
            if self.check_punctuator(")") {
                self.advance();
                return Ok(args);
            }
            self.expect_punctuator(",")?;
        }
    }

    fn decimal_exponent_suffix(&mut self) -> CompileResult<String> {
        let exponent = self.optional_decimal_exponent_suffix()?.unwrap_or_default();
        self.consume_float_suffix();
        Ok(exponent)
    }

    fn optional_decimal_exponent_suffix(&mut self) -> CompileResult<Option<String>> {
        let Some(Token {
            kind: TokenKind::Identifier(value),
            ..
        }) = self.peek()
        else {
            return Ok(None);
        };
        let value = value.clone();
        let Some(rest) = value.strip_prefix('e').or_else(|| value.strip_prefix('E')) else {
            return Ok(None);
        };
        self.advance();
        if !rest.is_empty() {
            let digits = rest.trim_end_matches(float_suffix_char);
            let suffix = &rest[digits.len()..];
            return (!digits.is_empty()
                && digits.chars().all(|current| current.is_ascii_digit())
                && suffix.chars().all(float_suffix_char))
            .then(|| Some(format!("e{digits}")))
            .ok_or_else(|| CompileError::new("invalid decimal exponent"));
        }
        let sign = match self.peek() {
            Some(Token {
                kind: TokenKind::Punctuator(value),
                ..
            }) if value == "+" || value == "-" => {
                let sign = value.clone();
                self.advance();
                sign
            }
            _ => String::new(),
        };
        let exponent = self.expect_integer()?;
        Ok(Some(format!("e{sign}{exponent}")))
    }

    fn consume_float_suffix(&mut self) {
        let Some(Token {
            kind: TokenKind::Identifier(value),
            ..
        }) = self.peek()
        else {
            return;
        };
        if value.chars().all(float_suffix_char) {
            self.advance();
        }
    }
}

const fn float_suffix_char(value: char) -> bool {
    matches!(value, 'f' | 'F' | 'l' | 'L')
}
