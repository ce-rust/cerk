use crate::routing_rules::{CloudEventFields, RoutingRules, RoutingTable};
use cerk::kernel::{BrokerEvent, Config};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::CloudEvent;
use serde_json;
use serde_json::error::Error as SerdeErrorr;

fn compare_field<F>(field: &CloudEventFields, cloud_event: &CloudEvent, compare: F) -> bool
where
    F: for<'a> Fn(Option<&'a str>) -> bool,
{
    match field {
        CloudEventFields::Id => compare(Some(get_event_field!(cloud_event, event_id))),
        CloudEventFields::Source => compare(Some(get_event_field!(cloud_event, source))),
        CloudEventFields::Subject => match cloud_event {
            CloudEvent::V0_2(_) => false,
            CloudEvent::V1_0(event) => compare(event.subject()),
        },
        CloudEventFields::Dataschema => match cloud_event {
            CloudEvent::V0_2(event) => compare(event.schema_url()),
            CloudEvent::V1_0(event) => compare(event.dataschema()),
        },
        CloudEventFields::Type => compare(Some(get_event_field!(cloud_event, event_type))),
    }
}

fn route_to_port(rules: &RoutingRules, cloud_event: &CloudEvent) -> bool {
    match rules {
        RoutingRules::And(rules) => rules.iter().all(|rule| route_to_port(rule, cloud_event)),
        RoutingRules::Or(rules) => rules.iter().any(|rule| route_to_port(rule, cloud_event)),
        RoutingRules::Exact(field, value) => compare_field(field, cloud_event, |field| {
            field == value.as_ref().map(|s| &**s)
        }),
        RoutingRules::Contains(field, value) => compare_field(field, cloud_event, |field| {
            field.map_or(false, |f| f.contains(value.as_str()))
        }),
        RoutingRules::StartsWith(field, value) => compare_field(field, cloud_event, |field| {
            field.map_or(false, |f| f.starts_with(value.as_str()))
        }),
        RoutingRules::EndsWith(field, value) => compare_field(field, cloud_event, |field| {
            field.map_or(false, |f| f.ends_with(value.as_str()))
        }),
    }
}

fn route_event(
    sender_to_kernel: &BoxedSender,
    port_config: &RoutingTable,
    cloud_event: &CloudEvent,
) {
    for (port_id, rules) in port_config.iter() {
        if route_to_port(rules, cloud_event) {
            sender_to_kernel.send(BrokerEvent::OutgoingCloudEvent(
                cloud_event.clone(),
                port_id.clone(),
            ))
        }
    }
}

fn parse_config(config_update: String) -> Result<RoutingTable, SerdeErrorr> {
    serde_json::from_str::<RoutingTable>(&config_update)
}

/// The rule-based router routes event based on the given configuration.
///
/// One configuration tree per output port should be configured. The configurations are given in a tree format.
/// The operations `And`, `Or`, `Contains`, `StartsWith` and more are possible.
///
/// # Configurations
///
/// The Socket expects a `Config::String` as configuration.
/// The string should be a json deserialized `routing_rules::RoutingTable;`.
///
/// minimal: `Config::String("{}".to_string())`
///
/// ## Example
///
/// ```
/// use serde_json;
/// use cerk_router_rule_based::{CloudEventFields, RoutingRules, RoutingTable};
///
/// let routing_rules: RoutingTable = [(
///   "dummy-logger-output".to_string(),
///   RoutingRules::And(vec![
///     RoutingRules::Exact(
///         CloudEventFields::Source,
///         Some("dummy.sequence-generator".to_string()),
///     ),
///     RoutingRules::EndsWith(CloudEventFields::Id, "0".to_string()),
///   ]),
/// )]
/// .iter()
/// .cloned()
/// .collect();
///
/// let routing_configs = serde_json::to_string(&routing_rules).unwrap();
/// ```
///
/// # Examples
///
/// * [Rule Based Routing Example](https://github.com/ce-rust/cerk/tree/master/examples/src/rule_based_routing)
///
pub fn router_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start broadcast router with id {}", id);
    let mut config: Option<RoutingTable> = None;
    loop {
        match inbox.receive() {
            BrokerEvent::Init => info!("{} initiated", id),
            BrokerEvent::IncommingCloudEvent(_, cloud_event) => {
                if let Some(config) = config.as_ref() {
                    route_event(&sender_to_kernel, config, &cloud_event);
                } else {
                    error!("No configs defined yet, event will be droped");
                }
            }
            BrokerEvent::ConfigUpdated(updated_config, _) => {
                if let Config::String(string_config) = updated_config {
                    match parse_config(string_config) {
                        Ok(parsed_config) => config = Some(parsed_config),
                        Err(err) => panic!("was not able to parse configs {:?}", err),
                    }
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing_rules::{CloudEventFields, RoutingRules};

    #[test]
    fn rout_to_port_by_id() {
        let rule = RoutingRules::Exact(CloudEventFields::Id, Some("1234".to_string()));
        // positive
        assert!(route_to_port(
            &rule,
            &cloudevent!(
                event_id: "1234",
                event_type: "test type",
                source: "testi",
            )
            .unwrap()
        ));
        // negative
        assert!(!route_to_port(
            &rule,
            &cloudevent!(
                event_id: "12345",
                event_type: "test type",
                source: "testi",
            )
            .unwrap()
        ));
    }

    #[test]
    fn rout_to_port_by_type_and_source() {
        let rule = RoutingRules::And(vec![
            RoutingRules::StartsWith(CloudEventFields::Type, "testtype".to_string()),
            RoutingRules::Contains(CloudEventFields::Source, "testsource".to_string()),
        ]);
        // positive
        assert!(route_to_port(
            &rule,
            &cloudevent!(
                event_id: "1",
                event_type: "testtype1",
                source: "1testsource1",
            )
            .unwrap()
        ));
        // negative
        // positive
        assert!(!route_to_port(
            &rule,
            &cloudevent!(
                event_id: "1",
                event_type: "1testtype",
                source: "1test1source1",
            )
            .unwrap()
        ));
    }
}
