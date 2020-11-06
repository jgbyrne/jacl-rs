use crate::tokeniser::Token;

pub fn token_line<'src>(input: &'src str, lines: &Vec<(usize, usize)>, token: &Token<'src>) -> &'src str {
    let line_ptrs = lines[token.lno - 1];
    &input[line_ptrs.0..=line_ptrs.1]
}
