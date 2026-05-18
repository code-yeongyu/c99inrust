use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostics::{CompileError, CompileResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessedUnit {
    pub source: String,
    pub included_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct Preprocessor {
    include_paths: Vec<PathBuf>,
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

    pub fn preprocess_file(&self, path: &Path) -> CompileResult<PreprocessedUnit> {
        let mut macros = HashMap::new();
        let mut included_files = Vec::new();
        let source = fs::read_to_string(path)
            .map_err(|error| CompileError::new(format!("failed to read source: {error}")))?;
        let current_dir = path.parent().map(Path::to_path_buf);
        let output = self.preprocess_source(
            &source,
            current_dir.as_deref(),
            &mut macros,
            &mut included_files,
        )?;
        Ok(PreprocessedUnit {
            source: output,
            included_files,
        })
    }

    pub fn preprocess_text(&self, _name: &str, source: &str) -> CompileResult<PreprocessedUnit> {
        let mut macros = HashMap::new();
        let mut included_files = Vec::new();
        let output = self.preprocess_source(source, None, &mut macros, &mut included_files)?;
        Ok(PreprocessedUnit {
            source: output,
            included_files,
        })
    }

    fn preprocess_source(
        &self,
        source: &str,
        current_dir: Option<&Path>,
        macros: &mut HashMap<String, String>,
        included_files: &mut Vec<PathBuf>,
    ) -> CompileResult<String> {
        let mut output = String::new();
        let mut condition_stack = Vec::new();
        for (line_index, raw_line) in source.lines().enumerate() {
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
                output.push_str(&expand_object_macros(raw_line, macros));
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
                let (name, value) = parse_define(rest.trim(), line_number)?;
                state.macros.insert(name, value);
            }
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("include") {
            if all_conditions_active(state.condition_stack) {
                let include_path = parse_include(rest.trim(), line_number)?;
                let resolved = self.resolve_include(&include_path, state.current_dir)?;
                let source = fs::read_to_string(&resolved).map_err(|error| {
                    CompileError::new(format!("failed to read include: {error}"))
                })?;
                state.included_files.push(resolved.clone());
                let include_dir = resolved.parent().map(Path::to_path_buf);
                let included = self.preprocess_source(
                    &source,
                    include_dir.as_deref(),
                    state.macros,
                    state.included_files,
                )?;
                state.output.push_str(&included);
            }
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("ifdef") {
            let name = rest.trim();
            state.condition_stack.push(state.macros.contains_key(name));
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("ifndef") {
            let name = rest.trim();
            state.condition_stack.push(!state.macros.contains_key(name));
            return Ok(());
        }
        if directive.starts_with("else") {
            let Some(last) = state.condition_stack.last_mut() else {
                return Err(CompileError::new("unexpected #else").at(line_number, 1));
            };
            *last = !*last;
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

    fn resolve_include(
        &self,
        include_path: &str,
        current_dir: Option<&Path>,
    ) -> CompileResult<PathBuf> {
        if let Some(dir) = current_dir {
            let candidate = dir.join(include_path);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        for dir in &self.include_paths {
            let candidate = dir.join(include_path);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        Err(CompileError::new(format!(
            "include not found: {include_path}"
        )))
    }
}

struct PreprocessState<'a> {
    current_dir: Option<&'a Path>,
    macros: &'a mut HashMap<String, String>,
    included_files: &'a mut Vec<PathBuf>,
    condition_stack: &'a mut Vec<bool>,
    output: &'a mut String,
}

fn parse_define(rest: &str, line: usize) -> CompileResult<(String, String)> {
    let mut parts = rest.splitn(2, char::is_whitespace);
    let Some(name) = parts.next().filter(|value| !value.is_empty()) else {
        return Err(CompileError::new("expected macro name").at(line, 1));
    };
    let value = parts.next().unwrap_or("1").trim().to_string();
    Ok((name.to_string(), value))
}

fn parse_include(rest: &str, line: usize) -> CompileResult<String> {
    if let Some(stripped) = rest
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return Ok(stripped.to_string());
    }
    Err(CompileError::new("only quoted includes are supported").at(line, 1))
}

fn all_conditions_active(condition_stack: &[bool]) -> bool {
    condition_stack.iter().all(|enabled| *enabled)
}

fn expand_object_macros(line: &str, macros: &HashMap<String, String>) -> String {
    let mut output = String::new();
    let mut chars = line.chars().peekable();
    while let Some(current) = chars.next() {
        if current == '"' || current == '\'' {
            output.push(current);
            copy_quoted(current, &mut chars, &mut output);
            continue;
        }
        if current.is_ascii_alphabetic() || current == '_' {
            let mut identifier = String::from(current);
            while chars
                .peek()
                .is_some_and(|next| next.is_ascii_alphanumeric() || *next == '_')
            {
                if let Some(next) = chars.next() {
                    identifier.push(next);
                }
            }
            if let Some(replacement) = macros.get(&identifier) {
                output.push_str(replacement);
            } else {
                output.push_str(&identifier);
            }
            continue;
        }
        output.push(current);
    }
    output
}

fn copy_quoted(
    quote: char,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    output: &mut String,
) {
    let mut escaped = false;
    for current in chars.by_ref() {
        output.push(current);
        if escaped {
            escaped = false;
            continue;
        }
        if current == '\\' {
            escaped = true;
            continue;
        }
        if current == quote {
            break;
        }
    }
}
