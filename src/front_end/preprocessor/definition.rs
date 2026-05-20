#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum MacroDefinition {
    Object {
        replacement: String,
    },
    Function {
        params: Vec<String>,
        replacement: String,
    },
}

impl MacroDefinition {
    pub(super) fn condition_value(&self) -> bool {
        match self {
            Self::Object { replacement } if replacement.trim().is_empty() => true,
            Self::Object { replacement } => replacement.trim().parse::<i64>().unwrap_or(0) != 0,
            Self::Function { .. } => true,
        }
    }
}
