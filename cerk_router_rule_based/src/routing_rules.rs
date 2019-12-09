use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// CloudEvent which could be used to route an event.
/// They are mapped to all implemented CloudEvents standards
/// (with some exceptions mentioned per field).
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CloudEventFields {
    Id,
    Type,
    /// not implemented in V0.2
    Source,
    Subject,
    Dataschema,
}

/// routing rules
///
/// They decide if an event get forwarded to a specified port.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RoutingRules {
    /// Routes the event to the destionation if all rules match
    And(Vec<RoutingRules>),

    /// Routes the event to the destionation if any rule matches
    Or(Vec<RoutingRules>),

    /// Pattern matching on field
    ///
    /// # Arguments
    ///
    /// * FieldName
    /// * String to compare to
    Exact(CloudEventFields, Option<String>),

    /// Pattern matching on field
    ///
    /// # Arguments
    ///
    /// * FieldName
    /// * String to search for
    Contains(CloudEventFields, String),

    /// Pattern matching on field
    ///
    /// # Arguments
    ///
    /// * FieldName
    /// * Strings with which the field should start with
    StartsWith(CloudEventFields, String),

    /// Pattern matching on field
    ///
    /// # Arguments
    ///
    /// * FieldName
    /// * Strings with which the field should end with
    EndsWith(CloudEventFields, String),
}

/// routing rules table
///
/// Routing rules indexed by the adapter that should receive the event
pub type RoutingTable = HashMap<String, RoutingRules>;

#[test]
fn serialize() {
    let rules = RoutingRules::Contains(CloudEventFields::Id, "1".to_string());

    let json = serde_json::to_string(&rules).unwrap();
    assert_eq!(json, "{\"Contains\":[\"Id\",\"1\"]}");
}

#[test]
fn deserialize() {
    let json = "{\"Contains\":[\"Id\",\"1\"]}";
    match serde_json::from_str::<RoutingRules>(&json) {
        Ok(rules) => assert_eq!(
            rules,
            RoutingRules::Contains(CloudEventFields::Id, "1".to_string())
        ),
        _ => assert!(false),
    }
}
