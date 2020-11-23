use cerk::kernel::{BrokerEvent, Config};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::Event;
use serde_json;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};

fn write_to_stream(
    listener: &UnixListener,
    stream: Option<UnixStream>,
    event: &Event,
    max_tries: usize,
) -> Option<UnixStream> {
    if max_tries == 0 {
        panic!("too many failures while trying to connect to stream");
    }
    match stream {
        None => match listener.accept() {
            Ok((socket, _)) => write_to_stream(listener, Some(socket), event, max_tries - 1),
            Err(err) => panic!(err),
        },
        Some(mut stream) => match serde_json::to_string(event) {
            Ok(mut message) => {
                message.push_str("\n");
                if let Err(_) = stream.write_all(message.as_bytes()) {
                    write_to_stream(listener, None, event, max_tries - 1)
                } else {
                    Some(stream)
                }
            }
            Err(err) => {
                error!("serialization filed: {:?}", err);
                Some(stream)
            }
        },
    }
}

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
/// * [UNIX Socket Example](https://github.com/ce-rust/cerk/tree/master/examples/src/unix_socket)
///
/// # Limitations
///
/// * **reliability** this port does not support any `DeliveryGuarantee` other then `Unspecified` and so does never send a `OutgoingCloudEventProcessed` message
///
pub fn port_output_unix_socket_json_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    _sender_to_kernel: BoxedSender,
) {
    info!("start output JSON over unix socket port with id {}", id);
    let mut listener: Option<UnixListener> = None;
    let mut stream: Option<UnixStream> = None;

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", id);
                match config {
                    Config::String(socket_path) => {
                        listener = Some(UnixListener::bind(socket_path).unwrap());
                    }
                    _ => error!("{} received invalide config", id),
                };
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                debug!("{} cloudevent received", id);
                match listener.as_ref() {
                    Some(listener) => {
                        stream = write_to_stream(listener, stream, &event.cloud_event, 10);
                    }
                    None => panic!("No valid port config found, message could not be sent!"),
                };
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
