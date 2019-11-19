use cerk::kernel::{BrokerEvent, CloudEvent, Config};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};

fn write_to_stream(
    listener: &UnixListener,
    stream: Option<UnixStream>,
    message: &CloudEvent,
    max_tries: usize,
) -> Option<UnixStream> {
    if max_tries == 0 {
        panic!("too many failures while trying to connect to stream");
    }
    match stream {
        None => match listener.accept() {
            Ok((socket, _)) => write_to_stream(listener, Some(socket), message, max_tries - 1),
            Err(err) => panic!(err),
        },
        Some(mut stream) => {
            if let Err(_) = stream.write_all(message.id.as_bytes()) {
                write_to_stream(listener, None, message, max_tries - 1)
            } else {
                Some(stream)
            }
        }
    }
}

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
            BrokerEvent::OutgoingCloudEvent(cloud_event, _) => {
                match listener.as_ref() {
                    Some(listener) => {
                        stream = write_to_stream(listener, stream, &cloud_event, 10);
                    }
                    None => panic!("No valid port config found, message could not be sent!"),
                };
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
