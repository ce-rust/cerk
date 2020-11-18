use cerk::kernel::{BrokerEvent, Config};
use cerk::runtime::InternalServerId;
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use futures_lite::stream::StreamExt;
use lapin::{options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection, ConnectionProperties, Result, Channel, ExchangeKind};
use std::collections::HashMap;
use cloudevents::CloudEvent;
use std::result::Result as stdresult;
use futures_lite::future;

struct AmqpConsumeOptions {
    ensure_queue: bool,
    bind_to_exchange: Option<String>,
}

struct AmqpPublishOptions {
    channel: Option<Channel>,
    ensure_exchange: bool,
}

struct AmqpOptions {
    uri: String,
    consume_channels: HashMap<String, AmqpConsumeOptions>,
    publish_channels: HashMap<String, AmqpPublishOptions>,
}

fn build_config(id: &InternalServerId, config: &Config) -> stdresult<AmqpOptions, &'static str> {
    match config {
        Config::HashMap(config_map) => {
            let mut options = if let Some(Config::String(uri)) = config_map.get("uri") {
                AmqpOptions {
                    uri: uri.to_string(),
                    consume_channels: HashMap::new(),
                    publish_channels: HashMap::new(),
                }
            } else {
                return Err("No uri option");
            };

            if let Some(Config::Vec(ref consumers)) = config_map.get("consume_channels") {
                for consumer_config in consumers.iter() {
                    if let Config::HashMap(consumer) = consumer_config {
                        let consumer_options = AmqpConsumeOptions {
                            ensure_queue: match consumer.get("ensure_queue") {
                                Some(Config::Bool(b)) => *b,
                                _ => false,
                            },
                            bind_to_exchange: match consumer.get("bind_to_exchange") {
                                Some(Config::String(s)) => Some(s.to_string()),
                                _ => None,
                            },
                        };

                        if let Some(Config::String(name)) = consumer.get("name") {
                            options.consume_channels.insert(name.to_string(), consumer_options);
                        } else {
                            return Err("consume_channels name is not set");
                        }
                    } else {
                        return Err("consume_channels entries have to be of type HashMap");
                    }
                }
            }

            if let Some(Config::Vec(ref publishers)) = config_map.get("publish_channels") {
                for publisher_config in publishers.iter() {
                    if let Config::HashMap(publisher) = publisher_config {
                        let publish_options = AmqpPublishOptions {
                            ensure_exchange: match publisher.get("ensure_exchange") {
                                Some(Config::Bool(b)) => *b,
                                _ => false,
                            },
                            channel: None,
                        };

                        if let Some(Config::String(name)) = publisher.get("name") {
                            options.publish_channels.insert(name.to_string(), publish_options);
                        } else {
                            return Err("publish_channels name is not set");
                        }
                    } else {
                        return Err("publish_channels entries have to be of type HashMap");
                    }
                }
            }

            Ok(options)
        }
        _ => Err("{} config has to be of type HashMap"),
    }
}

fn setup_connection(id: InternalServerId, sender_to_kernel: BoxedSender, connection: &Option<Connection>, config: Config) -> Result<(Connection, AmqpOptions)> {
    let mut config = match build_config(&id.clone(), &config) {
        Ok(c) => c,
        Err(e) => panic!(e),
    };

    async_global_executor::block_on(async {
        let conn = Connection::connect(
            &config.uri,
            ConnectionProperties::default().with_default_executor(8),
        )
            .await?;

        info!("CONNECTED");

        for (name, channel_options) in config.publish_channels.iter_mut() {
            let channel = conn.create_channel().await?;
            if channel_options.ensure_exchange {
                let exchange = channel.exchange_declare(
                    name.as_str(),
                    ExchangeKind::Fanout,
                    ExchangeDeclareOptions::default(),
                    FieldTable::default(),
                );
                info!("Declared exchange {:?}", exchange);
            }

            channel_options.channel = Some(channel);
        }

        for (name, channel_options) in config.consume_channels.iter() {
            let channel = conn.create_channel().await?;
            if channel_options.ensure_queue {
                let queue = channel
                    .queue_declare(
                        name.as_str(),
                        QueueDeclareOptions::default(),
                        FieldTable::default(),
                    )
                    .await?;
                info!("Declared queue {:?}", queue);

                if let Some(exchange) = &channel_options.bind_to_exchange {
                    channel.queue_bind(name.as_str(),
                                       exchange.as_str(),
                                       "",
                                       QueueBindOptions::default(),
                                       FieldTable::default())
                        .await?;
                }
            }

            let mut consumer = channel
                .basic_consume(
                    name.as_str(),
                    format!("cerk-{}", id.clone()).as_str(),
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await?;

            let cloned_sender = sender_to_kernel.clone_boxed();
            let cloned_id = id.clone();
            async_global_executor::spawn(async move {
                info!("will consume");
                while let Some(delivery) = consumer.next().await {
                    let (channel, delivery) = delivery.expect("error in consumer");
                    debug!("{} received CloudEvent on queue {}", cloned_id, channel.id());
                    let payload_str = std::str::from_utf8(&delivery.data).unwrap();
                    match serde_json::from_str::<CloudEvent>(&payload_str) {
                        Ok(cloud_event) => {
                            debug!("{} deserialized event successfully", cloned_id);
                            cloned_sender.send(BrokerEvent::IncommingCloudEvent(
                                cloned_id.clone(),
                                cloud_event,
                            ));
                        }
                        Err(err) => {
                            error!("{} while converting string to CloudEvent: {:?}", cloned_id, err);
                        }
                    };
                    channel
                        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                        .await
                        .expect("ack");
                }
            }).detach();
        }

        Ok((conn, config))
    })
}

async fn send_cloud_event(id: &InternalServerId, cloud_event: &CloudEvent, configurations: &AmqpOptions) -> stdresult<(), &'static str> {
    let payload = serde_json::to_string(cloud_event).unwrap();
    for (name, options) in configurations.publish_channels.iter() {
        let result = match options.channel {
            Some(ref channel) => {
                let result = publish_cloud_event(&payload, &name, channel)
                    .await;
                if let Ok(_) = result {
                    // todo shoud we check for acks?  ok_result.is_ack()
                    Ok(())
                } else {
                    Err("message was not sent successful")
                }
            }
            None => Err("channel to exchange is closed"),
        };
        if result.is_err() {
            return result;
        }
    }
    Ok(())
}

async fn publish_cloud_event(payload: &String, name: &String, channel: &Channel) -> Result<Confirmation> {
    let confirmation = channel.basic_publish(name.as_str(),
                                             "",
                                             BasicPublishOptions { mandatory: true, immediate: false },
                                             Vec::from(payload.as_str()),
                                             BasicProperties::default().with_delivery_mode(2))//persistent
        .await?
        .await?;
    Ok(confirmation)
}

/// This port publishes and/or subscribe CloudEvents to/from an AMQP broker with protocol version v0.9.1.
///
/// The port is implemented with [lapin](https://github.com/CleverCloud/lapin).
///
/// # Examples
///
/// * [Sequence to AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/src/sequence_to_amqp_to_printer/)
/// * [AMQP to Printer](https://github.com/ce-rust/cerk/tree/master/examples/src/amqp_to_printer/)
///
pub fn port_amqp_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    let mut connection_option: Option<Connection> = None;
    let mut configuration_option: Option<AmqpOptions> = None;

    info!("start amqp port with id {}", id);

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                let result = setup_connection(id.clone(), sender_to_kernel.clone_boxed(), &connection_option, config);
                if result.is_err() {
                    warn!("{} was not able to establish a connection", &id);
                }
                if let Ok(as_ok) = result {
                    connection_option = Some(as_ok.0);
                    configuration_option = Some(as_ok.1);
                } else {
                    connection_option = None;
                    configuration_option = None;
                }
            }
            BrokerEvent::OutgoingCloudEvent(cloud_event, _) => {
                debug!("{} CloudEvent received", &id);
                if let Some(configuration) = configuration_option.as_ref() {
                    let result = future::block_on(send_cloud_event(&id, &cloud_event, configuration));
                    if result.is_err() {
                        error!("{} was not able to send CloudEvent", &id);
                    }
                } else {
                    error!("received CloudEvent before connection was  set up - message will not be delivered")
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
