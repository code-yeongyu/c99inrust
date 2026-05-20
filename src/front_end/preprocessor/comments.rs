use crate::diagnostics::{CompileError, CompileResult};

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

const fn advance_position(value: char, line: &mut usize, column: &mut usize) {
    if value == '\n' {
        *line += 1;
        *column = 1;
    } else {
        *column += 1;
    }
}
