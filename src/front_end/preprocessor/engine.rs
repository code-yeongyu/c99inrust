use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostics::{CompileError, CompileResult};

use super::builtins::define_builtin_system_macros;
use super::comments::{remove_comments, splice_lines};
use super::condition::{
    ConditionalFrame, all_conditions_active, eval_condition, push_condition, update_elif,
    update_else,
};
use super::definition::MacroDefinition;
use super::directive::{Include, can_fall_back_to_system_include, parse_define, parse_include};
use super::expansion::{expand_builtin_macros, expand_macros};
use super::includes::resolve_include;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessedUnit {
    pub source: String,
    pub included_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct Preprocessor {
    include_paths: Vec<PathBuf>,
    predefined: HashMap<String, MacroDefinition>,
}

impl Preprocessor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_include_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.include_paths.push(path.into());
        self
    }

    #[must_use]
    pub fn with_define(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.predefined.insert(
            name.into(),
            MacroDefinition::Object {
                replacement: value.into(),
            },
        );
        self
    }

    /// Preprocesses a file and recursively resolves local includes.
    ///
    /// # Errors
    ///
    /// Returns an error when source or include files cannot be read, directives
    /// are malformed, or conditional preprocessing is unbalanced.
    pub fn preprocess_file(&self, path: &Path) -> CompileResult<PreprocessedUnit> {
        let mut macros = self.predefined.clone();
        let mut included_files = Vec::new();
        let source = fs::read_to_string(path)
            .map_err(|error| CompileError::new(format!("failed to read source: {error}")))?;
        let current_dir = path.parent().map(Path::to_path_buf);
        let current_file = path.to_string_lossy();
        let output = self.preprocess_source(
            &source,
            current_file.as_ref(),
            current_dir.as_deref(),
            &mut macros,
            &mut included_files,
        )?;
        Ok(PreprocessedUnit {
            source: output,
            included_files,
        })
    }

    /// Preprocesses source text without an on-disk current directory.
    ///
    /// # Errors
    ///
    /// Returns an error when directives are malformed or conditional
    /// preprocessing is unbalanced.
    pub fn preprocess_text(&self, name: &str, source: &str) -> CompileResult<PreprocessedUnit> {
        let mut macros = self.predefined.clone();
        let mut included_files = Vec::new();
        let output =
            self.preprocess_source(source, name, None, &mut macros, &mut included_files)?;
        Ok(PreprocessedUnit {
            source: output,
            included_files,
        })
    }

    fn preprocess_source(
        &self,
        source: &str,
        current_file: &str,
        current_dir: Option<&Path>,
        macros: &mut HashMap<String, MacroDefinition>,
        included_files: &mut Vec<PathBuf>,
    ) -> CompileResult<String> {
        let mut output = String::new();
        let mut condition_stack = Vec::new();
        let spliced = splice_lines(source);
        let uncommented = remove_comments(&spliced)?;
        for (line_index, raw_line) in uncommented.lines().enumerate() {
            let line_number = line_index + 1;
            let trimmed = raw_line.trim_start();
            if let Some(directive) = trimmed.strip_prefix('#') {
                let mut state = PreprocessState {
                    current_dir,
                    macros,
                    included_files,
                    condition_stack: &mut condition_stack,
                    output: &mut output,
                };
                self.handle_directive(directive.trim_start(), line_number, &mut state)?;
                continue;
            }
            if all_conditions_active(&condition_stack) {
                let expanded = expand_macros(raw_line, macros);
                output.push_str(&expand_builtin_macros(&expanded, current_file, line_number));
                output.push('\n');
            }
        }
        if !condition_stack.is_empty() {
            return Err(CompileError::new("unterminated conditional directive"));
        }
        Ok(output)
    }

    fn handle_directive(
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

struct PreprocessState<'a> {
    current_dir: Option<&'a Path>,
    macros: &'a mut HashMap<String, MacroDefinition>,
    included_files: &'a mut Vec<PathBuf>,
    condition_stack: &'a mut Vec<ConditionalFrame>,
    output: &'a mut String,
}
