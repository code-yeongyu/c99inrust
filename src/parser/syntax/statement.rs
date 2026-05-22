use super::super::{Constant, Global, ScalarType};
use super::expression::{Expr, LValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchCase {
    pub value: Expr,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Empty,
    Block(Vec<Self>),
    Declaration {
        is_static: bool,
        scalar_type: ScalarType,
        name: String,
        referent: Option<String>,
        initializer: Option<Expr>,
    },
    LocalCharArray {
        name: String,
        length: usize,
        is_unsigned: bool,
        initializer: Option<LocalCharArrayInitializer>,
    },
    LocalCharMatrix {
        name: String,
        rows: usize,
        columns: usize,
        initializer: Option<Vec<String>>,
    },
    LocalIntArray {
        name: String,
        length: usize,
        initializer: Option<Vec<i32>>,
    },
    LocalIntMatrix {
        name: String,
        rows: usize,
        columns: usize,
        initializer: Option<Vec<i32>>,
    },
    LocalShortArray {
        name: String,
        length: usize,
        is_unsigned: bool,
    },
    LocalPointerArray {
        name: String,
        length: usize,
        referent: Option<String>,
        initializer: Option<Vec<Expr>>,
    },
    LocalStruct {
        name: String,
        struct_name: String,
        initializer: Option<LocalStructInitializer>,
    },
    LocalStructArray {
        name: String,
        struct_name: String,
        length: usize,
        initializer: Option<Vec<LocalStructInitializerValue>>,
    },
    LocalConstants(Vec<Constant>),
    DeclarationList(Vec<Self>),
    ExpressionList(Vec<Self>),
    ExternGlobal(Global),
    Label(String),
    Goto(String),
    Assignment {
        target: LValue,
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Box<Self>,
        else_branch: Option<Box<Self>>,
    },
    While {
        condition: Expr,
        body: Box<Self>,
    },
    DoWhile {
        body: Box<Self>,
        condition: Expr,
    },
    For {
        initializer: Option<Box<Self>>,
        condition: Option<Expr>,
        post: Option<Box<Self>>,
        body: Box<Self>,
    },
    Switch {
        condition: Expr,
        cases: Vec<SwitchCase>,
        default: Vec<Self>,
        default_position: Option<usize>,
    },
    Expression(Expr),
    Break,
    Continue,
    Return(Option<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalCharArrayInitializer {
    StringLiteral(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalStructInitializer {
    Values(Vec<LocalStructInitializerValue>),
    Copy(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalStructInitializerValue {
    Expr(Expr),
    Nested(Vec<Self>),
}
