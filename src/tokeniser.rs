use crate::error::{Error};
use std::iter;

#[derive(Debug)]
pub enum Value<'src> {
    Name(&'src str),
    
    String(&'src str),
    Integer(i64),
    Float(f64),
    Boolean(bool),

    LBrace,
    RBrace,
    LBrack,
    RBrack,
    LBracePct,
    RBracePct,
    LParen,
    RParen,

    Equals,
    Comma,
    Plus,
    Minus,
    Star,
    Dollar,
    At,

    Break,

    Fault,
}

#[derive(Debug)]
pub struct Token<'src> {
    val: Value<'src>,
    lptr: usize,
    rptr: usize,
    pub lno: usize,
    pub col: usize,
    pub len: usize,
}

impl<'src> Token<'src> {
    fn new(val: Value<'src>, lptr: usize, rptr: usize,
           lno: usize, col: usize, len: usize) -> Token<'src> {
        Token {
            val,
            lptr,
            rptr,
            lno,
            col,
            len,
        }
    }
}

enum State {
    Neutral,
    InBare,
    StartString,
    InString { escaped: bool },
    InFloat,
    SeenBrace,
    SeenPct,

    Unrecoverable,
}

fn unambiguous_symbol<'src>(c: char) -> Option<Value<'src>> { 
    match c {
        '}' => Some(Value::RBrace),
        '[' => Some(Value::LBrack),
        ']' => Some(Value::RBrack),
        '(' => Some(Value::LParen),
        ')' => Some(Value::RParen),

        '=' => Some(Value::Equals),
        ',' => Some(Value::Comma),
        '+' => Some(Value::Plus),
        '-' => Some(Value::Minus),
        '*' => Some(Value::Star),
        '$' => Some(Value::Dollar),
        '@' => Some(Value::At),

        ';' => Some(Value::Break),
        _   => None,
    }
}

pub fn tokenise<'src>(input: &'src str) -> (Vec<(usize, usize)>, Result<Vec<Token<'src>>, Vec<Error<'src>>>) {
    let mut lines: Vec<(usize, usize)> = Vec::new(); // Spans of Lines - byte offsets
    let mut lineptr: usize = 0;                      // Start of current Line - byte offset

    let mut errors: Vec<Error> = Vec::new();

    let mut toks = Vec::new();       // Built Tokens
    let mut lptr: usize = 0;         // Left end of current Token - byte offset
    let mut rptr: usize = 0;         // Right end of current Token - byte offset
    let mut state = State::Neutral;  // Current Tokeniser State

    let mut lcol: usize = 1;         // Left end of current Token - char count

    let mut lno: usize = 1;          // Line number of current char
    let mut col: usize = 1;          // Column number of current char

    let end = iter::once((input.len(), ';'));
    for (offset, c) in input.char_indices().chain(end) {

        if col == 1 {
            lineptr = offset; 
        }

        // Check if we need to change state *before* updating `rptr`
        match &state {
            State::InBare => {
                if !(c.is_alphanumeric() || c == '_') {
                    let buf = &input[lptr..=rptr];
                    if buf.chars().all(|c| c.is_ascii_digit()) {
                        if c == '.' {
                            state = State::InFloat;
                        }
                        else {
                            if let Ok(val) = str::parse::<i64>(buf) {
                                toks.push(Token::new(Value::Integer(val), lptr, rptr,
                                                     lno, lcol, col - lcol));
                            }
                            else {
                                let tok = Token::new(Value::Fault, lptr, rptr,
                                                     lno, lcol, col - lcol);
                                println!("{}, {}", lcol, col);
                                errors.push(Error::detailed(100, String::from("Could not parse number as 64-byte signed Integer"),
                                                            tok, String::from("This value may be too large")));
                            }
                            state = State::Neutral;
                        }
                    }
                    else {
                        match buf {
                            "true" => {
                                toks.push(Token::new(Value::Boolean(true), lptr, rptr,
                                                     lno, lcol, col - lcol));
                            },
                            "false" => {
                                toks.push(Token::new(Value::Boolean(false), lptr, rptr,
                                                     lno, lcol, col - lcol));
                            },
                            _ => {
                                toks.push(Token::new(Value::Name(buf), lptr, rptr,
                                                     lno, lcol, col - lcol));
                            }
                        }
                        state = State::Neutral;
                    }
                }
            },
            State::InFloat => {
                if !c.is_ascii_digit() {
                    let buf = &input[lptr..=rptr];
                    if let Ok(val) = str::parse::<f64>(buf) {
                        toks.push(Token::new(Value::Float(val), lptr, rptr,
                                             lno, lcol, col - lcol));
                    }
                    else {
                        let tok = Token::new(Value::Fault, lptr, rptr,
                                             lno, lcol, col - lcol);

                        errors.push(Error::detailed(101, String::from("Could not parse number as 64-byte Float"),
                                                    tok, String::from("This value may be too large")));
                    }
                    state = State::Neutral;
                }
            },
            State::SeenBrace => {
                if c != '%' {
                    assert!(lptr == rptr);
                    toks.push(Token::new(Value::LBrace, lptr, rptr,
                                         lno, lcol, col - lcol));
                    state = State::Neutral; 
                }
            }
            _ => {},
        }

        if c == '\n' {
            if let State::SeenPct = state {
               let tok = Token::new(Value::Fault, lptr, rptr,
                                     lno, lcol, col - lcol);

                errors.push(Error::detailed(104, String::from("% was followed by a newline"),
                                            tok, String::from("Expected '}'"))); 

                state = State::Unrecoverable;
            }

            lines.push((lineptr, rptr));
            lno += 1;
            col = 1;
            continue;
        }

        rptr = offset;

        // Enter the string now we've got past the initial quote
        if let State::StartString = state {
            lptr = offset;
            lcol = col;
            state = State::InString { escaped: false };
        }

        // Check if we need to change state *after* updating `rptr`
        match &state {
            State::Neutral => {
                lptr = offset;
                lcol = col;

                if let Some(symbol) = unambiguous_symbol(c) {
                    toks.push(Token::new(symbol, lptr, rptr,
                                         lno, lcol, 1));
                }
                else if c == '{' {
                    state = State::SeenBrace;
                }
                else if c == '%' {
                    state = State::SeenPct;
                }
                else if c == '"' {
                    state = State::StartString;
                }
                else if c.is_alphanumeric() || c == '_' {
                    state = State::InBare;
                }
                else if !c.is_whitespace() {
                    let tok = Token::new(Value::Fault, lptr, rptr,
                                         lno, lcol, 1);

                    errors.push(Error::detailed(102, String::from("Unexpected Character"),
                                                tok, String::from("This character could not be understood")));

                    state = State::Unrecoverable;
                }
            },
            State::InString { escaped } => {
                if !escaped {
                    if c == '"' {
                        let buf = &input[lptr..rptr];
                        toks.push(Token::new(Value::String(buf), lptr, rptr,
                                             lno, lcol, col - lcol));
                        state = State::Neutral;
                    }
                    else if c == '\\' {
                        state = State::InString { escaped: true };
                    }
                }
                else {
                    state = State::InString { escaped: false };
                }
            },
            State::SeenBrace => {
                assert!(c == '%');
                toks.push(Token::new(Value::LBracePct, lptr, rptr,
                                     lno, lcol, col - lcol));
                state = State::Neutral;
            },
            State::SeenPct => {
                if c == '}' {
                    toks.push(Token::new(Value::RBracePct, lptr, rptr,
                                         lno, lcol, col - lcol));
                    state = State::Neutral;
                }
                else {
                    let tok = Token::new(Value::Fault, lptr, rptr,
                                         lno, lcol, col - lcol);

                    errors.push(Error::detailed(103, String::from("% was not followed by }"),
                                                tok, String::from("Unparseable character pair here"))); 

                    state = State::Unrecoverable;
                }
            },
            _ => {},
        }

        col += 1;
    }

    if errors.len() > 0 {
        (lines, Err(errors))
    }
    else {
        (lines, Ok(toks))
    }
}

