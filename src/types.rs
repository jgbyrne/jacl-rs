use indexmap::map::IndexMap;

#[derive(Clone, Debug)]
pub enum Value {
    Key(String),
    Var(String),
    Foreign(String),
    Tuple(Vec<Value>),

    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

pub type Entries = IndexMap<String, Option<Struct>>;
pub type Props = IndexMap<String, Value>;

#[derive(Clone, Debug)]
pub enum Struct {
    Object { entries: Entries, props: Props},
    Table { entries: Entries },
    Map { props: Props },
}
