use crate::routing_rules::{CloudEventFields, RoutingRules, RoutingTable};
use cerk::kernel::{BrokerEvent, Config, IncomingCloudEvent, OutgoingCloudEvent, RoutingResult};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::{AttributesReader, Event};
use serde_json;
use serde_json::error::Error as SerdeErrorr;

fn compare_field<F>(field: &CloudEventFields, cloud_event: &Event, compare: F) -> bool
where
    F: for<'a> Fn(Option<&'a str>) -> bool,
{
    match field {
        CloudEventFields::Id => compare(Some(cloud_event.id())),
        CloudEventFields::Source => compare(Some(cloud_event.source().as_str())),
        CloudEventFields::Subject => cloud_event
            .subject()
            .and_then(|s| Some(compare(Some(s))))
            .unwrap_or_else(|| false),
        CloudEventFields::Dataschema => compare(cloud_event.dataschema().map(|s| s.as_str())),
        CloudEventFields::Type => compare(Some(cloud_event.ty())),
    }
}

fn route_to_port(rules: &RoutingRules, cloud_event: &Event) -> bool {
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
    event: IncomingCloudEvent,
    sender_to_kernel: &BoxedSender,
    port_config: &RoutingTable,
) {
    let IncomingCloudEvent {
        cloud_event,
        routing_id,
        incoming_id,
        args,
    } = event;
    let routing: Vec<_> = port_config
        .iter()
        .filter(|(_, rules)| route_to_port(rules, &cloud_event))
        .map(|(port_id, _)| OutgoingCloudEvent {
            routing_id: routing_id.clone(),
            cloud_event: cloud_event.clone(),
            destination_id: port_id.clone(),
            args: args.clone(),
        })
        .collect();
    sender_to_kernel.send(BrokerEvent::RoutingResult(RoutingResult {
        routing_id,
        incoming_id,
        routing,
        args,
    }))
}

fn parse_config(config_update: String) -> Result<RoutingTable, SerdeErrorr> {
    serde_json::from_str::<RoutingTable>(&config_update)
}

/// This is the main function to start the router.
pub fn router_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start broadcast router with id {}", id);
    let mut config: Option<RoutingTable> = None;
    loop {
        match inbox.receive() {
            BrokerEvent::Init => info!("{} initiated", id),
            BrokerEvent::IncomingCloudEvent(event) => {
                if let Some(config) = config.as_ref() {
                    route_event(event, &sender_to_kernel, config);
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
    use cloudevents::{EventBuilder, EventBuilderV10};

    #[test]
    fn rout_to_port_by_id() {
        let rule = RoutingRules::Exact(CloudEventFields::Id, Some("1234".to_string()));
        // positive
        assert!(route_to_port(
            &rule,
            &EventBuilderV10::new()
                .id("1234")
                .ty("test type")
                .source("http://example.com/testi")
                .build()
                .unwrap(),
        ));
        // negative
        assert!(!route_to_port(
            &rule,
            &EventBuilderV10::new()
                .id("12345")
                .ty("test type")
                .source("http://example.com/testi")
                .build()
                .unwrap(),
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
            &EventBuilderV10::new()
                .id("1")
                .ty("testtype1")
                .source("http://example.com/1testsource1")
                .build()
                .unwrap(),
        ));
        // negative
        // positive
        assert!(!route_to_port(
            &rule,
            &EventBuilderV10::new()
                .id("1")
                .ty("1testtype")
                .source("http://example.com/1test1source1")
                .build()
                .unwrap(),
        ));
    }
}
