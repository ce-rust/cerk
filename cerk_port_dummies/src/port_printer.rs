use anyhow::Result;
use cerk::kernel::{
    BrokerEvent, OutgoingCloudEvent, OutgoingCloudEventProcessed, ProcessingResult,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use serde_json;

/// This port prints the CloudEvent id to the logger.
/// This port is for testing!
///
/// # Examples
///
/// * [Hello World Example](https://github.com/ce-rust/cerk/tree/master/examples/src/hello_world)
/// * [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/src/mqtt/)
///
pub fn port_printer_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start printer port with id {}", id);
    loop {
        match inbox.receive() {
            BrokerEvent::Init => info!("{} initiated", id),
            BrokerEvent::ConfigUpdated(_, _) => info!("{} received ConfigUpdated", id),
            BrokerEvent::OutgoingCloudEvent(event) => {
                if let Err(e) = print_event(&id, &sender_to_kernel, &event) {
                    error!("{} was not able to print event {:?}", id, e)
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

fn print_event(
    id: &String,
    sender_to_kernel: &BoxedSender,
    event: &OutgoingCloudEvent,
) -> Result<()> {
    info!(
        "{} received cloud event: {}!",
        id,
        serde_json::to_string(&event.cloud_event)?,
    );
    if event.args.delivery_guarantee.requires_acknowledgment() {
        sender_to_kernel.send(BrokerEvent::OutgoingCloudEventProcessed(
            OutgoingCloudEventProcessed {
                result: ProcessingResult::Successful,
                routing_id: event.routing_id.to_string(),
                sender_id: id.clone(),
            },
        ))
    }
    Ok(())
}

/// This is the pointer for the main function to start the port.
pub static PORT_PRINTER: InternalServerFnRefStatic = &(port_printer_start as InternalServerFn);

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::port_printer::print_event;
    use anyhow::Result;
    use cerk::kernel::{
        BrokerEvent, CloudEventRoutingArgs, DeliveryGuarantee, OutgoingCloudEvent,
        OutgoingCloudEventProcessed, ProcessingResult,
    };
    use cerk_runtime_threading::channel::new_channel_with_size;
    use cloudevents::Event;
    use std::thread;
    use std::time::Duration;

    const ID: &'static str = "port-id";

    fn build_event() -> OutgoingCloudEvent {
        OutgoingCloudEvent {
            routing_id: String::from("abc"),
            cloud_event: Event::default(),
            destination_id: ID.to_string(),
            args: CloudEventRoutingArgs::default(),
        }
    }

    #[test]
    fn print_valid_event() -> Result<()> {
        let (send, _) = new_channel_with_size(1);
        print_event(&ID.to_string(), &send, &build_event())
    }

    /// We send a CloudEvent to the port with `DeliveryGuarantee::default()`, it does not need to be acked.
    ///
    /// `thread '<unnamed>' panicked at 'called Result::unwrap() on an Err value: RecvError'`
    /// this is okay -> we don't do a real shutdown of the port but just kill the communication channel
    #[test]
    fn print_unack_message() {
        let (send_to_port, recv) = new_channel_with_size(1);
        let (send, recv_from_port) = new_channel_with_size(1);
        thread::spawn(move || {
            PORT_PRINTER(ID.to_string(), recv, send);
        });
        send_to_port.send(BrokerEvent::OutgoingCloudEvent(build_event()));
        let response = recv_from_port.receive_timeout(Duration::from_millis(10));
        assert!(response.is_none());
    }

    /// We send a CloudEvent to the port with `DeliveryGuarantee::AtLeastOnce`, it needs to be acked.
    ///
    /// `thread '<unnamed>' panicked at 'called Result::unwrap() on an Err value: RecvError'`
    /// this is okay -> we don't do a real shutdown of the port but just kill the communication channel
    #[test]
    fn print_ack_message() {
        let (send_to_port, recv) = new_channel_with_size(1);
        let (send, recv_from_port) = new_channel_with_size(1);
        thread::spawn(move || {
            PORT_PRINTER(ID.to_string(), recv, send);
        });
        let mut event = build_event();
        event.args.delivery_guarantee = DeliveryGuarantee::AtLeastOnce;
        send_to_port.send(BrokerEvent::OutgoingCloudEvent(event.clone()));
        let response = recv_from_port.receive_timeout(Duration::from_millis(10));
        assert!(response.is_some());

        if let BrokerEvent::OutgoingCloudEventProcessed(e) = response.unwrap() {
            assert_eq!(
                e,
                OutgoingCloudEventProcessed {
                    routing_id: event.routing_id.to_string(),
                    sender_id: ID.to_string(),
                    result: ProcessingResult::Successful
                }
            );
        } else {
            assert!(false, "response has wrong type");
        }
    }
}
