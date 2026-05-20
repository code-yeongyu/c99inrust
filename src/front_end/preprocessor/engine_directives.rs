use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostics::{CompileError, CompileResult};

use super::builtins::define_builtin_system_macros;
use super::condition::{
    ConditionalFrame, all_conditions_active, eval_condition, push_condition, update_elif,
    update_else,
};
use super::definition::MacroDefinition;
use super::directive::{Include, can_fall_back_to_system_include, parse_define, parse_include};
use super::engine::Preprocessor;
use super::includes::resolve_include;

pub(super) struct PreprocessState<'a> {
    pub(super) current_dir: Option<&'a Path>,
    pub(super) macros: &'a mut HashMap<String, MacroDefinition>,
    pub(super) included_files: &'a mut Vec<PathBuf>,
    pub(super) condition_stack: &'a mut Vec<ConditionalFrame>,
    pub(super) output: &'a mut String,
}

impl Preprocessor {
    pub(super) fn handle_directive(
        &self,
        directive: &str,
        line_number: usize,
        state: &mut PreprocessState<'_>,
    ) -> CompileResult<()> {
        if let Some(rest) = directive.strip_prefix("define") {
            if all_conditions_active(state.condition_stack) {
                let (name, definition) = parse_define(rest.trim(), line_number)?;
                state.macros.insert(name, definition);
            }
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("include") {
            if all_conditions_active(state.condition_stack) {
                self.handle_include(rest.trim(), line_number, state)?;
            }
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("undef") {
            if all_conditions_active(state.condition_stack) {
                state.macros.remove(rest.trim());
            }
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("ifdef") {
            push_condition(
                state.condition_stack,
                state.macros.contains_key(rest.trim()),
            );
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("ifndef") {
            push_condition(
                state.condition_stack,
                !state.macros.contains_key(rest.trim()),
            );
            return Ok(());
        }
        Self::handle_conditional_directive(directive, line_number, state)
    }

    fn handle_conditional_directive(
        directive: &str,
        line_number: usize,
        state: &mut PreprocessState<'_>,
    ) -> CompileResult<()> {
        if let Some(rest) = directive.strip_prefix("if") {
            let enabled = eval_condition(rest.trim(), state.macros, line_number)?;
            push_condition(state.condition_stack, enabled);
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("elif") {
            update_elif(
                state.condition_stack,
                eval_condition(rest.trim(), state.macros, line_number)?,
                line_number,
            )?;
            return Ok(());
        }
        if directive.starts_with("else") {
            update_else(state.condition_stack, line_number)?;
            return Ok(());
        }
        if directive.starts_with("endif") {
            if state.condition_stack.pop().is_none() {
                return Err(CompileError::new("unexpected #endif").at(line_number, 1));
            }
            return Ok(());
        }
        if all_conditions_active(state.condition_stack) {
            return Err(
                CompileError::new(format!("unsupported directive #{directive}")).at(line_number, 1),
            );
        }
        Ok(())
    }

    fn handle_include(
        &self,
        rest: &str,
        line_number: usize,
        state: &mut PreprocessState<'_>,
    ) -> CompileResult<()> {
        match parse_include(rest, line_number)? {
            Include::Local(include_path) => {
                self.handle_local_include(&include_path, state)?;
            }
            Include::System(include_path) => {
                define_builtin_system_macros(&include_path, state.macros);
                state.output.push_str("#include <");
                state.output.push_str(&include_path);
                state.output.push_str(">\n");
            }
        }
        Ok(())
    }

    fn handle_local_include(
        &self,
        include_path: &str,
        state: &mut PreprocessState<'_>,
    ) -> CompileResult<()> {
        match resolve_include(&self.include_paths, include_path, state.current_dir) {
            Ok(resolved) => {
                let source = fs::read_to_string(&resolved).map_err(|error| {
                    CompileError::new(format!("failed to read include: {error}"))
                })?;
                state.included_files.push(resolved.clone());
                let include_dir = resolved.parent().map(Path::to_path_buf);
                let include_file = resolved.to_string_lossy();
                let included = self.preprocess_source(
                    &source,
                    include_file.as_ref(),
                    include_dir.as_deref(),
                    state.macros,
                    state.included_files,
                )?;
                state.output.push_str(&included);
            }
            Err(_error) if can_fall_back_to_system_include(include_path) => {
                state.output.push_str("#include \"");
                state.output.push_str(include_path);
                state.output.push_str("\"\n");
            }
            Err(error) => return Err(error),
        }
        Ok(())
    }
}
