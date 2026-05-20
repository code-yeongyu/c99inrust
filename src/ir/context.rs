use super::{GlobalBinding, Instruction, LocalBinding, LocalSlot, LoweredGlobal};
use crate::parser::{ReturnType, StructLayout};
use std::collections::{HashMap, HashSet};

pub(in crate::ir) struct LoweringContext {
    pub(in crate::ir) function_name: String,
    pub(in crate::ir) return_type: ReturnType,
    pub(in crate::ir) structs: HashMap<String, StructLayout>,
    pub(in crate::ir) global_bindings: HashMap<String, GlobalBinding>,
    pub(in crate::ir) static_globals: Vec<LoweredGlobal>,
    pub(in crate::ir) constants: HashMap<String, i64>,
    pub(in crate::ir) pointer_return_functions: HashMap<String, Option<String>>,
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

impl LoweringContext {
    pub(in crate::ir) fn new(
        function_name: &str,
        return_type: ReturnType,
        structs: &HashMap<String, StructLayout>,
        global_bindings: &HashMap<String, GlobalBinding>,
        constants: &HashMap<String, i64>,
        pointer_return_functions: &HashMap<String, Option<String>>,
        function_names: &HashSet<String>,
    ) -> Self {
        Self {
            function_name: function_name.to_owned(),
            return_type,
            structs: structs.clone(),
            global_bindings: global_bindings.clone(),
            static_globals: Vec::new(),
            constants: constants.clone(),
            pointer_return_functions: pointer_return_functions.clone(),
            function_names: function_names.clone(),
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
