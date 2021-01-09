use anyhow::{Context, Result};
use cerk::kernel::{
    BrokerEvent, Config, OutgoingCloudEvent, OutgoingCloudEventProcessed, ProcessingResult,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use cloudevents::Event;
use serde_json;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};

fn write_to_stream(connection: &mut Connection, event: &Event, max_tries: usize) -> Result<()> {
    if max_tries == 0 {
        bail!("too many failures while trying to connect to stream")
    }
    match connection.stream.as_ref() {
        None => {
            if let Some(listener) = connection.listener.as_ref() {
                let (socket, _) = listener.accept()?;
                connection.stream = Some(socket);
                write_to_stream(connection, event, max_tries - 1)
            } else {
                bail!("no stream set")
            }
        }
        Some(mut stream) => {
            let mut message = serde_json::to_string(event)?;
            message.push_str("\n");
            if let Err(e) = stream.write_all(message.as_bytes()) {
                debug!("failed to write to stream {:?}", e);
                connection.stream = None;
                write_to_stream(connection, event, max_tries - 1)
                    .with_context(|| format!("failed to write to stream {:?}", e))
            } else {
                Ok(())
            }
        }
    }
}

fn send_event_out(
    id: &InternalServerId,
    connection: &mut Connection,
    event: &OutgoingCloudEvent,
    sender_to_kernel: &BoxedSender,
) -> Result<()> {
    debug!("{} cloudevent received", id);
    let send_result = write_to_stream(connection, &event.cloud_event, 10);
    if event.args.delivery_guarantee.requires_acknowledgment() {
        sender_to_kernel.send(BrokerEvent::OutgoingCloudEventProcessed(
            OutgoingCloudEventProcessed {
                result: ProcessingResult::from(send_result),
                routing_id: event.routing_id.to_string(),
                sender_id: id.clone(),
            },
        ));
    }
    Ok(())
}

fn update_config(id: &String, config: &Config) -> Result<UnixListener> {
    info!("{} received ConfigUpdated", id);
    match config {
        Config::String(socket_path) => Ok(UnixListener::bind(socket_path)?),
        _ => bail!("{} received invalid config", id),
    }
}

#[derive(Default)]
struct Connection {
    listener: Option<UnixListener>,
    stream: Option<UnixStream>,
}

/// This is the main function to start the port.
///
/// This port writes CloudEvents to a UNIX Socket.
///
/// # Configurations
///
/// The Socket expects a `Config::String` as configuration.
/// The string should be a file path where the UNIX Socket should be created.
///
///
/// e.g. `Config::String(String::from("path/to/the/socket"))`
///
/// # Examples
///
/// * [UNIX Socket Example](https://github.com/ce-rust/cerk/tree/master/examples/examples/src/unix_socket)
///
pub fn port_output_unix_socket_json_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start output JSON over unix socket port with id {}", id);
    let mut connection = Connection::default();

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                match update_config(&id, &config).context("create connection on config update") {
                    Ok(l) => connection.listener = Some(l),
                    Err(e) => error!("{} ConfigUpdated failed {:?}", id, e),
                }
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                if let Err(e) = send_event_out(&id, &mut connection, &event, &sender_to_kernel) {
                    error!("{} was not able to send event out {:?}", id, e)
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the port.
pub static PORT_OUTPUT_UNIX_SOCKET: InternalServerFnRefStatic =
    &(port_output_unix_socket_json_start as InternalServerFn);

#[cfg(test)]
mod tests {
    use super::super::*;
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

    /// We send a CloudEvent to the port with `DeliveryGuarantee::default()`, it does not need to be acked.
    ///
    /// test prints `thread '<unnamed>' panicked at 'called Result::unwrap() on an Err value: RecvError'`
    /// this is okay -> we don't do a real shutdown of the port but just kill the communication channel
    #[test]
    fn send_unack_message() {
        let (send_to_port, recv) = new_channel_with_size(1);
        let (send, recv_from_port) = new_channel_with_size(1);
        thread::spawn(move || {
            PORT_OUTPUT_UNIX_SOCKET(ID.to_string(), recv, send);
        });
        send_to_port.send(BrokerEvent::OutgoingCloudEvent(build_event()));
        let response = recv_from_port.receive_timeout(Duration::from_millis(10));
        assert!(response.is_none());
    }

    /// We send a CloudEvent to the port with `DeliveryGuarantee::AtLeastOnce`, it needs to be acked.
    /// However, we haven't provide any config -> send nack
    ///
    /// test prints `thread '<unnamed>' panicked at 'called Result::unwrap() on an Err value: RecvError'`
    /// this is okay -> we don't do a real shutdown of the port but just kill the communication channel
    #[test]
    fn send_ack_message_receive_nack() {
        let (send_to_port, recv) = new_channel_with_size(1);
        let (send, recv_from_port) = new_channel_with_size(1);
        thread::spawn(move || {
            PORT_OUTPUT_UNIX_SOCKET(ID.to_string(), recv, send);
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
                    result: ProcessingResult::PermanentError
                }
            );
        } else {
            assert!(false, "response has wrong type");
        }
    }
}
