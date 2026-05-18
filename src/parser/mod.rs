use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub return_expr: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Integer(i64),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    BitNot,
    LogicalNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitXor,
    BitOr,
}

pub fn parse(tokens: &[Token]) -> CompileResult<Program> {
    let mut parser = Parser { tokens, index: 0 };
    parser.program()
}

struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
}

impl Parser<'_> {
    fn program(&mut self) -> CompileResult<Program> {
        let mut functions = Vec::new();
        while !self.check_end() {
            functions.push(self.function()?);
        }
        Ok(Program { functions })
    }

    fn function(&mut self) -> CompileResult<Function> {
        self.expect_keyword(Keyword::Int)?;
        let name = self.expect_identifier()?;
        self.expect_punctuator("(")?;
        if self.check_keyword(Keyword::Void) {
            self.advance();
        }
        self.expect_punctuator(")")?;
        self.expect_punctuator("{")?;
        self.expect_keyword(Keyword::Return)?;
        let return_expr = self.expression()?;
        self.expect_punctuator(";")?;
        self.expect_punctuator("}")?;
        Ok(Function { name, return_expr })
    }

    fn expression(&mut self) -> CompileResult<Expr> {
        self.bit_or()
    }

    fn bit_or(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::bit_xor, &[("|", BinaryOp::BitOr)])
    }

    fn bit_xor(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::bit_and, &[("^", BinaryOp::BitXor)])
    }

    fn bit_and(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::shift, &[("&", BinaryOp::BitAnd)])
    }

    fn shift(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::additive,
            &[("<<", BinaryOp::ShiftLeft), (">>", BinaryOp::ShiftRight)],
        )
    }

    fn additive(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::multiplicative,
            &[("+", BinaryOp::Add), ("-", BinaryOp::Sub)],
        )
    }

    fn multiplicative(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::unary,
            &[
                ("*", BinaryOp::Mul),
                ("/", BinaryOp::Div),
                ("%", BinaryOp::Mod),
            ],
        )
    }

    fn left_associative(
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

    fn unary(&mut self) -> CompileResult<Expr> {
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
        self.primary()
    }

    fn primary(&mut self) -> CompileResult<Expr> {
        if let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::Integer(value) => {
                    let value = *value;
                    self.advance();
                    Ok(Expr::Integer(value))
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

    fn expect_keyword(&mut self, expected: Keyword) -> CompileResult<()> {
        if self.check_keyword(expected.clone()) {
            self.advance();
            return Ok(());
        }
        self.expected(format!("keyword {expected:?}"))
    }

    fn expect_identifier(&mut self) -> CompileResult<String> {
        if let Some(Token {
            kind: TokenKind::Identifier(value),
            ..
        }) = self.peek()
        {
            let value = value.clone();
            self.advance();
            return Ok(value);
        }
        self.expected("identifier".to_string())
    }

    fn expect_punctuator(&mut self, expected: &str) -> CompileResult<()> {
        if self.check_punctuator(expected) {
            self.advance();
            return Ok(());
        }
        self.expected(format!("punctuator {expected}"))
    }

    fn expected<T>(&self, expected: String) -> CompileResult<T> {
        if let Some(token) = self.peek() {
            return Err(
                CompileError::new(format!("expected {expected}")).at(token.line, token.column)
            );
        }
        Err(CompileError::new(format!("expected {expected}")))
    }

    fn check_keyword(&self, expected: Keyword) -> bool {
        matches!(self.peek(), Some(Token { kind: TokenKind::Keyword(value), .. }) if *value == expected)
    }

    fn check_punctuator(&self, expected: &str) -> bool {
        matches!(self.peek(), Some(Token { kind: TokenKind::Punctuator(value), .. }) if value == expected)
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

    fn advance(&mut self) {
        self.index += 1;
    }
}
