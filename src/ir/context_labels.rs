use super::{LocalBinding, LoweringContext};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::StructLayout;

impl LoweringContext {
    pub(in crate::ir) fn insert_scope_binding(
        &mut self,
        name: &str,
        binding: LocalBinding,
    ) -> CompileResult<()> {
        let Some(scope) = self.scopes.last_mut() else {
            return Err(CompileError::new("internal error: no local scope"));
        };
        scope.insert(name.to_string(), binding);
        Ok(())
    }

    pub(in crate::ir) fn pop_scope(&mut self) -> CompileResult<()> {
        if self.scopes.pop().is_none() {
            return Err(CompileError::new("internal error: no local scope to pop"));
        }
        Ok(())
    }

    pub(in crate::ir) const fn fresh_label(&mut self) -> usize {
        let label = self.next_label;
        self.next_label += 1;
        label
    }

    pub(in crate::ir) fn named_label(&mut self, name: &str) -> usize {
        if let Some(label) = self.named_labels.get(name) {
            return *label;
        }
        let label = self.fresh_label();
        self.named_labels.insert(name.to_owned(), label);
        label
    }

    pub(in crate::ir) fn local_binding(&self, name: &str) -> Option<LocalBinding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    pub(in crate::ir) fn local_offset(&self, slot: usize) -> CompileResult<usize> {
        self.local_slots
            .get(slot)
            .map(|local_slot| local_slot.offset)
            .ok_or_else(|| CompileError::new("internal error: missing local slot"))
    }

    pub(in crate::ir) fn struct_layout(&self, struct_name: &str) -> CompileResult<&StructLayout> {
        self.structs
            .get(struct_name)
            .ok_or_else(|| CompileError::new(format!("unknown struct: {struct_name}")))
    }
}
