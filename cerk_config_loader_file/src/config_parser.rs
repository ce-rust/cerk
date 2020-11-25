use anyhow::Result;
use cerk::kernel::{BrokerEvent, Config};
use serde::Deserialize;
use std::collections::HashMap;
use std::vec::Vec;

#[derive(Deserialize, Debug, PartialEq)]
struct Configuration {
    routing_rules: Config,
    ports: HashMap<String, Config>,
}

fn parse_json_to_config(content: String) -> Result<Configuration> {
    let config = serde_json::from_str(content.as_str())?;
    Ok(config)
}

pub fn parse_json_to_events(content: String) -> Result<Vec<BrokerEvent>> {
    let config = parse_json_to_config(content)?;
    let mut events: Vec<BrokerEvent> = config
        .ports
        .iter()
        .map(|(port, config)| BrokerEvent::ConfigUpdated(config.clone(), port.to_string()))
        .collect();
    events.push(BrokerEvent::ConfigUpdated(
        config.routing_rules,
        String::from("router"),
    ));
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn compare(parsed: Configuration, json: String) -> Result<()> {
        let config = parse_json_to_config(json.clone())?;

        assert_eq!(
            config, parsed,
            "config was not equal\njson:\n{}\nconfig:\n{:?}\n",
            json, config
        );
        Ok(())
    }

    #[rstest(json, parsed,
        case("null", Config::Null),
        case("false", Config::Bool(false)),
        case("true", Config::Bool(true)),
        case("0", Config::U8(0)),
        case("42", Config::U8(42)),
        case("[]", Config::Vec(vec![])),
        case("[42]", Config::Vec(vec![Config::U8(42)])),
    )]
    fn parse_minimal_all_types(json: &str, parsed: Config) -> Result<()> {
        let json = format!(
            "{}{}{}",
            r#"
        {
          "routing_rules": "#,
            json,
            r#",
          "ports": {}
        }
        "#
        )
        .to_string();
        let parsed_full = Configuration {
            routing_rules: parsed,
            ports: HashMap::default(),
        };
        compare(parsed_full, json)
    }

    #[test]
    fn parse_ports() -> Result<()> {
        let json = r#"
        {
          "routing_rules": null,
          "ports": {
            "dummy": null
          }
        }
        "#
        .to_string();

        let mut ports = HashMap::new();
        ports.insert("dummy".to_string(), Config::Null);

        let config = Configuration {
            routing_rules: Config::Null,
            ports,
        };
        compare(config, json)
    }
}
