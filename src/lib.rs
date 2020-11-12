mod util;
mod tokeniser;
mod parser;
mod error;
mod types;
mod api;

pub use error::{Error};
pub use crate::api::*;

type Lines = Vec<(usize, usize)>;

pub fn read_string<'src>(input: &'src str) -> Result<Jacl, JaclError> {
    match tokeniser::tokenise(&input) {
        (lines, Ok(toks)) => {
            match parser::parse(&input, &lines, toks) {
                Ok(data) => Ok(Jacl::init(data)),
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
