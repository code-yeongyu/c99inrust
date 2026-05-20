const PUNCTUATORS: &[&str] = &[
    ">>=", "<<=", "...", "++", "--", "->", "<<", ">>", "<=", ">=", "==", "!=", "&&", "||", "+=",
    "-=", "*=", "/=", "%=", "&=", "|=", "^=", "##", "{", "}", "(", ")", "[", "]", ";", ",", ".",
    "&", "*", "+", "-", "~", "!", "/", "%", "<", ">", "^", "|", "?", ":", "=", "#",
];

pub(super) fn first_match(input: &[char], index: usize) -> Option<&'static str> {
    PUNCTUATORS
        .iter()
        .copied()
        .find(|candidate| starts_with(input, index, candidate))
}

fn starts_with(input: &[char], index: usize, expected: &str) -> bool {
    expected
        .chars()
        .enumerate()
        .all(|(offset, expected_char)| input.get(index + offset).copied() == Some(expected_char))
}
