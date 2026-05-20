use crate::diagnostics::{CompileError, CompileResult};

use super::definition::MacroDefinition;
use super::expansion::{is_identifier_continue, is_identifier_start};

pub(super) enum Include {
    Local(String),
    System(String),
}

pub(super) fn parse_define(rest: &str, line: usize) -> CompileResult<(String, MacroDefinition)> {
    let mut chars = rest.char_indices().peekable();
    let Some((_, first)) = chars.next() else {
        return Err(CompileError::new("expected macro name").at(line, 1));
    };
    if !is_identifier_start(first) {
        return Err(CompileError::new("expected macro name").at(line, 1));
    }
    let mut name = String::from(first);
    let mut end = first.len_utf8();
    while let Some((index, current)) = chars.peek().copied() {
        if !is_identifier_continue(current) {
            break;
        }
        name.push(current);
        end = index + current.len_utf8();
        chars.next();
    }
    let after_name = &rest[end..];
    if let Some(after_open) = after_name.strip_prefix('(') {
        let Some(close_index) = after_open.find(')') else {
            return Err(CompileError::new("unterminated function-like macro params").at(line, 1));
        };
        let params_source = &after_open[..close_index];
        let params = if params_source.trim().is_empty() {
            Vec::new()
        } else {
            params_source
                .split(',')
                .map(str::trim)
                .map(|param| if param == "..." { "__VA_ARGS__" } else { param })
                .map(str::to_string)
                .collect()
        };
        let replacement = after_open[close_index + 1..].trim().to_string();
        return Ok((
            name,
            MacroDefinition::Function {
                params,
                replacement,
            },
        ));
    }
    let replacement = after_name.trim().to_string();
    Ok((name, MacroDefinition::Object { replacement }))
}

pub(super) fn parse_include(rest: &str, line: usize) -> CompileResult<Include> {
    if let Some(stripped) = rest
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return Ok(Include::Local(stripped.to_string()));
    }
    if let Some(stripped) = rest
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
    {
        return Ok(Include::System(stripped.to_string()));
    }
    Err(CompileError::new("expected quoted or system include").at(line, 1))
}

pub(super) fn can_fall_back_to_system_include(include_path: &str) -> bool {
    !include_path.contains('/') && !include_path.contains('\\')
}
