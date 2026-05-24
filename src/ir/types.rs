use crate::parser::{BinaryOp, ReturnType, ScalarType, UnaryOp};

use super::lowered_globals::LoweredGlobal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredProgram {
    pub globals: Vec<LoweredGlobal>,
    pub functions: Vec<LoweredFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredFunction {
    pub name: String,
    pub return_type: ReturnType,
    pub parameter_count: usize,
    pub is_variadic: bool,
    pub variadic_save_slot: Option<usize>,
    pub local_slots: Vec<LocalSlot>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSlot {
    pub offset: usize,
    pub scalar_type: ScalarType,
    pub byte_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    StoreLocal {
        slot: usize,
        offset: usize,
        scalar_type: ScalarType,
        value: LoweredExpr,
    },
    InitLocalBytes {
        offset: usize,
        values: Vec<u8>,
    },
    InitLocalInts {
        offset: usize,
        values: Vec<i32>,
    },
    StoreGlobal {
        name: String,
        scalar_type: ScalarType,
        value: LoweredExpr,
    },
    JumpIfZero {
        condition: LoweredExpr,
        label: usize,
    },
    Jump {
        label: usize,
    },
    Label {
        label: usize,
    },
    Eval(LoweredExpr),
    Return(Option<LoweredExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredExpr {
    Call {
        callee: String,
        args: Vec<Self>,
        return_type: ScalarType,
    },
    IndirectCall {
        callee: Box<Self>,
        args: Vec<Self>,
        return_type: ScalarType,
    },
    Integer(i64),
    LongInteger(i64),
    DoubleLiteral(String),
    StringLiteral(String),
    VaArg {
        list: Box<Self>,
        scalar_type: ScalarType,
    },
    Global {
        name: String,
        scalar_type: ScalarType,
    },
    GlobalByteSubscript {
        name: String,
        index: Box<Self>,
        is_unsigned: bool,
    },
    GlobalIntSubscript {
        name: String,
        index: Box<Self>,
    },
    GlobalPointerSubscript {
        name: String,
        index: Box<Self>,
    },
    GlobalAddress {
        name: String,
    },
    PointerSubscript {
        pointer: Box<Self>,
        index: Box<Self>,
        element_type: ScalarType,
        element_byte_size: usize,
        element_unsigned: bool,
    },
    PointerOffset {
        pointer: Box<Self>,
        index: Box<Self>,
        byte_size: usize,
    },
    PointerField {
        pointer: Box<Self>,
        offset: usize,
        scalar_type: ScalarType,
        byte_size: usize,
        is_unsigned: bool,
    },
    PointerFieldAddress {
        pointer: Box<Self>,
        offset: usize,
    },
    Assign {
        target: LoweredLValue,
        value: Box<Self>,
    },
    PostIncrement {
        target: LoweredLValue,
        increment: i64,
    },
    Local {
        offset: usize,
        scalar_type: ScalarType,
    },
    LocalAddress {
        offset: usize,
        byte_size: usize,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Self>,
    },
    Cast {
        target: ScalarType,
        expr: Box<Self>,
    },
    Conditional {
        condition: Box<Self>,
        then_expr: Box<Self>,
        else_expr: Box<Self>,
    },
    IndexSelect {
        index: Box<Self>,
        cases: Vec<Self>,
        default: Box<Self>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredLValue {
    Local {
        slot: usize,
        offset: usize,
        scalar_type: ScalarType,
        referent: Option<String>,
    },
    Global {
        name: String,
        scalar_type: ScalarType,
    },
    GlobalByteSubscript {
        name: String,
        index: Box<LoweredExpr>,
        is_unsigned: bool,
    },
    GlobalIntSubscript {
        name: String,
        index: Box<LoweredExpr>,
    },
    GlobalPointerSubscript {
        name: String,
        index: Box<LoweredExpr>,
    },
    PointerSubscript {
        pointer: Box<LoweredExpr>,
        index: Box<LoweredExpr>,
        element_type: ScalarType,
        element_byte_size: usize,
        element_unsigned: bool,
    },
    PointerField {
        pointer: Box<LoweredExpr>,
        offset: usize,
        scalar_type: ScalarType,
        byte_size: usize,
        is_unsigned: bool,
    },
}
