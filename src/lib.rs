mod tokeniser;

#[cfg(test)]
mod tests {
    use crate::tokeniser;
    use std::fs; 

    #[test]
    fn test_tokenise() {
        let input = fs::read_to_string("./src/test.jacl").unwrap();
        let toks = tokeniser::tokenise(&input);
        println!("{:#?}", toks);
    }
}
