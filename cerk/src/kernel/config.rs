use std::collections::HashMap;

pub enum Config {
    Null,
    Bool(bool),
    // Number(Number),
    String(String),
    Array(Vec<Config>),
    Object(HashMap<String, Config>),
}
