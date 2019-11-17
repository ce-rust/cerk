use cerk::kernel::{BrokerEvent, CloudEvent};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};

fn write_to_stream<'a>(
    listener: &UnixListener,
    mut stream: UnixStream,
    message: &CloudEvent,
    max_tries: usize,
) -> UnixStream {
    if max_tries == 0 {
        panic!("too many failiers while connection stream");
    }
    if let Err(_) = stream.write_all(message.id.as_bytes()) {
        match listener.accept() {
            Ok((socket, _)) => write_to_stream(listener, socket, message, max_tries - 1),
            Err(err) => panic!(err),
        }
    } else {
        stream
    }
}

pub fn port_output_unix_socket_json_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    _sender_to_kernel: BoxedSender,
) {
    info!("start printer port with id {}", id);
    let listener = UnixListener::bind("./cloud-events").unwrap();
    let mut ok_stream: UnixStream = match listener.accept() {
        Ok((socket, _)) => socket,
        Err(err) => panic!(err),
    };

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(_, _) => info!("{} received ConfigUpdated", id),
            BrokerEvent::OutgoingCloudEvent(cloud_event, _) => {
                ok_stream = write_to_stream(&listener, ok_stream, &cloud_event, 10);
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
