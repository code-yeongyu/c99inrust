use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};

use super::super::definition::MacroDefinition;
use super::token::ConditionToken;

pub(super) struct ConditionParser<'a> {
    tokens: Vec<ConditionToken>,
    index: usize,
    macros: &'a HashMap<String, MacroDefinition>,
    line_number: usize,
}

impl<'a> ConditionParser<'a> {
    pub(super) const fn new(
        tokens: Vec<ConditionToken>,
        macros: &'a HashMap<String, MacroDefinition>,
        line_number: usize,
    ) -> Self {
        Self {
            tokens,
            index: 0,
            macros,
            line_number,
        }
    }

    pub(super) fn expression(&mut self) -> CompileResult<bool> {
        self.or()
    }

    fn or(&mut self) -> CompileResult<bool> {
        let mut value = self.and()?;
        while self.matches(&ConditionToken::OrOr) {
            value = value || self.and()?;
        }
        Ok(value)
    }

    fn and(&mut self) -> CompileResult<bool> {
        let mut value = self.equality()?;
        while self.matches(&ConditionToken::AndAnd) {
            value = value && self.equality()?;
        }
        Ok(value)
    }

    fn equality(&mut self) -> CompileResult<bool> {
        let mut value = self.unary()?;
        loop {
            if self.matches(&ConditionToken::EqEq) {
                value = value == self.unary()?;
                continue;
            }
            if self.matches(&ConditionToken::NotEq) {
                value = value != self.unary()?;
                continue;
            }
            return Ok(value);
        }
    }

    fn unary(&mut self) -> CompileResult<bool> {
        if self.matches(&ConditionToken::Bang) {
            return Ok(!self.unary()?);
        }
        if self.matches(&ConditionToken::Defined) {
            return self.defined();
        }
        self.primary()
    }

    fn primary(&mut self) -> CompileResult<bool> {
        match self.peek() {
            ConditionToken::Integer(value) => {
                let value = *value != 0;
                self.index += 1;
                Ok(value)
            }
            ConditionToken::Ident(name) => {
                let value = self
                    .macros
                    .get(name)
                    .is_some_and(MacroDefinition::condition_value);
                self.index += 1;
                Ok(value)
            }
            ConditionToken::LParen => {
                self.index += 1;
                let value = self.expression()?;
                self.expect_token(&ConditionToken::RParen)?;
                Ok(value)
            }
            _ => Err(CompileError::new("expected #if expression").at(self.line_number, 1)),
        }
    }

    fn defined(&mut self) -> CompileResult<bool> {
        if self.matches(&ConditionToken::LParen) {
            let name = self.expect_ident()?;
            self.expect_token(&ConditionToken::RParen)?;
            return Ok(self.macros.contains_key(&name));
        }
        let name = self.expect_ident()?;
        Ok(self.macros.contains_key(&name))
    }

    fn expect_ident(&mut self) -> CompileResult<String> {
        let ConditionToken::Ident(name) = self.peek() else {
            return Err(CompileError::new("expected identifier").at(self.line_number, 1));
        };
        let name = name.clone();
        self.index += 1;
        Ok(name)
    }

    fn expect_token(&mut self, expected: &ConditionToken) -> CompileResult<()> {
        if self.matches(expected) {
            Ok(())
        } else {
            Err(CompileError::new("unexpected #if expression token").at(self.line_number, 1))
        }
    }

    fn matches(&mut self, expected: &ConditionToken) -> bool {
        if self.peek() == expected {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> &ConditionToken {
        &self.tokens[self.index]
    }
}
