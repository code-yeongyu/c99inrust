use super::ScalarType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constant {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Global {
    pub name: String,
    pub is_static: bool,
    pub initializer: GlobalInitializer,
}

impl Global {
    pub(in crate::parser) const fn new(name: String, initializer: GlobalInitializer) -> Self {
        Self {
            name,
            is_static: false,
            initializer,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerReturnFunction {
    pub name: String,
    pub referent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPointerAddress {
    pub base: String,
    pub index: usize,
    pub fields: Vec<String>,
    pub element_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalInitializer {
    Extern(ScalarType),
    ExternPointer {
        referent: Option<String>,
    },
    ExternIntArray,
    ExternShortArray {
        is_unsigned: bool,
        columns: Option<usize>,
    },
    ExternPointerArray {
        referent: Option<String>,
        columns: Option<usize>,
    },
    ExternUnsignedCharArray {
        is_unsigned: bool,
    },
    ExternUnsignedCharMatrix {
        columns: usize,
        is_unsigned: bool,
    },
    ExternStructArray {
        struct_name: String,
    },
    ExternStructObject {
        struct_name: String,
    },
    Int(i64),
    LongLong(i64),
    Double(String),
    ComplexReal {
        scalar_type: ScalarType,
        real: String,
    },
    ScalarZero(ScalarType),
    IntArray(Vec<i32>),
    ShortArray {
        values: Vec<i32>,
        is_unsigned: bool,
        columns: Option<usize>,
    },
    IntMatrix {
        values: Vec<i32>,
        columns: usize,
    },
    IntConstant(String),
    DoubleArray {
        length: usize,
    },
    ScalarArray {
        scalar_type: ScalarType,
        length: usize,
    },
    PointerNull {
        referent: Option<String>,
    },
    PointerString {
        referent: Option<String>,
        value: String,
        byte_offset: usize,
    },
    PointerName {
        referent: Option<String>,
        value: String,
    },
    PointerSubscriptAddress {
        referent: Option<String>,
        base: String,
        index: usize,
    },
    PointerMemberAddress {
        referent: Option<String>,
        address: GlobalPointerAddress,
    },
    PointerArray {
        referent: Option<String>,
        length: usize,
        columns: Option<usize>,
    },
    PointerStringArray {
        referent: Option<String>,
        values: Vec<Option<(String, usize)>>,
        length: usize,
    },
    PointerNameArray {
        referent: Option<String>,
        values: Vec<Option<GlobalPointerAddress>>,
        length: usize,
    },
    StructObject {
        struct_name: String,
        values: Vec<GlobalStructInitializerValue>,
    },
    StructArray {
        struct_name: String,
        length: usize,
        columns: Option<usize>,
        values: Vec<Vec<GlobalStructInitializerValue>>,
    },
    UnsignedCharArray {
        values: Vec<u8>,
        is_unsigned: bool,
    },
    UnsignedCharMatrix {
        values: Vec<u8>,
        columns: usize,
        is_unsigned: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalStructInitializerValue {
    Integer(i64),
    String(String),
    StringPointer {
        value: String,
        byte_offset: usize,
        cast_target: Option<ScalarType>,
    },
    Address(GlobalStructInitializerAddress),
    Nested(Vec<Self>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalStructInitializerAddress {
    pub base: String,
    pub index: Option<usize>,
}

impl GlobalInitializer {
    pub(in crate::parser) const fn is_extern(&self) -> bool {
        matches!(
            self,
            Self::Extern(_)
                | Self::ExternPointer { .. }
                | Self::ExternIntArray
                | Self::ExternShortArray { .. }
                | Self::ExternPointerArray { .. }
                | Self::ExternUnsignedCharArray { .. }
                | Self::ExternUnsignedCharMatrix { .. }
                | Self::ExternStructArray { .. }
                | Self::ExternStructObject { .. }
        )
    }
}
