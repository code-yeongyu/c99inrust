use super::expansion::{is_identifier_start, read_identifier};

pub(super) fn replace_params(replacement: &str, params: &[String], args: &[String]) -> String {
    let chars = replacement.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        let current = chars[index];
        if current == '"' || current == '\'' {
            copy_quoted(&chars, &mut index, &mut output);
            continue;
        }
        if current == '#' && chars.get(index + 1) == Some(&'#') {
            output.push_str("##");
            index += 2;
            continue;
        }
        if current == '#'
            && let Some((arg, next_index)) = stringified_arg(&chars, index + 1, params, args)
        {
            output.push_str(&c_string_literal(arg));
            index = next_index;
            continue;
        }
        if is_identifier_start(current) {
            let identifier = read_identifier(&chars, &mut index);
            if let Some(param_index) = params.iter().position(|param| param == &identifier) {
                if let Some(arg) = args.get(param_index) {
                    output.push_str(arg);
                }
            } else {
                output.push_str(&identifier);
            }
            continue;
        }
        output.push(current);
        index += 1;
    }
    paste_tokens(&output)
}

fn stringified_arg<'a>(
    chars: &[char],
    mut index: usize,
    params: &[String],
    args: &'a [String],
) -> Option<(&'a str, usize)> {
    while chars
        .get(index)
        .is_some_and(|current| current.is_whitespace())
    {
        index += 1;
    }
    let identifier = read_identifier(chars, &mut index);
    let param_index = params.iter().position(|param| param == &identifier)?;
    args.get(param_index).map(|arg| (arg.trim(), index))
}

fn paste_tokens(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '"' || chars[index] == '\'' {
            copy_quoted(&chars, &mut index, &mut output);
            continue;
        }
        if chars.get(index) == Some(&'#') && chars.get(index + 1) == Some(&'#') {
            trim_trailing_whitespace(&mut output);
            index += 2;
            while chars
                .get(index)
                .is_some_and(|current| current.is_whitespace())
            {
                index += 1;
            }
            continue;
        }
        output.push(chars[index]);
        index += 1;
    }
    output
}

fn trim_trailing_whitespace(value: &mut String) {
    while value.chars().next_back().is_some_and(char::is_whitespace) {
        value.pop();
    }
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
