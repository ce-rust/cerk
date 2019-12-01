use std::collections::HashMap;

/// This object represents the configuration for a component.
/// It can be defined recursively.
#[allow(missing_docs)]
pub enum Config {
    /// empty configuration
    Null,
    Bool(bool),
    String(String),
    Array(Vec<Config>),
    Object(HashMap<String, Config>),
}
