#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceTranslationUnit {
    pub items: Vec<ExternalItem>,
}

impl SurfaceTranslationUnit {
    #[must_use]
    pub fn typedef_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, ExternalItem::Typedef { .. }))
            .count()
    }

    #[must_use]
    pub fn prototype_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, ExternalItem::Prototype { .. }))
            .count()
    }

    #[must_use]
    pub fn declaration_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, ExternalItem::Declaration { .. }))
            .count()
    }

    #[must_use]
    pub fn function_definition_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, ExternalItem::FunctionDefinition { .. }))
            .count()
    }

    #[must_use]
    pub fn struct_forward_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, ExternalItem::StructForward { .. }))
            .count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalItem {
    Typedef { name: String },
    Declaration { name: String },
    Prototype { name: String },
    FunctionDefinition { name: String },
    StructForward { name: String },
}
