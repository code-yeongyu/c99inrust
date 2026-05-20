use std::collections::HashMap;

use super::definition::MacroDefinition;
use super::macro_operators::replace_params;

pub(super) fn expand_macros(line: &str, macros: &HashMap<String, MacroDefinition>) -> String {
    let mut current = line.to_string();
    for _ in 0..256 {
        let next = expand_macros_once(&current, macros);
        if next == current {
            return next;
        }
        current = next;
    }
    current
}

pub(super) fn expand_builtin_macros(line: &str, current_file: &str, line_number: usize) -> String {
    let chars = line.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        let current = chars[index];
        if current == '"' || current == '\'' {
            copy_quoted(&chars, &mut index, &mut output);
            continue;
        }
        if is_identifier_start(current) {
            let identifier = read_identifier(&chars, &mut index);
            match identifier.as_str() {
                "__FILE__" => output.push_str(&c_string_literal(current_file)),
                "__LINE__" => output.push_str(&line_number.to_string()),
                _ => output.push_str(&identifier),
            }
            continue;
        }
        output.push(current);
        index += 1;
    }
    output
}

fn c_string_literal(value: &str) -> String {
    let mut literal = String::from("\"");
    for current in value.chars() {
        match current {
            '\\' => literal.push_str("\\\\"),
            '"' => literal.push_str("\\\""),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            _ => literal.push(current),
        }
    }
    literal.push('"');
    literal
}

fn expand_macros_once(line: &str, macros: &HashMap<String, MacroDefinition>) -> String {
    let chars = line.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        let current = chars[index];
        if current == '"' || current == '\'' {
            copy_quoted(&chars, &mut index, &mut output);
            continue;
        }
        if is_identifier_start(current) {
            let identifier_start = index;
            let identifier = read_identifier(&chars, &mut index);
            if let Some(definition) = macros.get(&identifier) {
                match definition {
                    MacroDefinition::Object { replacement } => output.push_str(replacement),
                    MacroDefinition::Function {
                        params,
                        replacement,
                    } => {
                        let mut probe = index;
                        while chars.get(probe).is_some_and(|value| value.is_whitespace()) {
                            probe += 1;
                        }
                        if chars.get(probe) == Some(&'(') {
                            index = probe + 1;
                            if let Some(args) = read_macro_args(&chars, &mut index) {
                                let expanded_args = args
                                    .iter()
                                    .map(|arg| expand_macros(arg, macros))
                                    .collect::<Vec<_>>();
                                output.push_str(&replace_params(
                                    replacement,
                                    params,
                                    &args,
                                    &expanded_args,
                                ));
                            } else {
                                output.push_str(&line[identifier_start..]);
                                return output;
                            }
                        } else {
                            output.push_str(&identifier);
                        }
                    }
                }
                continue;
            }
            output.push_str(&identifier);
            continue;
        }
        output.push(current);
        index += 1;
    }
    output
}

fn read_macro_args(chars: &[char], index: &mut usize) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    while *index < chars.len() {
        let value = chars[*index];
        match value {
            '"' | '\'' => copy_quoted(chars, index, &mut current),
            '(' => {
                depth += 1;
                current.push(value);
                *index += 1;
            }
            ')' if depth == 0 => {
                args.push(current.trim().to_string());
                *index += 1;
                return Some(args);
            }
            ')' => {
                depth -= 1;
                current.push(value);
                *index += 1;
            }
            ',' if depth == 0 => {
                args.push(current.trim().to_string());
                current.clear();
                *index += 1;
            }
            _ => {
                current.push(value);
                *index += 1;
            }
        }
    }
    None
}

fn copy_quoted(chars: &[char], index: &mut usize, output: &mut String) {
    let quote = chars[*index];
    let mut escaped = false;
    output.push(quote);
    *index += 1;
    while *index < chars.len() {
        let current = chars[*index];
        output.push(current);
        *index += 1;
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

pub(super) fn read_identifier(chars: &[char], index: &mut usize) -> String {
    let mut value = String::new();
    while chars
        .get(*index)
        .is_some_and(|current| is_identifier_continue(*current))
    {
        value.push(chars[*index]);
        *index += 1;
    }
    value
}

pub(super) const fn is_identifier_start(value: char) -> bool {
    value.is_ascii_alphabetic() || value == '_'
}

pub(super) const fn is_identifier_continue(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_'
}
