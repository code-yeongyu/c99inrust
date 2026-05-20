use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostics::{CompileError, CompileResult};

use super::comments::{remove_comments, splice_lines, translate_trigraphs};
use super::condition::all_conditions_active;
use super::definition::MacroDefinition;
use super::engine_directives::PreprocessState;
use super::expansion::{expand_builtin_macros, expand_macros};
use super::extensions::normalize_extensions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessedUnit {
    pub source: String,
    pub included_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct Preprocessor {
    pub(super) include_paths: Vec<PathBuf>,
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

    pub(super) fn preprocess_source(
        &self,
        source: &str,
        current_file: &str,
        current_dir: Option<&Path>,
        macros: &mut HashMap<String, MacroDefinition>,
        included_files: &mut Vec<PathBuf>,
    ) -> CompileResult<String> {
        let mut output = String::new();
        let mut condition_stack = Vec::new();
        let trigraphs_translated = translate_trigraphs(source);
        let spliced = splice_lines(&trigraphs_translated);
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
                let builtins = expand_builtin_macros(&expanded, current_file, line_number);
                output.push_str(&normalize_extensions(&builtins));
                output.push('\n');
            }
        }
        if !condition_stack.is_empty() {
            return Err(CompileError::new("unterminated conditional directive"));
        }
        Ok(output)
    }
}
