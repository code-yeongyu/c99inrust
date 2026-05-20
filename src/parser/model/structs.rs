use super::{ScalarFieldType, ScalarType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLayout {
    pub name: String,
    pub fields: Vec<StructField>,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub field_type: FieldType,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Scalar(ScalarFieldType),
    Struct(String),
    Pointer {
        referent: Option<String>,
    },
    Array {
        element_type: ScalarType,
        element_size: usize,
        element_unsigned: bool,
        length: usize,
        columns: Option<usize>,
    },
    StructArray {
        struct_name: String,
        length: usize,
    },
}
