use crate::diagnostics::{CompileError, CompileResult};

use super::super::expansion::{is_identifier_start, read_identifier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ConditionToken {
    Ident(String),
    Integer(i64),
    Defined,
    Bang,
    AndAnd,
    OrOr,
    EqEq,
    NotEq,
    LParen,
    RParen,
    End,
}

pub(super) fn condition_tokens(
    source: &str,
    line_number: usize,
) -> CompileResult<Vec<ConditionToken>> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < chars.len() {
        match chars[index] {
            value if value.is_whitespace() => index += 1,
            value if is_identifier_start(value) => {
                let ident = read_identifier(&chars, &mut index);
                if ident == "defined" {
                    tokens.push(ConditionToken::Defined);
                } else {
                    tokens.push(ConditionToken::Ident(ident));
                }
            }
            value if value.is_ascii_digit() => {
                tokens.push(read_condition_integer(&chars, &mut index, line_number)?);
            }
            '/' if chars.get(index + 1) == Some(&'/') => break,
            '/' if chars.get(index + 1) == Some(&'*') => {
                index += 2;
                while index + 1 < chars.len() && !(chars[index] == '*' && chars[index + 1] == '/') {
                    index += 1;
                }
                if index + 1 < chars.len() {
                    index += 2;
                }
            }
            '!' if chars.get(index + 1) == Some(&'=') => {
                tokens.push(ConditionToken::NotEq);
                index += 2;
            }
            '!' => {
                tokens.push(ConditionToken::Bang);
                index += 1;
            }
            '&' if chars.get(index + 1) == Some(&'&') => {
                tokens.push(ConditionToken::AndAnd);
                index += 2;
            }
            '|' if chars.get(index + 1) == Some(&'|') => {
                tokens.push(ConditionToken::OrOr);
                index += 2;
            }
            '=' if chars.get(index + 1) == Some(&'=') => {
                tokens.push(ConditionToken::EqEq);
                index += 2;
            }
            '(' => {
                tokens.push(ConditionToken::LParen);
                index += 1;
            }
            ')' => {
                tokens.push(ConditionToken::RParen);
                index += 1;
            }
            _ => {
                return Err(
                    CompileError::new("unsupported #if expression token").at(line_number, 1)
                );
            }
        }
    }
    tokens.push(ConditionToken::End);
    Ok(tokens)
}

fn read_condition_integer(
    chars: &[char],
    index: &mut usize,
    line_number: usize,
) -> CompileResult<ConditionToken> {
    let start = *index;
    while chars.get(*index).is_some_and(char::is_ascii_digit) {
        *index += 1;
    }
    let value = chars[start..*index].iter().collect::<String>();
    let parsed = value
        .parse::<i64>()
        .map_err(|_| CompileError::new("integer literal is too large").at(line_number, 1))?;
    Ok(ConditionToken::Integer(parsed))
}
