use std::result;
use crate::Lines;
use crate::tokeniser::Token;
use crate::util::token_line;

#[derive(Clone, Debug)]
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

    pub fn render(&self, input: &'src str, lines: &Lines) -> String {
        if let (Some(ref token), Some(ref hint)) = (&self.token, &self.hint) {
            let code_line = format!("{:<3}| {}", token.lno, token_line(input, lines, token));
            let ptr_line = format!("{}{}", " ".repeat(4 + token.col), "^".repeat(token.len));
            let hint_line = format!("Hint: {}", hint);
            format!("[E{}] {}\n{}\n{}\n{}\n", self.code, self.msg, code_line, ptr_line, hint_line)
        }
        else {
            format!("[E{}] {}\n", self.code, self.msg)
        }
    }
}

pub type Result<'src, T> = result::Result<T, Error<'src>>;

