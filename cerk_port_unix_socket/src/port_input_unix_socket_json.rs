use cerk::kernel::{BrokerEvent, Config};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::{CloudEvent, Data};
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};

fn liten_to_stream(
    id: &InternalServerId,
    listener: &UnixListener,
    stream: Option<BufReader<UnixStream>>,
    sender_to_kernel: &BoxedSender,
    max_tries: usize,
) -> Option<BufReader<UnixStream>> {
    if max_tries == 0 {
        panic!("too many failures while trying to connect to stream");
    }
    debug!("listen to stream...");
    match stream {
        None => match listener.accept() {
            Ok((socket, _)) => {
                let stream = BufReader::new(socket);
                liten_to_stream(id, listener, Some(stream), sender_to_kernel, max_tries - 1)
            }
            Err(err) => panic!(err),
        },
        Some(stream) => {
            for line in stream.lines() {
                let cloud_event = CloudEvent {
                    id: String::from("1"),
                    event_type: String::from("socket-in"),
                    spec_version: String::from("1.0"),
                    source: id.clone(),
                    time: None,
                    subject: None,
                    data_schema: None,
                    data_content_type: Some(String::from("text/plain")),
                    data: Data::String(line.unwrap()),
                };
                info!("{:?}", cloud_event);
                sender_to_kernel.send(BrokerEvent::IncommingCloudEvent(id.clone(), cloud_event))
            }
            None // todo
        }
    }
}

pub fn port_input_unix_socket_json_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start input JSON over unix socket port with id {}", id);
    let mut listener: Option<UnixListener> = None;
    let mut stream: Option<BufReader<UnixStream>> = None;

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
            broker_event => warn!("event {} not implemented", broker_event),
        }

        if let Some(listener) = listener.as_ref() {
            stream = liten_to_stream(&id, listener, stream, &sender_to_kernel, 10);
        }
    }
}
