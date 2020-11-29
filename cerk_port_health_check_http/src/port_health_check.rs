use anyhow::Result;
use cerk::kernel::{
    BrokerEvent, Config, ConfigHelpers, HealthCheckRequest, HealthCheckResponse, HealthCheckStatus,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server, StatusCode};
use serde::Serialize;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::sync::oneshot::Sender;
use tokio::time::timeout;
use uuid::Uuid;

type ArcHealthCheckData = Arc<Mutex<HealthCheckData>>;

struct PendingRequest {
    receiver: Option<Sender<()>>,
    responses: HashMap<InternalServerId, Option<HealthCheckStatus>>,
}

struct HealthCheckConfig {
    http_port: u16,
    ports_to_check: Vec<InternalServerId>,
    timeout: Duration,
}

struct HealthCheckData {
    config: Option<HealthCheckConfig>,
    shotdown: Option<Sender<()>>,
    tokio: Handle,
    sender_to_kernel: BoxedSender,
    id: InternalServerId,
    pending_requests: HashMap<String, PendingRequest>,
}

#[derive(Serialize, Debug, PartialEq)]
struct HealthHttpResponse {
    message: String,
    requests: HashMap<InternalServerId, Option<HealthCheckStatus>>,
}

fn build_config(id: &InternalServerId, config: Config) -> Result<HealthCheckConfig> {
    let ports: Vec<Result<String>> = config
        .get_op_val_vec("ports_to_check")?
        .unwrap_or(vec![])
        .iter()
        .map(|c| String::try_from(c))
        .collect();
    if ports.iter().any(|r| r.is_err()) {
        bail!("{} ports_to_check have to be an array of strings", id)
    }
    let port_config = HealthCheckConfig {
        http_port: config.get_op_val_u32("http_port")?.unwrap_or(3000) as u16,
        ports_to_check: ports
            .iter()
            .filter_map(|c| c.as_ref().ok().map(|v| v.to_string()))
            .collect(),
        timeout: Duration::from_millis(config.get_op_val_u8("timeout")?.unwrap_or(10 as u8) as u64),
    };
    Ok(port_config)
}

fn update(config: Config, data: ArcHealthCheckData) -> Result<()> {
    if let Ok(data_config) = data.as_ref().lock().as_mut() {
        data_config.borrow_mut().config = Some(build_config(&data_config.id.clone(), config)?);
    } else {
        bail!("failed to write config")
    }

    start_server(data)?;
    Ok(())
}

fn start_server(data: ArcHealthCheckData) -> Result<()> {
    if let Ok(mut data_config) = data.clone().lock() {
        if let Some(tx) = data_config.borrow_mut().shotdown.take() {
            let r = tx.send(());
            if let Err(e) = r {
                error!("failed to shutdown: {:?}", e);
            }
        }
    } else {
        bail!("failed to write config")
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    data.lock().unwrap().borrow_mut().shotdown = Some(tx);

    let tokio = data.clone().lock().unwrap().tokio.clone();

    let port = data
        .clone()
        .lock()
        .unwrap()
        .config
        .as_ref()
        .unwrap()
        .http_port;

    let make_svc = make_service_fn(move |_| {
        let data = data.clone();
        async move {
            Ok::<_, Error>(service_fn(move |_req| {
                let data = data.clone();
                async move { handle_health_request(data).await }
            }))
        }
    });

    tokio.spawn(async move {
        let server = Server::bind(&([127, 0, 0, 1], port).into()).serve(make_svc);
        let graceful = server.with_graceful_shutdown(async {
            rx.await.ok();
        });
        if let Err(e) = graceful.await {
            eprintln!("server error: {}", e);
        }
    });

    Ok(())
}

async fn handle_health_request(data: Arc<Mutex<HealthCheckData>>) -> Result<Response<Body>, Error> {
    let uuid = Uuid::new_v4();
    let sender_id = data.clone().lock().unwrap().id.clone();
    let sender = data.lock().unwrap().sender_to_kernel.clone_boxed();
    let timeout_duration = data
        .lock()
        .unwrap()
        .config
        .as_ref()
        .unwrap()
        .timeout
        .clone();
    let ports = data
        .clone()
        .lock()
        .unwrap()
        .config
        .as_ref()
        .unwrap()
        .ports_to_check
        .clone();

    if ports.len() > 0 {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let pending_request = PendingRequest {
            receiver: Some(tx),
            responses: ports.iter().map(|id| (id.clone(), None)).collect(),
        };
        data.lock()
            .unwrap()
            .pending_requests
            .insert(uuid.to_string(), pending_request);

        for port in ports {
            sender.send(BrokerEvent::HealthCheckRequest(HealthCheckRequest {
                sender_id: sender_id.clone(),
                destination_id: port.clone(),
                id: uuid.to_string(),
            }));
        }

        let result = timeout(timeout_duration, rx).await;
        let responses = data
            .lock()
            .unwrap()
            .pending_requests
            .remove(uuid.to_string().as_str())
            .unwrap();
        let mut status_code = StatusCode::SERVICE_UNAVAILABLE;
        let body = HealthHttpResponse {
            message: if let Err(e) = result {
                warn!(
                    "did not receive a value in {}ms: {:?}",
                    timeout_duration.as_millis(),
                    e
                );
                "Timeout, ports did not respond; current result: {}".to_string()
            } else {
                if responses
                    .responses
                    .iter()
                    .any(|(_, status)| status.eq(&Some(HealthCheckStatus::Healthy)))
                {
                    "not all responses were successful".to_string()
                } else {
                    status_code = StatusCode::OK;
                    "successful".to_string()
                }
            },
            requests: responses.responses.clone(),
        };

        let body = serde_json::to_vec(&body).unwrap();

        let response = Response::builder()
            .status(status_code)
            .body(Body::from(body))
            .unwrap();
        Ok::<_, Error>(response)
    } else {
        Ok::<_, Error>(Response::new(Body::from("OK")))
    }
}

fn received_health_check_from_port(
    event: HealthCheckResponse,
    data: ArcHealthCheckData,
) -> Result<()> {
    if let Ok(mut data) = data.lock() {
        if let Some(request) = data.pending_requests.get_mut(event.id.as_str()) {
            if let Some(port) = request.responses.get_mut(event.sender_id.as_str()) {
                *port = Some(event.status);

                if request.responses.iter().all(|(_, status)| status.is_some()) {
                    if let Some(send) = request.receiver.take() {
                        if send.send(()).is_err() {
                            bail!("failed to notify web server")
                        }
                        return Ok(());
                    } else {
                        bail!("receiver was not set - could not notify web server")
                    }
                }
            }
        }
    }
    bail!("failed to received_health_check_from_port")
}

/// This is the main function to start the port.
pub fn port_health_check_http(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start http health check port with id {}", id);
    let tokio = tokio::runtime::Runtime::new().unwrap();
    let data = HealthCheckData {
        tokio: tokio.handle().clone(),
        config: None,
        shotdown: None,
        sender_to_kernel,
        id,
        pending_requests: HashMap::new(),
    };
    let data: ArcHealthCheckData = Arc::new(Mutex::new(data));

    loop {
        match inbox.receive() {
            BrokerEvent::ConfigUpdated(config, _) => {
                if let Err(e) = update(config, data.clone()) {
                    error!("failed to build config {:?}", e)
                }
            }
            BrokerEvent::HealthCheckResponse(event) => {
                if let Err(e) = received_health_check_from_port(event, data.clone()) {
                    error!("failed to register HealthCheckResponse {:?}", e)
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the port.
pub static PORT_HEALTH_CHECK_HTTP: InternalServerFnRefStatic =
    &(port_health_check_http as InternalServerFn);

#[cfg(test)]
mod tests {
    use super::*;
    use cerk_runtime_threading::channel::new_channel_with_size;
    use env_logger::Env;
    use hyper::Client;
    use tokio::time::delay_for;

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    }

    #[test]
    fn test_server_no_ports() -> Result<()> {
        test_server(vec![], 200, 3001)
    }

    #[test]
    fn test_start_server_non_existing_port() -> Result<()> {
        test_server(vec!["non-existing".to_string()], 503, 3002)
    }

    fn test_server(ports: Vec<InternalServerId>, status: u16, server_port: u16) -> Result<()> {
        let (send, _receive) = new_channel_with_size(1);
        let tokio = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();
        let mut config = HealthCheckData {
            shotdown: None,
            config: None,
            tokio: tokio.handle().clone(),
            sender_to_kernel: send,
            id: "the-id".to_string(),
            pending_requests: HashMap::new(),
        };
        config.config = Some(HealthCheckConfig {
            http_port: server_port,
            timeout: Duration::from_millis(10),
            ports_to_check: ports,
        });
        let data: ArcHealthCheckData = Arc::new(Mutex::new(config));
        let e = start_server(data.clone());
        assert!(e.is_ok());

        let handle = data.clone().lock().unwrap().tokio.clone();
        handle.block_on(async {
            delay_for(Duration::from_millis(10)).await;
            let client = Client::new();
            let r = client
                .get(format!("http://localhost:{}", server_port).parse().unwrap())
                .await;
            match r {
                Ok(r) => assert_eq!(r.status(), status),
                Err(e) => assert!(false, e),
            }
        });

        assert!(data.lock().unwrap().shotdown.is_some());
        let e = data.lock().unwrap().shotdown.take().unwrap().send(());
        assert!(e.is_ok());
        Ok(())
    }
}
