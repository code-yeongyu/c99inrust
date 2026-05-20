use super::super::ScalarType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LValue {
    Identifier(String),
    Subscript {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Member {
        base: Box<Expr>,
        field: String,
        dereference: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Call {
        callee: String,
        args: Vec<Self>,
    },
    IndirectCall {
        callee: Box<Self>,
        args: Vec<Self>,
    },
    Identifier(String),
    Integer(i64),
    DoubleLiteral(String),
    StringLiteral(String),
    SizeOfExpr {
        expr: Box<Self>,
    },
    Subscript {
        array: Box<Self>,
        index: Box<Self>,
    },
    Dereference {
        pointer: Box<Self>,
    },
    AddressOf {
        target: LValue,
    },
    Member {
        base: Box<Self>,
        field: String,
        dereference: bool,
    },
    Assignment {
        target: LValue,
        value: Box<Self>,
    },
    PostIncrement {
        target: LValue,
        decrement: bool,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Self>,
    },
    Cast {
        target: ScalarType,
        referent: Option<String>,
        expr: Box<Self>,
    },
    Conditional {
        condition: Box<Self>,
        then_expr: Box<Self>,
        else_expr: Box<Self>,
    },
    Comma {
        left: Box<Self>,
        right: Box<Self>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Self>,
        right: Box<Self>,
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
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    LogicalAnd,
    LogicalOr,
    BitAnd,
    BitXor,
    BitOr,
}
