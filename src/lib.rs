mod util;
mod tokeniser;
mod parser;
mod error;

pub use error::{Error, Result};

type Lines = Vec<(usize, usize)>;

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
    use crate::parser;
    use std::fs; 

    #[test]
    fn test_tokenise() {
        let input = fs::read_to_string("./src/minimal.jacl").unwrap();
        match tokeniser::tokenise(&input) {
            (lines, Ok(toks)) => {
                println!("{:#?}", toks);
                match parser::parse(&input, &lines, toks) {
                    Ok(data) => println!("{:#?}", data),
                    Err(e) => e.output(&input, &lines),
                }
            },
            (lines, Err(errors)) => {
                for err in errors {
                    err.output(&input, &lines);
                }
            }
        }
    }
}
