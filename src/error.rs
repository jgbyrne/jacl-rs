use std::result;
use crate::tokeniser::Token;
use crate::util::token_line;

pub struct Error<'src> {
    code: u8,
    msg: String,
    token: Option<Token<'src>>,
    hint: Option<String>,
}

impl<'src> Error<'src> {
    pub fn basic(code: u8, msg: String) -> Error<'src> {
        Error {
            code,
            msg,
            token: None,
            hint: None,
        }
    }

    pub fn detailed(code: u8, msg: String,
                token: Token<'src>, hint: String) -> Error<'src> {
        Error {
            code,
            msg,
            token: Some(token),
            hint: Some(hint),
        }
    }

    pub fn output(&self, input: &'src str, lines: &Vec<(usize, usize)>) {
        if let (Some(ref token), Some(ref hint)) = (&self.token, &self.hint) {
            let code_line = format!("{:<3}| {}", token.lno, token_line(input, lines, token));
            let ptr_line = format!("{}{}", " ".repeat(4 + token.col), "^".repeat(token.len));
            let hint_line = format!("Hint: {}", hint);
            eprintln!("[E{}] {}\n{}\n{}\n{}", self.code, self.msg, code_line, ptr_line, hint_line);
        }
        else {
            eprintln!("[E{}] {}", self.code, self.msg);
        }
    }
}

pub type Result<'src, T> = result::Result<T, Error<'src>>;

