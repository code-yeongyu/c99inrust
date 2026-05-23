mod global;
mod scalar;
mod structs;

pub use global::{
    Constant, Global, GlobalInitializer, GlobalPointerAddress, GlobalStructInitializerAddress,
    GlobalStructInitializerValue, PointerReturnFunction,
};
pub use scalar::{ReturnType, ScalarFieldType, ScalarType};
pub use structs::{FieldType, StructField, StructLayout};
