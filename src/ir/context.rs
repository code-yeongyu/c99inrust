use super::{GlobalBinding, Instruction, LocalBinding, LocalSlot, LoweredGlobal};
use crate::parser::{ReturnType, ScalarType, StructLayout};
use std::collections::{HashMap, HashSet};

pub(in crate::ir) struct LoweringContext {
    pub(in crate::ir) function_name: String,
    pub(in crate::ir) return_type: ReturnType,
    pub(in crate::ir) structs: HashMap<String, StructLayout>,
    pub(in crate::ir) global_bindings: HashMap<String, GlobalBinding>,
    pub(in crate::ir) static_globals: Vec<LoweredGlobal>,
    pub(in crate::ir) constants: HashMap<String, i64>,
    pub(in crate::ir) pointer_return_functions: HashMap<String, Option<String>>,
    pub(in crate::ir) function_return_types: HashMap<String, ScalarType>,
    pub(in crate::ir) function_names: HashSet<String>,
    pub(in crate::ir) scopes: Vec<HashMap<String, LocalBinding>>,
    pub(in crate::ir) local_slots: Vec<LocalSlot>,
    pub(in crate::ir) next_local_offset: usize,
    pub(in crate::ir) instructions: Vec<Instruction>,
    pub(in crate::ir) next_label: usize,
    pub(in crate::ir) named_labels: HashMap<String, usize>,
    pub(in crate::ir) break_labels: Vec<usize>,
    pub(in crate::ir) continue_labels: Vec<usize>,
    pub(in crate::ir) has_return: bool,
}

#[derive(Clone, Copy)]
pub(in crate::ir) struct LoweringContextInputs<'a> {
    pub(in crate::ir) structs: &'a HashMap<String, StructLayout>,
    pub(in crate::ir) global_bindings: &'a HashMap<String, GlobalBinding>,
    pub(in crate::ir) constants: &'a HashMap<String, i64>,
    pub(in crate::ir) pointer_return_functions: &'a HashMap<String, Option<String>>,
    pub(in crate::ir) function_return_types: &'a HashMap<String, ScalarType>,
    pub(in crate::ir) function_names: &'a HashSet<String>,
}

impl LoweringContext {
    pub(in crate::ir) fn new(
        function_name: &str,
        return_type: ReturnType,
        inputs: LoweringContextInputs<'_>,
    ) -> Self {
        Self {
            function_name: function_name.to_owned(),
            return_type,
            structs: inputs.structs.clone(),
            global_bindings: inputs.global_bindings.clone(),
            static_globals: Vec::new(),
            constants: inputs.constants.clone(),
            pointer_return_functions: inputs.pointer_return_functions.clone(),
            function_return_types: inputs.function_return_types.clone(),
            function_names: inputs.function_names.clone(),
            scopes: vec![HashMap::new()],
            local_slots: Vec::new(),
            next_local_offset: 0,
            instructions: Vec::new(),
            next_label: 0,
            named_labels: HashMap::new(),
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
            has_return: false,
        }
    }
}
