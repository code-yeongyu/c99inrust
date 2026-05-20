use super::{CompileError, CompileResult, Keyword, Parser, Token, TokenKind};

impl Parser<'_> {
    pub(super) fn expect_keyword(&mut self, expected: Keyword) -> CompileResult<()> {
        if self.check_keyword(expected) {
            self.advance();
            return Ok(());
        }
        self.expected(&format!("keyword {expected:?}"))
    }

    pub(super) fn expect_identifier(&mut self) -> CompileResult<String> {
        if let Some(Token {
            kind: TokenKind::Identifier(value),
            ..
        }) = self.peek()
        {
            let value = value.clone();
            self.advance();
            return Ok(value);
        }
        self.expected("identifier")
    }

    pub(super) fn expect_integer(&mut self) -> CompileResult<i64> {
        if let Some(value) = self.peek().and_then(|token| token.kind.integer_value()) {
            self.advance();
            return Ok(value);
        }
        self.expected("integer")
    }

    pub(super) fn expect_punctuator(&mut self, expected: &str) -> CompileResult<()> {
        if self.check_punctuator(expected) {
            self.advance();
            return Ok(());
        }
        self.expected(&format!("punctuator {expected}"))
    }

    pub(super) fn expected<T>(&self, expected: &str) -> CompileResult<T> {
        if let Some(token) = self.peek() {
            return Err(
                CompileError::new(format!("expected {expected}")).at(token.line, token.column)
            );
        }
        Err(CompileError::new(format!("expected {expected}")))
    }

    pub(super) fn check_keyword(&self, expected: Keyword) -> bool {
        matches!(self.peek(), Some(Token { kind: TokenKind::Keyword(value), .. }) if *value == expected)
    }

    pub(super) fn check_punctuator(&self, expected: &str) -> bool {
        matches!(self.peek(), Some(Token { kind: TokenKind::Punctuator(value), .. }) if value == expected)
    }

    pub(super) fn check_end(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token {
                kind: TokenKind::End,
                ..
            }) | None
        )
    }

    pub(super) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    pub(super) const fn advance(&mut self) {
        self.index += 1;
    }
}
