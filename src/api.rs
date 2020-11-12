use indexmap::map::IndexMap;

use crate::Lines;
use crate::error::Error;
use crate::types::{Entries, Props, Struct};
pub use crate::types::Value;


#[derive(Debug)]
pub struct JaclError<'src> {
    internal: Error<'src>,
    input: &'src str,
    lines: Lines,
}

impl<'src> JaclError<'src> {
    pub fn from_error(err: Error<'src>, input: &'src str, lines: Lines) -> JaclError<'src> {
        JaclError {
            internal: err,
            input,
            lines,
        }
    }

    pub fn render(&self) -> String {
        self.internal.render(self.input, &self.lines)
    }
}

fn transform_entry<'s, 'jacl: 's>(entry: &'s Option<Struct>, jacl: &'jacl Jacl) -> Option<JaclStruct<'s>> {
    match entry {
        Some(Struct::Object { entries, props }) => {
            Some(JaclStruct::Object(
                Object { jacl, entries, props }
            ))
        },
        Some(Struct::Table { entries }) => {
            Some(JaclStruct::Table(
                Table { jacl, entries }
            ))
        },
        Some(Struct::Map { props }) => {
            Some(JaclStruct::Map(
                Map { jacl, props }
            ))
        }
        None => {
            None
        }
    }
}

fn transform_entries<'s, 'jacl: 's>(entries: &'s Entries, jacl: &'jacl Jacl) -> Vec<(Option<&'s String>, Option<JaclStruct<'s>>)> {
    entries.iter()
        .map(|(key, entry)| {
            let strct = transform_entry(entry, jacl);
            if key.starts_with('#') {
               (None, strct)
            }
            else {
               (Some(key), strct)
            }
        }).collect::<Vec<(Option<&String>, Option<JaclStruct>)>>()
}

pub trait EntryStruct<'s> {
    fn entries(&self) -> Vec<(Option<&String>, Option<JaclStruct<'s>>)>;
    fn get_entry(&self, key: &str) -> Option<JaclStruct<'s>>;
    fn resolve_key(&self, key: &Value) -> Option<JaclStruct<'s>>;
}

pub trait PropertyStruct {
    fn properties(&self) -> Vec<(&String, &Value)>;
    fn get_property(&self, val: &str) -> Option<&Value>;
}

#[derive(Debug)]
pub struct Object<'s> {
    jacl: &'s Jacl,
    entries: &'s Entries,
    props: &'s Props,
}

impl<'s> EntryStruct<'s> for Object<'s> {
    fn entries(&self) -> Vec<(Option<&String>, Option<JaclStruct<'s>>)> {
        transform_entries(self.entries, self.jacl)
    }

    fn get_entry(&self, key: &str) -> Option<JaclStruct<'s>> {
        match self.entries.get(key) {
            Some(entry) => transform_entry(entry, self.jacl),
            None => None,
        }
    }

    fn resolve_key(&self, key: &Value) -> Option<JaclStruct<'s>> {
        if let Value::Key(key) = key {
            self.get_entry(&key)
        }
        else {
            None
        }
    }
}

impl PropertyStruct for Object<'_> {
    fn properties(&self) -> Vec<(&String, &Value)> {
        self.props.iter().collect::<Vec<(&String, &Value)>>()
    }

    fn get_property(&self, val: &str) -> Option<&Value> {
        self.props.get(val)
    }
}

impl<'s> Object<'s> {
    pub fn resolve_property<S: AsRef<str>>(&self, val: S) -> Option<JaclStruct<'s>> {
        match self.props.get(val.as_ref()) {
            Some(key@Value::Key(..)) => self.resolve_key(key),
            Some(..) => None,
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct Table<'s> {
    jacl: &'s Jacl,
    entries: &'s Entries,
}

impl<'s> EntryStruct<'s> for Table<'s> {
    fn entries(&self) -> Vec<(Option<&String>, Option<JaclStruct<'s>>)> {
        transform_entries(self.entries, self.jacl)
    }

    fn get_entry(&self, key: &str) -> Option<JaclStruct<'s>> {
        match self.entries.get(key) {
            Some(entry) => transform_entry(entry, self.jacl),
            None => None,
        }
    }

    fn resolve_key(&self, key: &Value) -> Option<JaclStruct<'s>> {
        if let Value::Key(key) = key {
            self.get_entry(&key)
        }
        else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Map<'s> {
    jacl: &'s Jacl,
    props: &'s Props,
}

impl PropertyStruct for Map<'_> {
    fn properties(&self) -> Vec<(&String, &Value)> {
        self.props.iter().collect::<Vec<(&String, &Value)>>()
    }

    fn get_property(&self, val: &str) -> Option<&Value> {
        self.props.get(val)
    }
}

#[derive(Debug)]
pub enum JaclStruct<'s> {
    Object(Object<'s>),
    Table(Table<'s>),
    Map(Map<'s>),
}

impl<'s> JaclStruct<'s> {
    pub fn as_entry_struct(&self) -> Option<&dyn EntryStruct> {
        match self {
            JaclStruct::Map(_) => None,
            JaclStruct::Object(strct) => Some(strct as &dyn EntryStruct),
            JaclStruct::Table(strct) => Some(strct as &dyn EntryStruct),
        }
    }

    pub fn as_property_struct(&self) -> Option<&dyn PropertyStruct> {
        match self {
            JaclStruct::Map(strct) => Some(strct as &dyn PropertyStruct),
            JaclStruct::Object(strct) => Some(strct as &dyn PropertyStruct),
            JaclStruct::Table(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct Jacl {
    inr: Struct 
}

impl Jacl {
    pub fn init(data: Struct) -> Jacl {
        Jacl {
            inr: data,
        }
    }

    pub fn root<'s, 'jacl: 's>(&'jacl self) -> Object<'s> {
        if let Struct::Object { entries, props } = &self.inr {
            Object {
                jacl: &self,
                entries: &entries,
                props: &props,
            }
        }
        else {
            panic!();
        }
    }
}
