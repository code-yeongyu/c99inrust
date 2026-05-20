pub(super) fn normalize_extensions(line: &str) -> String {
    normalize_hex_floats(&strip_attributes(line))
}

fn strip_attributes(line: &str) -> String {
    let chars = line.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        if starts_with(&chars, index, "__attribute__")
            && let Some(end) = attribute_end(&chars, index + "__attribute__".len())
        {
            index = end;
            continue;
        }
        output.push(chars[index]);
        index += 1;
    }
    output
}

fn attribute_end(chars: &[char], mut index: usize) -> Option<usize> {
    while chars
        .get(index)
        .is_some_and(|current| current.is_whitespace())
    {
        index += 1;
    }
    if chars.get(index) != Some(&'(') || chars.get(index + 1) != Some(&'(') {
        return None;
    }
    let mut depth = 0usize;
    while index < chars.len() {
        match chars[index] {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn normalize_hex_floats(line: &str) -> String {
    let chars = line.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        if let Some((literal, end)) = read_hex_float(&chars, index) {
            output.push_str(&literal);
            index = end;
            continue;
        }
        if chars[index] == '"' || chars[index] == '\'' {
            copy_quoted(&chars, &mut index, &mut output);
            continue;
        }
        output.push(chars[index]);
        index += 1;
    }
    output
}

fn read_hex_float(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars.get(start) != Some(&'0') || !matches!(chars.get(start + 1), Some('x' | 'X')) {
        return None;
    }
    let mut index = start + 2;
    let mut value = 0.0f64;
    let mut saw_dot = false;
    let mut divisor = 16.0f64;
    while let Some(current) = chars.get(index).copied() {
        if current == '.' {
            saw_dot = true;
            index += 1;
            break;
        }
        let digit = hex_digit(current)?;
        value = value.mul_add(16.0, digit);
        index += 1;
    }
    if saw_dot {
        while let Some(current) = chars.get(index).copied() {
            let Some(digit) = hex_digit(current) else {
                break;
            };
            value += digit / divisor;
            divisor *= 16.0;
            index += 1;
        }
    }
    if !matches!(chars.get(index), Some('p' | 'P')) {
        return None;
    }
    index += 1;
    let sign = match chars.get(index) {
        Some('-') => {
            index += 1;
            -1
        }
        Some('+') => {
            index += 1;
            1
        }
        _ => 1,
    };
    let exponent_start = index;
    let mut exponent = 0i32;
    while let Some(current) = chars.get(index).copied() {
        let Some(digit) = current.to_digit(10) else {
            break;
        };
        exponent = exponent
            .checked_mul(10)?
            .checked_add(i32::try_from(digit).ok()?)?;
        index += 1;
    }
    if index == exponent_start {
        return None;
    }
    value *= 2f64.powi(sign * exponent);
    Some((format!("{value:.17}"), index))
}

fn hex_digit(value: char) -> Option<f64> {
    value.to_digit(16).map(f64::from)
}

fn starts_with(chars: &[char], index: usize, expected: &str) -> bool {
    expected
        .chars()
        .enumerate()
        .all(|(offset, expected)| chars.get(index + offset) == Some(&expected))
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
