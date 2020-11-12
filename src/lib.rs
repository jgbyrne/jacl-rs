mod util;
mod tokeniser;
mod parser;
mod error;
mod types;

pub use error::{Error};

type Lines = Vec<(usize, usize)>;

pub struct Jacl {
    strct: types::Struct,
}

pub struct JaclError<'src> {
    internal: Error<'src>,
    input: &'src str,
    lines: Lines,
}

impl<'src> JaclError<'src> {
    fn from_error(err: Error<'src>, input: &'src str, lines: Lines) -> JaclError<'src> {
        JaclError {
            internal: err,
            input,
            lines,
        }
    }
}

pub fn read_string<'src>(input: &'src str) -> Result<Jacl, JaclError> {
    match tokeniser::tokenise(&input) {
        (lines, Ok(toks)) => {
            match parser::parse(&input, &lines, toks) {
                Ok(data) => Ok(Jacl { strct: data }),
                Err(err) => Err(JaclError::from_error(err, &input, lines)),
            }
        },
        (lines, Err(errors)) => {
            Err(JaclError::from_error(
                    errors.get(0).expect("Tokeniser returned empty error list").clone(),
                    &input,
                    lines))
        }
    }
}
