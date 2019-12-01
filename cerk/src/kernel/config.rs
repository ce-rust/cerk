use std::collections::HashMap;

/// This object represents the configuration for a component.
/// It could be defined recursively.
pub enum Config {
    Null,
    Bool(bool),
    String(String),
    Array(Vec<Config>),
    Object(HashMap<String, Config>),
}
