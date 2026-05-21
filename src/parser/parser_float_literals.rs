use super::{CompileError, CompileResult, Parser, Token, TokenKind};

impl Parser<'_> {
    pub(super) fn decimal_exponent_suffix(&mut self) -> CompileResult<String> {
        let exponent = self.optional_decimal_exponent_suffix()?.unwrap_or_default();
        self.consume_float_suffix();
        Ok(exponent)
    }

    pub(super) fn optional_decimal_exponent_suffix(&mut self) -> CompileResult<Option<String>> {
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
        self.consume_float_suffix();
        Ok(Some(format!("e{sign}{exponent}")))
    }

    pub(super) fn consume_float_suffix(&mut self) {
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
