use crate::lapin_helper::{assert_exchange, assert_queue};
use amq_protocol_types::LongString;
use amq_protocol_types::ShortString;
use amq_protocol_types::{AMQPValue, LongLongUInt};
use anyhow::{Context, Result};
use async_std::future::timeout;
use cerk::kernel::{
    BrokerEvent, CloudEventMessageRoutingId, CloudEventRoutingArgs, Config, DeliveryGuarantee,
    IncomingCloudEvent, OutgoingCloudEvent, OutgoingCloudEventProcessed, ProcessingResult,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use cloudevents::{AttributesReader, Event};
use futures_lite::future;
use futures_lite::stream::StreamExt;
use lapin::message::Delivery;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Channel,
    Connection, ConnectionProperties, ExchangeKind,
};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct PendingDelivery {
    consume_channel_id: String,
    delivery_tag: LongLongUInt,
}

type PendingDeliveries = HashMap<CloudEventMessageRoutingId, PendingDelivery>;

struct AmqpConsumeOptions {
    channel: Option<Channel>,
    ensure_queue: bool,
    ensure_dlx: bool,
    bind_to_exchange: Option<String>,
    delivery_guarantee: DeliveryGuarantee,
}

struct AmqpPublishOptions {
    channel: Option<Channel>,
    ensure_exchange: bool,
    delivery_guarantee: DeliveryGuarantee,
}

struct AmqpOptions {
    uri: String,
    consume_channels: HashMap<String, AmqpConsumeOptions>,
    publish_channels: HashMap<String, AmqpPublishOptions>,
}

fn try_get_delivery_option(config: &HashMap<String, Config>) -> Result<DeliveryGuarantee> {
    Ok(match config.get("delivery_guarantee") {
        Some(config) => DeliveryGuarantee::try_from(config)?,
        _ => DeliveryGuarantee::Unspecified,
    })
}

fn build_config(config: &Config) -> Result<AmqpOptions> {
    match config {
        Config::HashMap(config_map) => {
            let mut options = if let Some(Config::String(uri)) = config_map.get("uri") {
                AmqpOptions {
                    uri: uri.to_string(),
                    consume_channels: HashMap::new(),
                    publish_channels: HashMap::new(),
                }
            } else {
                bail!("No uri option")
            };

            if let Some(Config::Vec(ref consumers)) = config_map.get("consume_channels") {
                for consumer_config in consumers.iter() {
                    if let Config::HashMap(consumer) = consumer_config {
                        let ensure_queue = match consumer.get("ensure_queue") {
                            Some(Config::Bool(b)) => *b,
                            _ => false,
                        };
                        let consumer_options = AmqpConsumeOptions {
                            ensure_queue,
                            ensure_dlx: ensure_queue,
                            bind_to_exchange: match consumer.get("bind_to_exchange") {
                                Some(Config::String(s)) => Some(s.to_string()),
                                _ => None,
                            },
                            delivery_guarantee: try_get_delivery_option(consumer)?,
                            channel: None,
                        };

                        if let Some(Config::String(name)) = consumer.get("name") {
                            options
                                .consume_channels
                                .insert(name.to_string(), consumer_options);
                        } else {
                            bail!("consume_channels name is not set")
                        }
                    } else {
                        bail!("consume_channels entries have to be of type HashMap")
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
                            delivery_guarantee: try_get_delivery_option(publisher)?,
                            channel: None,
                        };

                        if let Some(Config::String(name)) = publisher.get("name") {
                            options
                                .publish_channels
                                .insert(name.to_string(), publish_options);
                        } else {
                            bail!("publish_channels name is not set");
                        }
                    } else {
                        bail!("publish_channels entries have to be of type HashMap");
                    }
                }
            }

            Ok(options)
        }
        _ => bail!("{} config has to be of type HashMap"),
    }
}

fn setup_connection(
    id: InternalServerId,
    sender_to_kernel: BoxedSender,
    _connection: &Option<Connection>, // todo reuse connection, if there is already one
    config: Config,
    pending_deliveries: Arc<Mutex<HashMap<String, PendingDelivery>>>,
) -> Result<(Connection, AmqpOptions)> {
    let mut config = match build_config(&config) {
        Ok(c) => c,
        Err(e) => panic!(e),
    };

    async_global_executor::block_on(async {
        let setup =
            setup_connection_async(&id, &sender_to_kernel, &pending_deliveries, &mut config);
        let result = timeout(Duration::from_secs(1), setup)
            .await
            .map_err(|_| anyhow!("setup_connection timed out"))??;
        Ok((result, config))
    })
}

async fn setup_connection_async(
    id: &String,
    sender_to_kernel: &BoxedSender,
    pending_deliveries: &Arc<Mutex<HashMap<String, PendingDelivery>>>,
    config: &mut AmqpOptions,
) -> Result<Connection> {
    let connection = Connection::connect(
        &config.uri,
        ConnectionProperties::default().with_default_executor(8),
    )
    .await?;

    info!("CONNECTED");

    for (name, channel_options) in config.publish_channels.iter_mut() {
        let channel = setup_publish_channel(&connection, &name, channel_options)
            .await
            .with_context(|| format!("failed to setup publish channel {}", &name))?;

        channel_options.channel = Some(channel);
    }

    for (name, channel_options) in config.consume_channels.iter_mut() {
        let channel = setup_consume_channel(
            &connection,
            id,
            sender_to_kernel,
            pending_deliveries,
            &connection,
            name,
            channel_options,
        )
        .await
        .with_context(|| format!("failed to setup consume channel {}", &name))?;
        channel_options.channel = Some(channel);
    }
    Ok(connection)
}

async fn setup_consume_channel(
    connection: &Connection,
    id: &String,
    sender_to_kernel: &BoxedSender,
    pending_deliveries: &Arc<Mutex<HashMap<String, PendingDelivery>>>,
    conn: &Connection,
    name: &String,
    channel_options: &mut AmqpConsumeOptions,
) -> Result<Channel> {
    let mut channel = conn.create_channel().await?;
    if channel_options.delivery_guarantee.requires_acknowledgment() {
        channel
            .confirm_select(ConfirmSelectOptions { nowait: false })
            .await?;
    }
    if channel_options.ensure_queue {
        let mut queue_args = FieldTable::default();
        if channel_options.ensure_dlx {
            let dlx = setup_dlx(conn, &name, &mut channel)
                .await
                .context("failed to setup dlx")?;
            queue_args.insert(
                ShortString::from("x-dead-letter-exchange"),
                AMQPValue::LongString(LongString::from(dlx)),
            );
        }

        let mut queue_options = QueueDeclareOptions::default();
        queue_options.durable = true;
        let queue = assert_queue(
            connection,
            &mut channel,
            name.as_str(),
            queue_options,
            queue_args,
        )
        .await?;
        info!("Declared queue {:?}", queue);

        if let Some(exchange) = &channel_options.bind_to_exchange {
            channel
                .queue_bind(
                    name.as_str(),
                    exchange.as_str(),
                    "",
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
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
    let cloned_delivery_guarantee = channel_options.delivery_guarantee.clone();
    let cloned_name = name.clone();
    let weak_clone = pending_deliveries.clone();
    async_global_executor::spawn(async move {
        info!("will consume");
        while let Some(delivery) = consumer.next().await {
            if let Err(e) = receive_message(
                &cloned_name,
                &cloned_sender,
                &cloned_id,
                weak_clone.clone(),
                &delivery,
                &cloned_delivery_guarantee,
            ) {
                warn!("{} error while receive_message: {:?}", &cloned_id, e)
            }
        }
    })
    .detach();

    Ok(channel)
}

async fn setup_dlx(
    connection: &Connection,
    name: &&String,
    channel: &mut Channel,
) -> Result<String> {
    let dlx_name = format!("{}-dlx", &name);
    let mut exchange_options = ExchangeDeclareOptions::default();
    exchange_options.durable = true;
    let mut queue_options = QueueDeclareOptions::default();
    queue_options.durable = true;

    assert_exchange(
        connection,
        channel,
        dlx_name.as_str(),
        ExchangeKind::Fanout,
        exchange_options,
        FieldTable::default(),
    )
    .await?;
    assert_queue(
        connection,
        channel,
        dlx_name.as_str(),
        queue_options,
        FieldTable::default(),
    )
    .await?;
    channel
        .queue_bind(
            dlx_name.as_str(),
            dlx_name.as_str(),
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;
    Ok(dlx_name)
}

async fn setup_publish_channel(
    conn: &Connection,
    name: &&String,
    channel_options: &mut AmqpPublishOptions,
) -> Result<Channel> {
    let mut channel = conn.create_channel().await?;
    if channel_options.delivery_guarantee.requires_acknowledgment() {
        channel
            .confirm_select(ConfirmSelectOptions { nowait: false })
            .await?;
    }
    if channel_options.ensure_exchange {
        assert_exchange(
            conn,
            &mut channel,
            name.as_str(),
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
        info!("Declared exchange {}", &name);
    }
    Ok(channel)
}

fn receive_message(
    name: &String,
    sender: &BoxedSender,
    id: &String,
    pending_deliveries: Arc<Mutex<HashMap<String, PendingDelivery>>>,
    delivery: &lapin::Result<(Channel, Delivery)>,
    delivery_guarantee: &DeliveryGuarantee,
) -> Result<()> {
    let (channel, delivery) = delivery.as_ref().expect("error in consumer");
    debug!("{} received CloudEvent on queue {}", id, channel.id());
    let payload_str = std::str::from_utf8(&delivery.data).unwrap();
    match serde_json::from_str::<Event>(&payload_str) {
        Ok(cloud_event) => {
            debug!("{} deserialized event successfully", id);
            let routing_id = get_event_id(&cloud_event, &delivery.delivery_tag);
            info!(
                "pending_deliveries size: {}",
                pending_deliveries.clone().lock().unwrap().len()
            );
            if pending_deliveries
                .clone()
                .lock()
                .unwrap()
                .insert(
                    routing_id.to_string(),
                    PendingDelivery {
                        delivery_tag: delivery.delivery_tag.clone(),
                        consume_channel_id: name.to_string(),
                    },
                )
                .is_some()
            {
                error!(
                    "failed event_id={} was already in the table - this should not happen",
                    &routing_id
                );
            }
            sender.send(BrokerEvent::IncomingCloudEvent(IncomingCloudEvent {
                incoming_id: id.clone(),
                routing_id,
                cloud_event,
                args: CloudEventRoutingArgs {
                    delivery_guarantee: delivery_guarantee.clone(),
                },
            }));
        }
        Err(err) => {
            bail!("{} while converting string to CloudEvent: {:?}", id, err);
        }
    }

    Ok(())
}

fn get_event_id(cloud_event: &Event, delivery_tag: &LongLongUInt) -> String {
    format!("{}--{}", cloud_event.id(), delivery_tag)
}

async fn send_cloud_event(cloud_event: &Event, configurations: &AmqpOptions) -> Result<()> {
    let payload = serde_json::to_string(cloud_event).unwrap();
    for (name, options) in configurations.publish_channels.iter() {
        let result = match options.channel {
            Some(ref channel) => {
                let result = publish_cloud_event(&payload, &name, channel).await;
                if let Ok(result) = result {
                    if !options.delivery_guarantee.requires_acknowledgment() || result.is_ack() {
                        Ok(())
                    } else {
                        bail!("Message was not acknowledged, but channel delivery_guarantee requires it: {:?}", result)
                    }
                } else {
                    Err(anyhow!("message was not sent successful"))
                }
            }
            None => Err(anyhow!("channel to exchange is closed")),
        };
        result?
    }
    Ok(())
}

async fn publish_cloud_event(
    payload: &String,
    name: &String,
    channel: &Channel,
) -> Result<Confirmation> {
    let confirmation = channel
        .basic_publish(
            name.as_str(),
            "",
            BasicPublishOptions {
                mandatory: true,
                immediate: false,
            },
            Vec::from(payload.as_str()),
            BasicProperties::default()
                .with_delivery_mode(2) //persistent
                .with_content_type(ShortString::from(
                    "application/cloudevents+json; charset=UTF-8",
                )),
        )
        .await?
        .await?;
    Ok(confirmation)
}

async fn ack_nack_pending_event(
    configuration_option: &Option<AmqpOptions>,
    pending_deliveries: &mut HashMap<String, PendingDelivery>,
    event_id: &String,
    result: ProcessingResult,
) -> Result<()> {
    let pending_event = pending_deliveries
        .get(event_id)
        .with_context(|| format!("pending delivery with id={} not found", event_id))?;
    let configuration_option = configuration_option
        .as_ref()
        .and_then(|o| Some(Ok(o)))
        .unwrap_or(Err(anyhow!("configuration_option is not set")))?;
    let channel_options = configuration_option
        .consume_channels
        .get(&pending_event.consume_channel_id)
        .context("channel not found to ack/nack pending delivery")?;
    let channel = channel_options
        .channel
        .as_ref()
        .context("channel not open")?;
    match result {
        ProcessingResult::Successful => {
            channel
                .basic_ack(pending_event.delivery_tag, BasicAckOptions::default())
                .await?
        }
        ProcessingResult::TransientError => {
            channel
                .basic_nack(
                    pending_event.delivery_tag,
                    BasicNackOptions {
                        multiple: false,
                        requeue: true,
                    },
                )
                .await?
        }
        ProcessingResult::PermanentError => {
            channel
                .basic_nack(
                    pending_event.delivery_tag,
                    BasicNackOptions {
                        multiple: false,
                        requeue: false,
                    },
                )
                .await?
        }
    };
    Ok(())
}

/// This is the main function to start the port.
pub fn port_amqp_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    let mut connection_option: Option<Connection> = None;
    let mut configuration_option: Option<AmqpOptions> = None;
    let pending_deliveries: PendingDeliveries = HashMap::new();
    let arc_pending_deliveries: Arc<Mutex<HashMap<String, PendingDelivery>>> =
        Arc::new(Mutex::new(pending_deliveries));

    info!("start amqp port with id {}", id);

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                info!("{} initiated", id);
            }
            BrokerEvent::ConfigUpdated(config, _) => {
                info!("{} received ConfigUpdated", &id);
                let result = setup_connection(
                    id.clone(),
                    sender_to_kernel.clone_boxed(),
                    &connection_option,
                    config,
                    arc_pending_deliveries.clone(),
                );
                match result {
                    Ok(as_ok) => {
                        connection_option = Some(as_ok.0);
                        configuration_option = Some(as_ok.1);
                    }
                    Err(e) => {
                        warn!("{} was not able to establish a connection: {:?}", &id, e);
                        connection_option = None;
                        configuration_option = None;
                    }
                }
            }
            BrokerEvent::OutgoingCloudEvent(event) => {
                let OutgoingCloudEvent {
                    routing_id,
                    cloud_event,
                    destination_id: _,
                    args,
                } = event;
                debug!("{} CloudEvent received", &id);
                if let Some(configuration) = configuration_option.as_ref() {
                    let result = future::block_on(send_cloud_event(&cloud_event, configuration));
                    let result = match result {
                        Ok(_) => {
                            info!("sent cloud event to queue");
                            ProcessingResult::Successful
                        }
                        Err(e) => {
                            error!("{} was not able to send CloudEvent {}", &id, e);
                            // todo transient or permanent?
                            ProcessingResult::PermanentError
                        }
                    };
                    if args.delivery_guarantee.requires_acknowledgment() {
                        sender_to_kernel.send(BrokerEvent::OutgoingCloudEventProcessed(
                            OutgoingCloudEventProcessed {
                                sender_id: id.clone(),
                                routing_id,
                                result,
                            },
                        ));
                    }
                } else {
                    error!("received CloudEvent before connection was  set up - message will not be delivered")
                }
            }
            BrokerEvent::IncomingCloudEventProcessed(event_id, result) => {
                let result = future::block_on(ack_nack_pending_event(
                    &configuration_option,
                    arc_pending_deliveries.lock().unwrap().borrow_mut(),
                    &event_id,
                    result,
                ));
                match result {
                    Ok(()) => debug!("IncomingCloudEventProcessed was ack/nack successful"),
                    Err(err) => warn!("IncomingCloudEventProcessed was not ack/nack {:?}", err),
                };
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the port.
pub static PORT_AMQP: InternalServerFnRefStatic = &(port_amqp_start as InternalServerFn);
