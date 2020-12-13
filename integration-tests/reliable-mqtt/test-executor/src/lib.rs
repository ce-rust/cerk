#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

extern crate ctor;

use anyhow::{Context, Result};
use async_std;
use async_std::future::timeout;
use async_std::prelude::*;
use async_std::stream;
use async_std::stream::Stream;
use async_std::stream::StreamExt;
use chrono::{DateTime, Utc};
use cloudevents::{AttributesReader, Data, Event, EventBuilder, EventBuilderV10};
use env_logger::Env;
use futures::try_join;
use paho_mqtt as mqtt;
use serde_json;
use std::collections::HashSet;
use std::env;
use std::time::Duration;

fn get_event_id(payload: &[u8]) -> Result<String> {
    let event: Event = serde_json::from_slice(payload)?;
    Ok(event.id().to_string())
}

async fn setup_connection(client: mqtt::AsyncClient) -> Result<()> {
    let connection_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        .clean_session(false)
        .finalize();

    client.connect(connection_opts).await?;
    Ok(())
}

async fn observe_topic(
    mut client: mqtt::AsyncClient,
    topic_name: &str,
    expected_event_count: usize,
) -> Result<()> {
    let mut stream = client.get_stream(10 * expected_event_count);
    let mut received_message_ids = HashSet::new();

    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            if msg.topic() == topic_name {
                let message_id = get_event_id(msg.payload())?;
                debug!("received message on {}: {}", message_id, topic_name);
                received_message_ids.insert(message_id);
                let event_count = received_message_ids.len();
                info!("{}: {}/{}", topic_name, event_count, expected_event_count);
                if event_count == expected_event_count {
                    info!("all expected messages received");
                    break;
                }
            }
        } else {
            break;
        }
    }

    return Ok(());
}

async fn observe_stored_messages(
    mut client: mqtt::AsyncClient,
    expected_stored_unretained: usize,
) -> Result<()> {
    let mut stream = client.get_stream(10 * expected_stored_unretained);
    let mut retained: usize = 0;
    let mut stored: usize = 0;
    info!("observe stored messages");

    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            let topic_name = msg.topic();
            info!("received message on {}", topic_name);
            info!(
                "match retained {}",
                topic_name == "$SYS/broker/retained messages/count"
            );
            if topic_name == "$SYS/broker/retained messages/count" {
                retained = msg.payload_str().parse()?;
            } else {
                stored = msg.payload_str().parse()?;
            }
            info!("stored={}", stored);
            info!("retained={}", retained);
            if stored >= retained {
                let stored_unretained = stored - retained;
                info!(
                    "stored-retained messages: {}/{}",
                    stored_unretained, expected_stored_unretained
                );
                if stored_unretained == expected_stored_unretained {
                    break;
                }
            }
        } else {
            bail!("no message received");
        }
    }
    info!("exited while loop");

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        env_logger::from_env(Env::default().default_filter_or("info")).init();
    }

    #[async_std::test]
    async fn test_successful_routing() -> Result<()> {
        let expected_event_count = env::var("SAMPLE_SIZE")
            .unwrap_or(100.to_string())
            .parse::<usize>()
            .unwrap();

        let inbox_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri("tcp://unlimited:1883")
            .client_id("inbox-observer")
            .finalize();
        let inbox_client = mqtt::AsyncClient::new(inbox_opts)?;

        let outbox_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri("tcp://unlimited:1883")
            .client_id("outbox-observer")
            .finalize();
        let outbox_client = mqtt::AsyncClient::new(outbox_opts)?;

        let stored_messages_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri("tcp://unlimited:1883")
            .client_id("storage-observer")
            .finalize();
        let stored_messages_client = mqtt::AsyncClient::new(stored_messages_opts)?;

        setup_connection(inbox_client.clone()).await?;
        setup_connection(outbox_client.clone()).await?;
        setup_connection(stored_messages_client.clone()).await?;

        inbox_client.subscribe_many(&["inbox"], &[1]).await?;
        outbox_client.subscribe_many(&["outbox"], &[1]).await?;
        stored_messages_client
            .subscribe_many(
                &[
                    "$SYS/broker/retained messages/count",
                    "$SYS/broker/store/messages/count",
                ],
                &[1, 1],
            )
            .await?;

        let inbox_observer = async_std::task::spawn(observe_topic(
            inbox_client.clone(),
            "inbox",
            expected_event_count,
        ));
        let outbox_observer =
            async_std::task::spawn(observe_topic(outbox_client.clone(), "outbox", 1));
        let stored_messages_observer = async_std::task::spawn(observe_stored_messages(
            stored_messages_client.clone(),
            expected_event_count,
        ));

        for i in 0..expected_event_count {
            let mut data: Vec<u8> = Vec::new();
            data.resize(5000, 0);
            let event = EventBuilderV10::new()
                .id(format!("{}", i))
                .ty("my_event.my_application")
                .source("http://localhost:8080")
                .time(Utc::now())
                .data("binary", Data::Binary(data))
                .build()?;
            let serialized = serde_json::to_string(&event).unwrap();
            let mesage = mqtt::MessageBuilder::new()
                .topic("inbox")
                .payload(serialized)
                .qos(mqtt::QOS_1)
                .finalize();
            if inbox_client.is_connected() {
                inbox_client.publish(mesage).await?;
            } else {
                bail!("connection error")
            }
        }

        timeout(Duration::from_secs(20), inbox_observer).await.context("Failed to publish all expected messages")??;
        timeout(Duration::from_secs(20), outbox_observer).await.context("Failed to publish 1 message to outbox")??;
        timeout(Duration::from_secs(20), stored_messages_observer).await.context("Failed build backpressure")??;

        info!("test done");

        return Ok(());
    }
}
