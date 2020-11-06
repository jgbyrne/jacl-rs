mod util;
mod tokeniser;
mod error;

pub use error::{Error, Result};

pub fn read_string<'src>(input: &'src str) -> () {
    match tokeniser::tokenise(&input) {
        (lines, Ok(toks)) => {
            println!("{:#?}", toks);
        },
        (lines, Err(errors)) => {
            for err in errors {
                err.output(input, &lines);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tokeniser;
    use std::fs; 

    #[test]
    fn test_tokenise() {
        let input = fs::read_to_string("./src/test.jacl").unwrap();
        match tokeniser::tokenise(&input) {
            (lines, Ok(toks)) => {
                println!("{:#?}", toks);
            },
            (lines, Err(errors)) => {
                for err in errors {
                    err.output(&input, &lines);
                }
            }
        }
    }
}
