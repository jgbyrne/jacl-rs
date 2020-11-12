use indexmap::map::IndexMap;

#[derive(Clone, Debug)]
pub enum Value {
    Key(String),
    Tuple(Vec<Value>),

    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

pub type Entries = IndexMap<String, Option<Struct>>;
pub type Props = IndexMap<String, Value>;

#[derive(Clone, Debug)]
pub enum Struct {
    Object { entries: Entries, props: Props},
    Table { entries: Entries },
    Map { props: Props },
}
