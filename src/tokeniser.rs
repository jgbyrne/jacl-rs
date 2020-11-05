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
}

#[derive(Debug)]
pub struct Token<'src> {
    val: Value<'src>,
    lptr: usize,
    rptr: usize,
    lno: usize,
    col: usize,
    len: usize,
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

pub fn tokenise<'src>(input: &'src str) -> Vec<Token<'src>> {
    let mut toks = Vec::new();       // Built Tokens
    let mut lptr: usize = 0;         // Left end of current Token - byte offset
    let mut rptr: usize = 0;         // Right end of current Token - byte offset
    let mut state = State::Neutral;  // Current Tokeniser State

    let mut lcol: usize = 1;         // Left end of current Token - char count

    let mut lno: usize = 1;          // Line number of current char
    let mut col: usize = 1;          // Column number of current char

    for (offset, c) in input.char_indices() {

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
                            let val = str::parse::<i64>(buf).expect("Could not parse number to i64");
                            toks.push(Token::new(Value::Integer(val), lptr, rptr,
                                                 lno, lcol, col - lcol));
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
                    let val = str::parse::<f64>(buf).expect("Could not parse number to f64");
                    toks.push(Token::new(Value::Float(val), lptr, rptr,
                                         lno, lcol, col - lcol));
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
                                         lno, lcol, col - lcol));
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
                    panic!("Unrecognised symbol {}", c);
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
                if c != '}' {
                    panic!("% was not followed by }");
                }
                toks.push(Token::new(Value::RBracePct, lptr, rptr,
                                     lno, lcol, col - lcol));
                state = State::Neutral;
            },
            _ => {},
        }

        if c == '\n' {
            lno += 1;
            col = 1;
            continue;
        }

        if c == '\r' {
            continue;
        }


        col += 1;
    }

    toks
}
