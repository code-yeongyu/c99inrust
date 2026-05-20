use super::{
    Constant, Global, PointerReturnFunction, ReturnType, ScalarType, Statement, StructLayout,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub structs: Vec<StructLayout>,
    pub constants: Vec<Constant>,
    pub globals: Vec<Global>,
    pub pointer_return_functions: Vec<PointerReturnFunction>,
    pub function_prototypes: Vec<String>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub return_type: ReturnType,
    pub parameters: Vec<Parameter>,
    pub is_variadic: bool,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub scalar_type: ScalarType,
    pub referent: Option<String>,
}
