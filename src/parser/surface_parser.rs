use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Token, TokenKind};

use super::SurfaceTranslationUnit;
use super::external_declarations::classify_external_item;
use super::token_scan::{decrease_depth, last_token_is_punctuator, token_is_punctuator};

pub(super) struct SurfaceParser<'a> {
    tokens: &'a [Token],
    index: usize,
}

impl<'a> SurfaceParser<'a> {
    pub(super) const fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, index: 0 }
    }
}

impl SurfaceParser<'_> {
    pub(super) fn translation_unit(&mut self) -> CompileResult<SurfaceTranslationUnit> {
        let external_items = self.external_token_groups()?;
        let mut items = Vec::new();
        for external_tokens in external_items {
            if let Some(item) = classify_external_item(&external_tokens) {
                items.push(item);
            }
        }
        Ok(SurfaceTranslationUnit { items })
    }

    pub(super) fn external_token_groups(&mut self) -> CompileResult<Vec<Vec<Token>>> {
        let mut items = Vec::new();
        while !self.check_end() {
            if self.check_punctuator("#") {
                self.skip_directive();
                continue;
            }
            if self.check_punctuator(";") {
                self.advance();
                continue;
            }
            let external_tokens = self.collect_external_item()?;
            items.push(external_tokens);
        }
        Ok(items)
    }

    fn collect_external_item(&mut self) -> CompileResult<Vec<Token>> {
        let start = self.index;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut function_body = false;
        while !self.check_end() {
            let Some(token) = self.peek() else {
                break;
            };
            let is_top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if is_top_level && token_is_punctuator(token, ";") {
                self.advance();
                return Ok(self.tokens[start..self.index].to_vec());
            }
            match &token.kind {
                TokenKind::Punctuator(value) if value == "(" => {
                    paren_depth += 1;
                }
                TokenKind::Punctuator(value) if value == ")" => {
                    decrease_depth(&mut paren_depth, token, "parenthesis")?;
                }
                TokenKind::Punctuator(value) if value == "[" => {
                    bracket_depth += 1;
                }
                TokenKind::Punctuator(value) if value == "]" => {
                    decrease_depth(&mut bracket_depth, token, "bracket")?;
                }
                TokenKind::Punctuator(value) if value == "{" => {
                    if is_top_level
                        && last_token_is_punctuator(&self.tokens[start..self.index], ")")
                    {
                        function_body = true;
                    }
                    brace_depth += 1;
                }
                TokenKind::Punctuator(value) if value == "}" => {
                    decrease_depth(&mut brace_depth, token, "brace")?;
                    self.advance();
                    if function_body && brace_depth == 0 {
                        return Ok(self.tokens[start..self.index].to_vec());
                    }
                    continue;
                }
                _ => {}
            }
            self.advance();
        }
        if paren_depth != 0 || bracket_depth != 0 || brace_depth != 0 {
            let error = CompileError::new("unterminated external declaration");
            let Some(token) = self.tokens.get(start) else {
                return Err(error);
            };
            return Err(error.at(token.line, token.column));
        }
        Ok(self.tokens[start..self.index].to_vec())
    }

    fn skip_directive(&mut self) {
        let Some(start_line) = self.peek().map(|token| token.line) else {
            return;
        };
        while let Some(token) = self.peek() {
            if matches!(token.kind, TokenKind::End) || token.line != start_line {
                return;
            }
            self.advance();
        }
    }

    fn check_punctuator(&self, expected: &str) -> bool {
        matches!(self.peek(), Some(token) if token_is_punctuator(token, expected))
    }

    fn check_end(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token {
                kind: TokenKind::End,
                ..
            }) | None
        )
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    const fn advance(&mut self) {
        self.index += 1;
    }
}
