use std::collections::HashMap;

use crate::Lines;
use crate::tokeniser::{Token, TokVal};
use crate::error::Error;

// If `Value` and `Struct` are good enough to be the final repr, maybe factor out?

#[derive(Clone, Debug)]
pub enum Value {
    Key(String),
    ForeignKey(String),
    Property(String),
    Tuple(Vec<Value>),

    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug)]
pub enum Struct {
    Object { entries: Entries, props: Props},
    Table { entries: Entries },
    Map { props: Props },
}

enum RValue {
    Value(Value),
    Struct(Struct),
}

type Entries = HashMap<String, Option<Struct>>;
type Props = HashMap<String, Value>;

struct Parser<'src> {
    input: &'src str,
    lines: &'src Lines,
    tokens: Vec<Token<'src>>,

    ptr: usize,
}

impl<'src> Parser<'src> {
    fn new(input: &'src str, lines: &'src Lines, tokens: Vec<Token<'src>>) -> Parser<'src> {
        Parser {
           input,
           lines,
           tokens,
           ptr: 0,
        }
    }

    fn cur(&self) -> Option<&Token<'src>> {
        self.tokens.get(self.ptr)
    }

    fn cur_expect(&self) -> Result<Token<'src>, Error<'src>> {
        match self.tokens.get(self.ptr) {
            Some(tok) => {
                Ok(tok.clone())
            },
            None => {
                panic!();
                Err(Error::basic(154, String::from("Unexpected End-of-file")))
            },
        }
    }

    fn peek(&self, n: usize) -> Option<&Token<'src>> {
        self.tokens.get(self.ptr + n)
    }

    fn peek_expect(&self, n: usize) -> Result<&Token<'src>, Error<'src>> {
        match self.tokens.get(self.ptr + n) {
            Some(tok) => {
                Ok(tok)
            },
            None => {
                Err(Error::basic(155, String::from("Unexpected End-of-file")))
            },
        }
    }

    fn nxt(&self) -> Option<&Token<'src>> {
        self.peek(1)
    }

    fn nxt_expect(&self) -> Result<&Token<'src>, Error<'src>> {
        self.peek_expect(1)
    }

    fn advance(&mut self, n: usize) -> Option<&Token<'src>> {
        self.ptr += n;
        self.cur()
    }

    fn advance_expect(&mut self, n: usize) -> Result<Token<'src>, Error<'src>> {
        self.ptr += n;
        self.cur_expect()
    }

    fn step(&mut self) -> Option<&Token<'src>> {
        self.advance(1)
    }

    fn step_expect(&mut self) -> Result<Token<'src>, Error<'src>> {
        self.advance_expect(1)
    }

    fn expect(&mut self, gate: fn(tv: &TokVal) -> bool, exp: &str) -> Result<Token<'src>, Error<'src>> {
        let ret = match self.cur() {
            Some(tok) => {
                if gate(&tok.val) {
                    Ok(tok.clone())
                }
                else {
                    println!("expected {:?}", tok);
                    Err(Error::detailed(150, format!("Expected {}", exp),
                                        tok.clone(), format!("Found {:?}", tok)))
                }
            },
            None => {
                Err(Error::basic(151, format!("Expected {} but found End-of-File", exp)))
            }
        };
        self.step();
        ret
    }
    
    fn allow_break(&mut self) {
        loop {
            match self.cur() {
                Some(tok) => {
                    match tok.val {
                        TokVal::Break => { self.step(); },
                        _ => { return }
                    }
                },
                None => {
                    return
                },
            }
        }
    }
}

/* Parse Bindings */

fn parse_val<'src>(parser: &mut Parser<'src>) -> Result<Value, Error<'src>> {
    match parser.cur_expect()?.val {
        TokVal::Name(name) => {
            parser.step();
            Ok(Value::Key(name.to_string()))
        }
        TokVal::Dollar => {
            parser.step();
            if let TokVal::Name(name) = parser.expect(
                |tv| matches!(tv, TokVal::Name(_)), "name")?.val {
                Ok(Value::Property(name.to_string()))
            }
            else { Err(
                Error::basic(1, String::from("Internal Error: Reached the unreachable!"))
            )}
        },
        TokVal::At => {
            parser.step();
            if let TokVal::Name(name) = parser.expect(
                |tv| matches!(tv, TokVal::Name(_)), "name")?.val {
                Ok(Value::ForeignKey(name.to_string()))
            }
            else { Err(
                Error::basic(1, String::from("Internal Error: Reached the unreachable!"))
            )}
        },
        TokVal::String(string) => {
            parser.step();
            Ok(Value::Str(string.to_string()))
        },
        TokVal::Integer(integer) => {
            parser.step();
            Ok(Value::Int(integer))
        },
        TokVal::Float(float) => {
            parser.step();
            Ok(Value::Float(float))
        },
        TokVal::Boolean(boolean) => {
            parser.step();
            Ok(Value::Bool(boolean))
        }
        _ => {
            Err(
                Error::detailed(158,
                                String::from("Expected Value"),
                                parser.cur_expect()?.clone(),
                                String::from("Make this a value"))
            )
        }
    }
}

fn parse_rval<'src>(parser: &mut Parser<'src>) -> Result<RValue, Error<'src>> {
    match parser.cur_expect()?.val {
        TokVal::LBrace | TokVal::LBrack | TokVal::LBracePct => {
            Ok(RValue::Struct(parse_struct(parser)?))
        }
        _ => {
            Ok(RValue::Value(parse_val(parser)?))
        }
    }
}

fn parse_rhs<'src>(parser: &mut Parser<'src>, strct: &mut Struct, names: Vec<String>) -> Result<(), Error<'src>> {
    let eq = parser.expect(|tv| matches!(tv, TokVal::Equals), "'='")?;
    let rval_start = parser.cur_expect()?;
    match parse_rval(parser)? {
        RValue::Value(val) => {
            match strct {
                Struct::Object { entries: _ , props } |
                Struct::Map { props } => {
                    for name in names {
                        props.insert(name.to_string(), val.clone());
                    }
                    Ok(())
                },
                Struct::Table { entries } => {
                     Err(Error::detailed(157, String::from("Tables cannot contain Bindings"),
                             eq.clone(), String::from("Remove this entry")))
                }
            }
        },
        RValue::Struct(st) => {
            match strct {
                Struct::Object { entries, props } => {
                    let anon_key = format!("#anon{}", entries.len());
                    for name in names {
                        props.insert(name.to_string(), Value::Key(anon_key.clone()));
                    }
                    entries.insert(anon_key, Some(st));
                    Ok(())
                },
                Struct::Map { props } => {
                    Err(Error::detailed(156, String::from("Maps cannot contain Entries"),
                             rval_start.clone(), String::from("Remove this entry")))
                },
                Struct::Table { entries } => {
                     Err(Error::detailed(157, String::from("Tables cannot contain Bindings"),
                             eq.clone(), String::from("Remove this entry")))
                }
            }
        },
    }
}

fn parse_single_binding<'src>(parser: &mut Parser<'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    if let TokVal::Name(name) = parser.cur_expect()?.val {
        parser.step();
        parse_rhs(parser, strct, vec![name.to_string()])
    }
    else {
        Err(Error::basic(1, String::from("Internal Parser Error: TokVal was not Name")))
    }
}

fn parse_multiple_binding<'src>(parser: &mut Parser<'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    Ok(())
}

/* Parse Entries */

fn parse_single_entry<'src>(parser: &mut Parser<'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    match strct {
        Struct::Object { entries, props: _ } |
        Struct::Table { entries } => {
            if let TokVal::Name(name) = parser.cur_expect()?.val {
                parser.step();
                let strct = parse_struct(parser)?;
                entries.insert(name.to_string(), Some(strct));
                Ok(())
            }
            else {
                Err(Error::basic(1, String::from("Internal Parser Error: TokVal was not Name")))
            }
        },
        Struct::Map { props: _ } => {
            Err(Error::detailed(156, String::from("Maps cannot contain Entries"),
                                parser.cur_expect()?.clone(), String::from("Remove this entry")))
        }
    }
}

fn parse_compound_entry<'src>(parser: &mut Parser<'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    Ok(())
}

fn parse_wild_entry<'src>(parser: &mut Parser<'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    Ok(())
}

fn parse_prop_entry<'src>(parser: &mut Parser<'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    Ok(())
}

fn parse_empty_entry<'src>(parser: &mut Parser<'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    match strct {
        Struct::Object { entries, props: _ } |
        Struct::Table { entries } => {
            if let TokVal::Name(name) = parser.cur_expect()?.val {
                parser.step();
                entries.insert(name.to_string(), None);
                Ok(())
            }
            else {
                Err(Error::basic(1, String::from("Internal Parser Error: TokVal was not Name")))
            }
        },
        Struct::Map { props: _ } => {
            Err(Error::detailed(156, String::from("Maps cannot contain Entries"),
                                parser.cur_expect()?.clone(), String::from("Remove this entry")))
        }
    }
}

fn parse_anon_entry<'src>(parser: &mut Parser<'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    match strct {
        Struct::Object { entries, props: _ } |
        Struct::Table { entries } => {
            let strct = parse_struct(parser)?;
            entries.insert(format!("#anon{}", entries.len()), Some(strct));
            Ok(())
        },
        Struct::Map { props: _ } => {
            Err(Error::detailed(156, String::from("Maps cannot contain Entries"),
                                parser.cur_expect()?.clone(), String::from("Remove this entry")))
        }
    }
}

/* Parse Structures */

fn parse_inner<'src>(parser: &mut Parser<'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    loop {
        parser.allow_break();
        let cur_tok = parser.cur();
        if let Some(tok) = cur_tok {
            match tok.val {
                TokVal::RBrace | TokVal::RBrack | TokVal::RBracePct => {
                    break;
                },
                TokVal::Name(name) => {
                    let nxt = parser.nxt_expect()?;
                    match nxt.val {
                        TokVal::Comma => { // multi binding
                            parse_multiple_binding(parser, strct)?;
                        },
                        TokVal::Equals => { // single binding
                            parse_single_binding(parser, strct)?;
                        },
                        TokVal::Plus => { // compound selector
                            parse_compound_entry(parser, strct)?;
                        },
                        TokVal::Break => { // atomic obj
                            parse_empty_entry(parser, strct)?;
                        },
                        TokVal::Star => {
                            parse_wild_entry(parser, strct)?;
                        },
                        TokVal::Dollar => {
                            parse_prop_entry(parser, strct)?;
                        },
                        _ => { // assume we have a simple selector
                            parse_single_entry(parser, strct)?;
                        }
                    }
                },
                _ => {
                    parse_anon_entry(parser, strct)?;
                }
            }
        }
        else {
            break;
        }
    }
    Ok(())
}


fn parse_obj_struct<'src>(parser: &mut Parser<'src>) -> Result<Struct, Error<'src>> {
    parser.expect(|tv| matches!(tv, TokVal::LBrace), "'{'");
    let mut obj = Struct::Object {
        entries: HashMap::new(),
        props: HashMap::new(),
    };
    parse_inner(parser, &mut obj)?;
    parser.expect(|tv| matches!(tv, TokVal::RBrace), "'}'");
    Ok(obj)
}

fn parse_tbl_struct<'src>(parser: &mut Parser<'src>) -> Result<Struct, Error<'src>> {
    parser.expect(|tv| matches!(tv, TokVal::LBrack), "'['");
    let mut tbl = Struct::Table {
        entries: HashMap::new(),
    };
    parse_inner(parser, &mut tbl)?;
    parser.expect(|tv| matches!(tv, TokVal::RBrack), "']'");
    Ok(tbl)
}

fn parse_map_struct<'src>(parser: &mut Parser<'src>) -> Result<Struct, Error<'src>> {
    parser.expect(|tv| matches!(tv, TokVal::LBracePct), "'{%'");
    let mut map = Struct::Map {
        props: HashMap::new(),
    };
    parse_inner(parser, &mut map)?;
    parser.expect(|tv| matches!(tv, TokVal::RBracePct), "'%}'");
    Ok(map)
}

fn parse_struct<'src>(parser: &mut Parser<'src>) -> Result<Struct, Error<'src>> {
    let tok = parser.cur_expect()?;
    match tok.val {
        TokVal::LBrace => {
            parse_obj_struct(parser)
        },
        TokVal::LBrack => {
            parse_tbl_struct(parser)
        },
        TokVal::LBracePct => {
            parse_map_struct(parser)
        },
        _ => {
            Err(Error::detailed(153, String::from("Expected Struct"),
                                tok.clone(), format!("Found {:?}", tok)))
        },
    }
}

pub fn parse<'src>(input: &'src str,
                   lines: &'src Lines,
                   tokens: Vec<Token<'src>>) -> Result<Struct, Error<'src>> {
    let mut parser = Parser::new(input, lines, tokens);
    let mut root = Struct::Object {
        entries: HashMap::new(),
        props: HashMap::new(),
    };
    parse_inner(&mut parser, &mut root)?;
    Ok(root)
}


