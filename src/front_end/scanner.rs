use crate::diagnostics::{CompileError, CompileResult};

use super::token::{self, Token, TokenKind};
use super::{ignored, integer_literal, punctuator};

pub(super) struct Scanner {
    pub(super) input: Vec<char>,
    pub(super) index: usize,
    pub(super) line: usize,
    pub(super) column: usize,
}

impl Scanner {
    pub(super) fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
        }
    }

    pub(super) fn next_token(&mut self) -> CompileResult<Token> {
        ignored::skip(self)?;
        let line = self.line;
        let column = self.column;
        let Some(current) = self.current() else {
            return Ok(Token {
                kind: TokenKind::End,
                line,
                column,
            });
        };
        if current == 'L' && self.peek() == Some('"') {
            self.advance();
            return Ok(Token {
                kind: TokenKind::StringLiteral(self.string_literal()?),
                line,
                column,
            });
        }
        if current == 'L' && self.peek() == Some('\'') {
            self.advance();
            return Ok(Token {
                kind: TokenKind::CharLiteral(self.char_literal()?),
                line,
                column,
            });
        }
        if current.is_ascii_alphabetic() || current == '_' {
            return Ok(Token {
                kind: self.identifier_or_keyword(),
                line,
                column,
            });
        }
        if current.is_ascii_digit() {
            return Ok(Token {
                kind: self.integer()?,
                line,
                column,
            });
        }
        if current == '"' {
            return Ok(Token {
                kind: TokenKind::StringLiteral(self.string_literal()?),
                line,
                column,
            });
        }
        if current == '\'' {
            return Ok(Token {
                kind: TokenKind::CharLiteral(self.char_literal()?),
                line,
                column,
            });
        }
        Ok(Token {
            kind: TokenKind::Punctuator(self.punctuator()?),
            line,
            column,
        })
    }

    fn identifier_or_keyword(&mut self) -> TokenKind {
        let mut value = String::new();
        while self
            .current()
            .is_some_and(|current| current.is_ascii_alphanumeric() || current == '_')
        {
            if let Some(current) = self.current() {
                value.push(current);
            }
            self.advance();
        }
        token::identifier_or_keyword_kind(value)
    }

    fn integer(&mut self) -> CompileResult<TokenKind> {
        let line = self.line;
        let column = self.column;
        let mut value = String::new();
        if self.current() == Some('0') && matches!(self.peek(), Some('x' | 'X')) {
            self.advance();
            self.advance();
            while self
                .current()
                .is_some_and(|current| current.is_ascii_hexdigit())
            {
                if let Some(current) = self.current() {
                    value.push(current);
                }
                self.advance();
            }
            if value.is_empty() {
                return Err(CompileError::new("expected hexadecimal digits").at(line, column));
            }
            self.consume_integer_suffix();
            let parsed = integer_literal::parse_hexadecimal(&value, line, column)?;
            return Ok(TokenKind::Integer(parsed));
        }
        while self
            .current()
            .is_some_and(|current| current.is_ascii_digit())
        {
            if let Some(current) = self.current() {
                value.push(current);
            }
            self.advance();
        }
        self.consume_integer_suffix();
        let parsed = integer_literal::parse_decimal_or_octal(&value, line, column)?;
        Ok(TokenKind::Integer(parsed))
    }

    fn consume_integer_suffix(&mut self) {
        while self
            .current()
            .is_some_and(|current| matches!(current, 'u' | 'U' | 'l' | 'L'))
        {
            self.advance();
        }
    }

    fn string_literal(&mut self) -> CompileResult<String> {
        let line = self.line;
        let column = self.column;
        self.advance();
        let mut value = String::new();
        while let Some(current) = self.current() {
            if current == '"' {
                self.advance();
                return Ok(value);
            }
            if current == '\\' {
                self.advance();
                value.push(self.escape_sequence(line, column)?);
                continue;
            }
            value.push(current);
            self.advance();
        }
        Err(CompileError::new("unterminated string literal").at(line, column))
    }

    fn char_literal(&mut self) -> CompileResult<char> {
        let line = self.line;
        let column = self.column;
        self.advance();
        let value = if self.current() == Some('\\') {
            self.advance();
            self.escape_sequence(line, column)?
        } else {
            let Some(current) = self.current() else {
                return Err(CompileError::new("unterminated character literal").at(line, column));
            };
            self.advance();
            current
        };
        if self.current() != Some('\'') {
            return Err(CompileError::new("expected closing character quote").at(line, column));
        }
        self.advance();
        Ok(value)
    }

    fn escape_sequence(&mut self, line: usize, column: usize) -> CompileResult<char> {
        let Some(current) = self.current() else {
            return Err(CompileError::new("unterminated escape sequence").at(line, column));
        };
        let escaped = match current {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0' => '\0',
            '\\' => '\\',
            '\'' => '\'',
            '"' => '"',
            other => other,
        };
        self.advance();
        Ok(escaped)
    }

    fn punctuator(&mut self) -> CompileResult<String> {
        if let Some(candidate) = punctuator::first_match(&self.input, self.index) {
            for _ in candidate.chars() {
                self.advance();
            }
            return Ok(candidate.to_string());
        }
        Err(CompileError::new("unexpected character").at(self.line, self.column))
    }

    pub(super) fn current(&self) -> Option<char> {
        self.input.get(self.index).copied()
    }

    pub(super) fn peek(&self) -> Option<char> {
        self.input.get(self.index + 1).copied()
    }

    pub(super) fn advance(&mut self) {
        if let Some(current) = self.current() {
            self.index += 1;
            if current == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }
}
