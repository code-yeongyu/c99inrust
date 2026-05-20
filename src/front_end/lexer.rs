use crate::diagnostics::{CompileError, CompileResult};

use super::integer_literal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Auto,
    Bool,
    Break,
    Case,
    Char,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    Float,
    For,
    Goto,
    If,
    Inline,
    Int,
    Long,
    Register,
    Restrict,
    Return,
    Short,
    Signed,
    Sizeof,
    Static,
    Struct,
    Switch,
    Typedef,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    Integer(i64),
    StringLiteral(String),
    CharLiteral(char),
    Keyword(Keyword),
    Punctuator(String),
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

/// Lexes C source into frontend tokens.
///
/// # Errors
///
/// Returns an error when a literal, punctuator, or block comment is malformed
/// for the supported C surface.
pub fn lex(source: &str) -> CompileResult<Vec<Token>> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token()?;
        let is_end = token.kind == TokenKind::End;
        tokens.push(token);
        if is_end {
            return Ok(tokens);
        }
    }
}

struct Lexer {
    input: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
        }
    }

    fn next_token(&mut self) -> CompileResult<Token> {
        self.skip_ignored()?;
        let line = self.line;
        let column = self.column;
        let Some(current) = self.current() else {
            return Ok(Token {
                kind: TokenKind::End,
                line,
                column,
            });
        };
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

    fn skip_ignored(&mut self) -> CompileResult<()> {
        loop {
            while self.current().is_some_and(char::is_whitespace) {
                self.advance();
            }
            if self.current() == Some('/') && self.peek() == Some('/') {
                while self.current().is_some_and(|value| value != '\n') {
                    self.advance();
                }
                continue;
            }
            if self.current() == Some('/') && self.peek() == Some('*') {
                let line = self.line;
                let column = self.column;
                self.advance();
                self.advance();
                let mut closed = false;
                while self.current().is_some() {
                    if self.current() == Some('*') && self.peek() == Some('/') {
                        self.advance();
                        self.advance();
                        closed = true;
                        break;
                    }
                    self.advance();
                }
                if !closed {
                    return Err(CompileError::new("unterminated block comment").at(line, column));
                }
                continue;
            }
            return Ok(());
        }
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
        match value.as_str() {
            "_Bool" => TokenKind::Keyword(Keyword::Bool),
            "auto" => TokenKind::Keyword(Keyword::Auto),
            "break" => TokenKind::Keyword(Keyword::Break),
            "case" => TokenKind::Keyword(Keyword::Case),
            "char" => TokenKind::Keyword(Keyword::Char),
            "const" => TokenKind::Keyword(Keyword::Const),
            "continue" => TokenKind::Keyword(Keyword::Continue),
            "default" => TokenKind::Keyword(Keyword::Default),
            "do" => TokenKind::Keyword(Keyword::Do),
            "double" => TokenKind::Keyword(Keyword::Double),
            "else" => TokenKind::Keyword(Keyword::Else),
            "enum" => TokenKind::Keyword(Keyword::Enum),
            "extern" => TokenKind::Keyword(Keyword::Extern),
            "float" => TokenKind::Keyword(Keyword::Float),
            "for" => TokenKind::Keyword(Keyword::For),
            "goto" => TokenKind::Keyword(Keyword::Goto),
            "if" => TokenKind::Keyword(Keyword::If),
            "inline" => TokenKind::Keyword(Keyword::Inline),
            "int" => TokenKind::Keyword(Keyword::Int),
            "long" => TokenKind::Keyword(Keyword::Long),
            "register" => TokenKind::Keyword(Keyword::Register),
            "restrict" => TokenKind::Keyword(Keyword::Restrict),
            "return" => TokenKind::Keyword(Keyword::Return),
            "short" => TokenKind::Keyword(Keyword::Short),
            "signed" => TokenKind::Keyword(Keyword::Signed),
            "sizeof" => TokenKind::Keyword(Keyword::Sizeof),
            "static" => TokenKind::Keyword(Keyword::Static),
            "struct" => TokenKind::Keyword(Keyword::Struct),
            "switch" => TokenKind::Keyword(Keyword::Switch),
            "typedef" => TokenKind::Keyword(Keyword::Typedef),
            "union" => TokenKind::Keyword(Keyword::Union),
            "unsigned" => TokenKind::Keyword(Keyword::Unsigned),
            "void" => TokenKind::Keyword(Keyword::Void),
            "volatile" => TokenKind::Keyword(Keyword::Volatile),
            "while" => TokenKind::Keyword(Keyword::While),
            _ => TokenKind::Identifier(value),
        }
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
        const PUNCTUATORS: &[&str] = &[
            ">>=", "<<=", "...", "++", "--", "->", "<<", ">>", "<=", ">=", "==", "!=", "&&", "||",
            "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "##", "{", "}", "(", ")", "[", "]",
            ";", ",", ".", "&", "*", "+", "-", "~", "!", "/", "%", "<", ">", "^", "|", "?", ":",
            "=", "#",
        ];
        for candidate in PUNCTUATORS {
            if self.starts_with(candidate) {
                for _ in candidate.chars() {
                    self.advance();
                }
                return Ok((*candidate).to_string());
            }
        }
        Err(CompileError::new("unexpected character").at(self.line, self.column))
    }

    fn starts_with(&self, expected: &str) -> bool {
        for (offset, expected_char) in expected.chars().enumerate() {
            if self.input.get(self.index + offset).copied() != Some(expected_char) {
                return false;
            }
        }
        true
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.index).copied()
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.index + 1).copied()
    }

    fn advance(&mut self) {
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
