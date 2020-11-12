use indexmap::map::IndexMap;

use crate::Lines;
use crate::tokeniser::{Token, TokVal};
use crate::error::Error;
use crate::types::{Struct, Value, Entries, Props};

impl Struct {
    fn entries_extend<'ln, 'src>(parser: &mut Parser<'ln, 'src>,
                            ex_entries: &mut Entries,
                            new_entries: &Entries) -> Result<(), Error<'src>> {
        for (new_key, new_entry) in new_entries.iter() {
            if let Some(Some(ex_entry)) = ex_entries.get_mut(new_key) {
                if let Some(new_entry) = new_entry {
                    ex_entry.extend(parser, new_key, new_entry.clone())?;
                }
                else {
                    return Err(Error::detailed(162, format!("Entry {} redefined with no new data", new_key),
                               parser.cur_expect()?.clone(), String::from("Remove this redefinition")));
                }
            }
            else {
                ex_entries.insert(new_key.clone(), new_entry.clone());
            }
        }
        Ok(())
    }

    fn extend<'ln, 'src>(&mut self,
                    parser: &mut Parser<'ln, 'src>,
                    name: &str, new: Struct) -> Result<(), Error<'src>>{
        match self {
            Struct::Object { entries: ex_entries,
                                  props: ex_props } => {
                if let Struct::Object { entries: new_entries,
                                        props: new_props } = new {
                    ex_props.extend(new_props);
                    Struct::entries_extend(parser, ex_entries, &new_entries)
                }
                else {
                     Err(Error::detailed(161, format!("Entry {} already defined as Object", name),
                         parser.cur_expect()?.clone(), String::from("Make this entry an Object")))
                }
            },
            Struct::Table { entries: ex_entries } => {
                if let Struct::Table { entries: new_entries } = new {
                    Struct::entries_extend(parser, ex_entries, &new_entries)
                }
                else {
                    Err(Error::detailed(160, format!("Entry {} already defined as Table", name),
                        parser.cur_expect()?.clone(), String::from("Make this entry a Table")))
                }
           },
           Struct::Map {..} => {
               Err(Error::detailed(159, format!("Entry {} already defined as Map", name),
                   parser.cur_expect()?.clone(), String::from("Make this entry a Map")))
           }
        }
    }
}

enum RValue {
    Value(Value),
    Struct(Struct),
}

struct Parser<'ln, 'src: 'ln> {
    input: &'src str,
    lines: &'ln Lines,
    tokens: Vec<Token<'src>>,

    ptr: usize,
}

impl<'ln, 'src> Parser<'ln, 'src> {
    fn new(input: &'src str, lines: &'ln Lines, tokens: Vec<Token<'src>>) -> Parser<'ln, 'src> {
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

fn parse_val<'ln, 'src>(parser: &mut Parser<'ln, 'src>) -> Result<Value, Error<'src>> {
    match parser.cur_expect()?.val {
        TokVal::Name(name) => {
            parser.step();
            Ok(Value::Key(name.to_string()))
        }
        TokVal::Dollar => {
            parser.step();
            if let TokVal::Name(name) = parser.expect(
                |tv| matches!(tv, TokVal::Name(_)), "name")?.val {
                Ok(Value::Var(name.to_string()))
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
        TokVal::LParen => {
            parser.step();
            let mut tuple: Vec<Value> = Vec::new();
            loop {
                tuple.push(parse_val(parser)?);
                let tok = parser.cur_expect()?;
                match tok.val {
                    TokVal::RParen => {
                        parser.step();
                        break;
                    },
                    TokVal::Comma => {
                        parser.step();
                    },
                    _ => {
                        return Err(
                            Error::detailed(164,
                                String::from("Invalid syntax inside tuple"),
                                tok.clone(),
                                String::from("Expected ',' or ')'"))
                        );
                    }
                }
            }
            Ok(Value::Tuple(tuple))
        },
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

fn parse_rval<'ln, 'src>(parser: &mut Parser<'ln, 'src>) -> Result<RValue, Error<'src>> {
    match parser.cur_expect()?.val {
        TokVal::LBrace | TokVal::LBrack | TokVal::LBracePct => {
            Ok(RValue::Struct(parse_struct(parser)?))
        }
        _ => {
            Ok(RValue::Value(parse_val(parser)?))
        }
    }
}

fn parse_rhs<'ln, 'src>(parser: &mut Parser<'ln, 'src>, strct: &mut Struct, names: Vec<String>) -> Result<(), Error<'src>> {
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

fn parse_single_binding<'ln, 'src>(parser: &mut Parser<'ln, 'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    if let TokVal::Name(name) = parser.cur_expect()?.val {
        parser.step();
        parse_rhs(parser, strct, vec![name.to_string()])
    }
    else {
        Err(Error::basic(1, String::from("Internal Parser Error: TokVal was not Name")))
    }
}

fn parse_multiple_binding<'ln, 'src>(parser: &mut Parser<'ln, 'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    let mut names = Vec::new();
    loop {
        if let TokVal::Name(name) = parser.expect(|tv| matches!(tv, TokVal::Name{..}), "name")?.val {
            names.push(name.to_string());
            let tok = parser.cur_expect()?;
            match tok.val {
                TokVal::Equals => {
                    break;
                },
                TokVal::Comma => {
                    parser.step();
                    continue;
                },
                _ => {
                    return Err(Error::detailed(163, String::from("Expected '=' or ','"),
                            tok.clone(), String::from("Could not parse this token")));
                }
            }
        }
        else { 
            return Err(Error::basic(1, String::from("Internal Error: Reached the unreachable!")));
        }
    }
    parse_rhs(parser, strct, names)
}

/* Parse Entries */

fn parse_single_entry<'ln, 'src>(parser: &mut Parser<'ln, 'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    match strct {
        Struct::Object { entries, props: _ } |
        Struct::Table { entries } => {
            if let TokVal::Name(name) = parser.cur_expect()?.val {
                parser.step();
                let strct = parse_struct(parser)?;
                if let Some(extant) = entries.get_mut(name) {
                    match extant {
                        Some(extant) => {
                            extant.extend(parser, name, strct)
                        },
                        None => {
                            entries.insert(name.to_string(), Some(strct));
                            Ok(())
                        }
                    }
                }
                else {
                    entries.insert(name.to_string(), Some(strct));
                    Ok(())
                }
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

fn parse_compound_entry<'ln, 'src>(parser: &mut Parser<'ln, 'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    match strct {
        Struct::Object { entries, props: _ } |
        Struct::Table { entries } => {
            let mut names = Vec::new();
            loop {
                if let TokVal::Name(name) = parser.expect(
                        |tv| matches!(tv, TokVal::Name(..)), "name")?.val {
                    names.push(name); 
                }
                else {
                    return Err(
                        Error::basic(1, String::from("Internal Error: Reached the unreachable!"))
                    );
                }
                if !matches!(parser.cur_expect()?.val, TokVal::Plus) { break; }
                parser.step();
            }

            let strct = parse_struct(parser)?;
            for name in names {
                if let Some(extant) = entries.get_mut(name) {
                    match extant {
                        Some(extant) => {
                            extant.extend(parser, name, strct.clone())?;
                        },
                        None => {
                            entries.insert(name.to_string(), Some(strct.clone()));
                        }
                    }
                }
                else {
                    entries.insert(name.to_string(), Some(strct.clone()));
                }
            }
            Ok(())
        },
        Struct::Map { props: _ } => {
            Err(Error::detailed(156, String::from("Maps cannot contain Entries"),
                                parser.cur_expect()?.clone(), String::from("Remove this entry")))
        }
    }
}

fn parse_wild_entry<'ln, 'src>(parser: &mut Parser<'ln, 'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    Err(Error::basic(1, String::from("Internal Error: Unimplemented")))
}

fn parse_prop_entry<'ln, 'src>(parser: &mut Parser<'ln, 'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    Err(Error::basic(1, String::from("Internal Error: Unimplemented")))
}

fn parse_empty_entry<'ln, 'src>(parser: &mut Parser<'ln, 'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
    match strct {
        Struct::Object { entries, props: _ } |
        Struct::Table { entries } => {
            let tok = parser.cur_expect()?;
            if let TokVal::Name(name) = tok.val {
                parser.step();
                match entries.get(name) {
                    Some(_) => {
                        Err(Error::detailed(162, format!("Entry {} redefined with no new data", name),
                            tok.clone(), String::from("Remove this redefinition")))
                    },
                    None => {
                        entries.insert(name.to_string(), None);
                        Ok(())
                    }
                }
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

fn parse_anon_entry<'ln, 'src>(parser: &mut Parser<'ln, 'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
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

fn parse_inner<'ln, 'src>(parser: &mut Parser<'ln, 'src>, strct: &mut Struct) -> Result<(), Error<'src>> {
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


fn parse_obj_struct<'ln, 'src>(parser: &mut Parser<'ln, 'src>) -> Result<Struct, Error<'src>> {
    parser.expect(|tv| matches!(tv, TokVal::LBrace), "'{'")?;
    let mut obj = Struct::Object {
        entries: IndexMap::new(),
        props: IndexMap::new(),
    };
    parse_inner(parser, &mut obj)?;
    parser.expect(|tv| matches!(tv, TokVal::RBrace), "'}'")?;
    Ok(obj)
}

fn parse_tbl_struct<'ln, 'src>(parser: &mut Parser<'ln, 'src>) -> Result<Struct, Error<'src>> {
    parser.expect(|tv| matches!(tv, TokVal::LBrack), "'['")?;
    let mut tbl = Struct::Table {
        entries: IndexMap::new(),
    };
    parse_inner(parser, &mut tbl)?;
    parser.expect(|tv| matches!(tv, TokVal::RBrack), "']'")?;
    Ok(tbl)
}

fn parse_map_struct<'ln, 'src>(parser: &mut Parser<'ln, 'src>) -> Result<Struct, Error<'src>> {
    parser.expect(|tv| matches!(tv, TokVal::LBracePct), "'{%'")?;
    let mut map = Struct::Map {
        props: IndexMap::new(),
    };
    parse_inner(parser, &mut map)?;
    parser.expect(|tv| matches!(tv, TokVal::RBracePct), "'%}'")?;
    Ok(map)
}

fn parse_struct<'ln, 'src>(parser: &mut Parser<'ln, 'src>) -> Result<Struct, Error<'src>> {
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

pub fn parse<'ln, 'src>(input: &'src str,
                   lines: &'ln Lines,
                   tokens: Vec<Token<'src>>) -> Result<Struct, Error<'src>> {
    let mut parser = Parser::new(input, lines, tokens);
    let mut root = Struct::Object {
        entries: IndexMap::new(),
        props: IndexMap::new(),
    };
    parse_inner(&mut parser, &mut root)?;
    Ok(root)
}


