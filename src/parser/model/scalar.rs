#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnType {
    Int,
    Pointer,
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    Bool,
    Int,
    LongLong,
    ComplexFloat,
    ComplexDouble,
    ComplexLongDouble,
    Double,
    LongDouble,
    Pointer,
    VaList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScalarFieldType {
    pub scalar_type: ScalarType,
    pub byte_size: usize,
    pub is_unsigned: bool,
}
