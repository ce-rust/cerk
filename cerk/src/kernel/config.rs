use anyhow::Result;
use serde::export::TryFrom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type ConfigHashMap = HashMap<String, Config>;

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
    U32(u32),
    Vec(Vec<Config>),
    HashMap(ConfigHashMap),
}

impl TryFrom<&Config> for u8 {
    type Error = anyhow::Error;
    fn try_from(value: &Config) -> Result<Self, Self::Error> {
        Ok(match value {
            Config::U8(v) => v.clone(),
            _ => bail!("expected U8"),
        })
    }
}

impl TryFrom<&Config> for u32 {
    type Error = anyhow::Error;
    fn try_from(value: &Config) -> Result<Self, Self::Error> {
        Ok(match value {
            Config::U32(v) => v.clone(),
            _ => bail!("expected U32"),
        })
    }
}

impl TryFrom<&Config> for String {
    type Error = anyhow::Error;
    fn try_from(value: &Config) -> Result<Self, anyhow::Error> {
        Ok(match value {
            Config::String(v) => v.to_string(),
            _ => bail!("expected String"),
        })
    }
}

impl TryFrom<&Config> for Vec<Config> {
    type Error = anyhow::Error;

    fn try_from(value: &Config) -> Result<Self, Self::Error> {
        Ok(match value {
            Config::Vec(v) => v.clone(),
            _ => bail!("expected Array"),
        })
    }
}

impl TryFrom<Config> for ConfigHashMap {
    type Error = anyhow::Error;
    fn try_from(value: Config) -> Result<Self, Self::Error> {
        Ok(match value {
            Config::HashMap(v) => v,
            _ => bail!("expected String"),
        })
    }
}

impl From<Config> for Option<ConfigHashMap> {
    fn from(value: Config) -> Self {
        ConfigHashMap::try_from(value).map_or_else(|_| None, |v| Some(v))
    }
}

/// Helper function to unwrap config to the inner types
pub trait ConfigHelpers {
    /// get a config value from the HashMap
    fn get_op_val_config<'a>(&'a self, key: &'static str) -> Result<Option<&'a Config>>;
    /// get a string value from the HashMap
    fn get_op_val_string<'a>(&'a self, key: &'static str) -> Result<Option<String>>;
    /// get a u8 value from the HashMap
    fn get_op_val_u8<'a>(&'a self, key: &'static str) -> Result<Option<u8>>;
    /// Get a u32 value from the HashMap. If it does not exist, it tries to get an u8, too.
    fn get_op_val_u32<'a>(&'a self, key: &'static str) -> Result<Option<u32>>;
    /// get a vec value from the HashMap
    fn get_op_val_vec<'a>(&'a self, key: &'static str) -> Result<Option<Vec<Config>>>;
}

impl ConfigHelpers for Config {
    fn get_op_val_config<'a>(&'a self, key: &'static str) -> Result<Option<&'a Config>> {
        Ok(match self {
            Config::HashMap(v) => v.get(key),
            _ => bail!("expected String"),
        })
    }

    fn get_op_val_string<'a>(&'a self, key: &'static str) -> Result<Option<String>> {
        self.get_op_val(key).into()
    }

    fn get_op_val_u8<'a>(&'a self, key: &'static str) -> Result<Option<u8>> {
        self.get_op_val(key).into()
    }

    fn get_op_val_u32<'a>(&'a self, key: &'static str) -> Result<Option<u32>> {
        match self.get_op_val(key).into() {
            Err(e) => match self.get_op_val_u8(key) {
                Err(_) => Err(e),
                Ok(v) => Ok(v.map(|v| v as u32)),
            },
            o => o,
        }
    }

    fn get_op_val_vec<'a>(&'a self, key: &'static str) -> Result<Option<Vec<Config>>> {
        self.get_op_val(key).into()
    }
}

impl Config {
    fn get_op_val<'a, T>(&'a self, key: &'static str) -> Result<Option<T>>
    where
        T: TryFrom<&'a Config, Error = anyhow::Error>,
    {
        let c = self.get_op_val_config(key)?;
        if let Some(c) = c {
            Ok(Some(T::try_from(c)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_string() -> Result<()> {
        let conf = Config::HashMap(
            [("test".to_string(), Config::String("val".to_string()))]
                .iter()
                .cloned()
                .collect(),
        );
        assert_eq!(conf.get_op_val_string("test")?, Some("val".to_string()));
        assert_eq!(conf.get_op_val_string("nonexsiting")?, None);
        Ok(())
    }

    #[test]
    fn get_string_fail() {
        assert!(Config::U8(1).get_op_val_string("nonexsiting").is_err());
        let config = Config::HashMap(
            [("test".to_string(), Config::U8(1))]
                .iter()
                .cloned()
                .collect(),
        );
        assert!(config.get_op_val_string("test").is_err());
    }

    #[test]
    fn get_u32() -> Result<()> {
        let conf = Config::HashMap(
            [("test".to_string(), Config::U8(3))]
                .iter()
                .cloned()
                .collect(),
        );
        assert_eq!(conf.get_op_val_u32("test")?, Some(3));
        assert_eq!(conf.get_op_val_u32("nonexsiting")?, None);
        Ok(())
    }
}
