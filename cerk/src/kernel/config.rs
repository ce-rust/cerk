use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// This object represents the configuration for a component.
/// It can be defined recursively.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
#[serde(untagged)]
#[derive(Deserialize, Serialize)]
pub enum Config {
    /// empty configuration
    Null,
    Bool(bool),
    String(String),
    /// unsigned 8-bit number
    U8(u8),
    Vec(Vec<Config>),
    HashMap(HashMap<String, Config>),
}
