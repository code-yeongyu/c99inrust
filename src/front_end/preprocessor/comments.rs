use crate::diagnostics::{CompileError, CompileResult};

pub(super) fn translate_trigraphs(source: &str) -> String {
    let chars = source.chars().collect::<Vec<_>>();
    let mut translated = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        if chars.get(index) == Some(&'?')
            && chars.get(index + 1) == Some(&'?')
            && let Some(value) = trigraph_replacement(chars.get(index + 2).copied())
        {
            translated.push(value);
            index += 3;
            continue;
        }
        translated.push(chars[index]);
        index += 1;
    }
    translate_digraphs(&translated)
}

pub(super) fn splice_lines(source: &str) -> String {
    source.replace("\\\r\n", "").replace("\\\n", "")
}

pub(super) fn remove_comments(source: &str) -> CompileResult<String> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    let mut line = 1usize;
    let mut column = 1usize;
    while index < chars.len() {
        let current = chars[index];
        if current == '"' || current == '\'' {
            copy_quoted_and_update_position(
                &chars,
                &mut index,
                &mut output,
                &mut line,
                &mut column,
            );
            continue;
        }
        if current == '/' && chars.get(index + 1) == Some(&'/') {
            output.push(' ');
            advance_position(current, &mut line, &mut column);
            index += 1;
            advance_position(chars[index], &mut line, &mut column);
            index += 1;
            while chars.get(index).is_some_and(|value| *value != '\n') {
                advance_position(chars[index], &mut line, &mut column);
                index += 1;
            }
            continue;
        }
        if current == '/' && chars.get(index + 1) == Some(&'*') {
            let start_line = line;
            let start_column = column;
            output.push(' ');
            advance_position(current, &mut line, &mut column);
            index += 1;
            advance_position(chars[index], &mut line, &mut column);
            index += 1;
            let mut closed = false;
            while index < chars.len() {
                if chars[index] == '*' && chars.get(index + 1) == Some(&'/') {
                    advance_position(chars[index], &mut line, &mut column);
                    index += 1;
                    advance_position(chars[index], &mut line, &mut column);
                    index += 1;
                    closed = true;
                    break;
                }
                if chars[index] == '\n' {
                    output.push('\n');
                }
                advance_position(chars[index], &mut line, &mut column);
                index += 1;
            }
            if !closed {
                return Err(
                    CompileError::new("unterminated block comment").at(start_line, start_column)
                );
            }
            continue;
        }
        output.push(current);
        advance_position(current, &mut line, &mut column);
        index += 1;
    }
    Ok(output)
}

fn copy_quoted_and_update_position(
    chars: &[char],
    index: &mut usize,
    output: &mut String,
    line: &mut usize,
    column: &mut usize,
) {
    let quote = chars[*index];
    output.push(quote);
    advance_position(quote, line, column);
    *index += 1;
    let mut escaped = false;
    while *index < chars.len() {
        let current = chars[*index];
        output.push(current);
        *index += 1;
        advance_position(current, line, column);
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

fn translate_digraphs(source: &str) -> String {
    let chars = source.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        let current = chars[index];
        if current == '"' || current == '\'' {
            copy_quoted(&chars, &mut index, &mut output);
            continue;
        }
        if chars.get(index) == Some(&'%')
            && chars.get(index + 1) == Some(&':')
            && chars.get(index + 2) == Some(&'%')
            && chars.get(index + 3) == Some(&':')
        {
            output.push_str("##");
            index += 4;
            continue;
        }
        if let Some((replacement, width)) = digraph_replacement(&chars[index..]) {
            output.push_str(replacement);
            index += width;
            continue;
        }
        output.push(current);
        index += 1;
    }
    output
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

const fn trigraph_replacement(value: Option<char>) -> Option<char> {
    match value {
        Some('=') => Some('#'),
        Some('/') => Some('\\'),
        Some('\'') => Some('^'),
        Some('(') => Some('['),
        Some(')') => Some(']'),
        Some('!') => Some('|'),
        Some('<') => Some('{'),
        Some('>') => Some('}'),
        Some('-') => Some('~'),
        _ => None,
    }
}

fn digraph_replacement(chars: &[char]) -> Option<(&'static str, usize)> {
    match (chars.first(), chars.get(1)) {
        (Some('<'), Some(':')) => Some(("[", 2)),
        (Some(':'), Some('>')) => Some(("]", 2)),
        (Some('<'), Some('%')) => Some(("{", 2)),
        (Some('%'), Some('>')) => Some(("}", 2)),
        (Some('%'), Some(':')) => Some(("#", 2)),
        _ => None,
    }
}

const fn advance_position(value: char, line: &mut usize, column: &mut usize) {
    if value == '\n' {
        *line += 1;
        *column = 1;
    } else {
        *column += 1;
    }
}
