use super::LoweredExpr;
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{FieldType, ScalarType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::ir) enum LocalBinding {
    Scalar {
        slot: usize,
        scalar_type: ScalarType,
        referent: Option<String>,
    },
    StaticScalar {
        global_name: String,
        scalar_type: ScalarType,
        referent: Option<String>,
    },
    CharArray {
        slot: usize,
        length: usize,
        is_unsigned: bool,
    },
    CharMatrix {
        slot: usize,
        rows: usize,
        columns: usize,
    },
    IntArray {
        slot: usize,
        length: usize,
    },
    IntMatrix {
        slot: usize,
        rows: usize,
        columns: usize,
    },
    ShortArray {
        slot: usize,
        length: usize,
        is_unsigned: bool,
    },
    ScalarArray {
        slot: usize,
        scalar_type: ScalarType,
        length: usize,
    },
    PointerArray {
        slot: usize,
        length: usize,
        referent: Option<String>,
    },
    StructObject {
        slot: usize,
        struct_name: String,
        byte_size: usize,
    },
    StructArray {
        slot: usize,
        struct_name: String,
        byte_size: usize,
        length: usize,
    },
    VaList {
        slot: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::ir) enum GlobalBinding {
    Int,
    LongLong,
    Scalar(ScalarType),
    IntArray,
    ShortArray {
        is_unsigned: bool,
        columns: Option<usize>,
    },
    IntMatrix {
        columns: usize,
    },
    DoubleArray,
    ScalarArray {
        scalar_type: ScalarType,
        length: Option<usize>,
    },
    Pointer {
        referent: Option<String>,
    },
    PointerArray {
        referent: Option<String>,
        length: Option<usize>,
        columns: Option<usize>,
    },
    StructObject {
        struct_name: String,
        byte_size: usize,
    },
    StructArray {
        struct_name: String,
        byte_size: usize,
        length: Option<usize>,
        columns: Option<usize>,
    },
    UnsignedCharArray {
        is_unsigned: bool,
    },
    UnsignedCharMatrix {
        columns: usize,
        is_unsigned: bool,
    },
}

impl GlobalBinding {
    pub(in crate::ir) fn from_scalar_type(scalar_type: ScalarType) -> CompileResult<Self> {
        match scalar_type {
            ScalarType::Bool => Ok(Self::Scalar(ScalarType::Bool)),
            ScalarType::Int => Ok(Self::Int),
            ScalarType::LongLong => Ok(Self::LongLong),
            ScalarType::Pointer => Ok(Self::Pointer { referent: None }),
            ScalarType::ComplexFloat
            | ScalarType::ComplexDouble
            | ScalarType::ComplexLongDouble
            | ScalarType::Double
            | ScalarType::LongDouble => Ok(Self::Scalar(scalar_type)),
            ScalarType::VaList => Err(CompileError::new("unsupported extern global scalar type")),
        }
    }

    pub(in crate::ir) const fn scalar_type(&self) -> Option<ScalarType> {
        match self {
            Self::Int => Some(ScalarType::Int),
            Self::LongLong => Some(ScalarType::LongLong),
            Self::Scalar(scalar_type) => Some(*scalar_type),
            Self::Pointer { .. } => Some(ScalarType::Pointer),
            Self::IntArray
            | Self::ShortArray { .. }
            | Self::IntMatrix { .. }
            | Self::DoubleArray
            | Self::ScalarArray { .. }
            | Self::PointerArray { .. }
            | Self::StructObject { .. }
            | Self::StructArray { .. }
            | Self::UnsignedCharArray { .. }
            | Self::UnsignedCharMatrix { .. } => None,
        }
    }

    pub(in crate::ir) const fn is_addressable_array(&self) -> bool {
        matches!(
            self,
            Self::IntArray
                | Self::ShortArray { .. }
                | Self::IntMatrix { .. }
                | Self::DoubleArray
                | Self::ScalarArray { .. }
                | Self::PointerArray { .. }
                | Self::StructArray { .. }
                | Self::UnsignedCharArray { .. }
                | Self::UnsignedCharMatrix { .. }
        )
    }
}

pub(in crate::ir) struct ResolvedMember {
    pub(in crate::ir) pointer: LoweredExpr,
    pub(in crate::ir) offset: usize,
    pub(in crate::ir) field_type: FieldType,
}

pub(in crate::ir) struct StructAddress {
    pub(in crate::ir) pointer: LoweredExpr,
    pub(in crate::ir) offset: usize,
    pub(in crate::ir) struct_name: String,
}

pub(in crate::ir) type ArrayFieldSubscript = (LoweredExpr, ScalarType, usize, bool);
pub(in crate::ir) type NestedArrayFieldSubscript =
    (LoweredExpr, LoweredExpr, ScalarType, usize, bool);
