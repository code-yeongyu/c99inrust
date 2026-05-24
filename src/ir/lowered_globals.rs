use crate::parser::ScalarType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredGlobal {
    pub name: String,
    pub is_static: bool,
    pub initializer: LoweredGlobalInitializer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredGlobalInitializer {
    Int(i32),
    LongLong(i64),
    Double(String),
    RealThenZero {
        real: String,
        byte_len: usize,
    },
    RealArray {
        scalar_type: ScalarType,
        length: usize,
        values: Vec<String>,
    },
    LongLongArray(Vec<i64>),
    IntArray(Vec<i32>),
    ShortArray(Vec<i32>),
    PointerNull,
    PointerString(String, usize),
    PointerGlobalOffset {
        base: String,
        byte_offset: usize,
    },
    PointerArray(usize),
    PointerStringArray {
        values: Vec<Option<(String, usize)>>,
        length: usize,
    },
    PointerNameArray {
        values: Vec<Option<(String, usize)>>,
        length: usize,
    },
    StructArray {
        byte_len: usize,
        values: Vec<LoweredStructInitializerValue>,
    },
    ZeroBytes(usize),
    UnsignedCharArray(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredStructInitializerValue {
    pub byte_offset: usize,
    pub value: LoweredStructInitializerScalar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredStructInitializerScalar {
    Int {
        value: i32,
        byte_size: usize,
    },
    IntString {
        value: String,
        byte_size: usize,
        byte_offset: usize,
    },
    LongLong(i64),
    Bytes {
        values: Vec<u8>,
        byte_len: usize,
    },
    PointerNull,
    PointerInteger(i64),
    PointerString(String, usize),
    PointerGlobalOffset {
        base: String,
        byte_offset: usize,
    },
}
