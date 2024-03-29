#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

use anyhow::{Context, Error, Result};
use cerk_port_amqp::lapin_helper::assert_queue;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties,
    Result as LapinResult,
};
use std::env;
use std::{thread, time};

const ROUTER_OUTPUT_QUEUE: &'static str = "test_queue_router_output";
const ROUTER_INPUT_QUEUE: &'static str = "router_input";
const ROUTER_INPUT_DLX_QUEUE: &'static str = "router_input-dlx";
const ROUTER_OUTPUT_EXCHANGE: &'static str = "router_output";

async fn connect() -> LapinResult<Connection> {
    let amqp_broker_uri: String =
        env::var("AMQP_BROKER_URL").unwrap_or(String::from("amqp://127.0.0.1:5672/%2f"));

    let conn = Connection::connect(
        amqp_broker_uri.as_str(),
        ConnectionProperties::default().with_default_executor(8),
    )
    .await?;

    info!("connected");
    Ok(conn)
}

async fn has_message_on_output(channel: &Channel, queue: &'static str) -> Result<u32> {
    // todo does not work as intended, result.unwrap().message_count is always 0
    let result = channel
        .basic_get(queue, BasicGetOptions { no_ack: false })
        .await?;

    if result.is_some() {
        Ok(1)
    } else {
        Ok(0)
    }
}

async fn create_test_and_bind_queue(
    connection: &Connection,
    test_queue_options: &FieldTable,
) -> Result<()> {
    let mut channel = open_channel(connection).await?;

    assert_queue(
        connection,
        &mut channel,
        ROUTER_OUTPUT_QUEUE,
        QueueDeclareOptions {
            nowait: false,
            auto_delete: false,
            durable: true,
            exclusive: false,
            passive: false,
        },
        test_queue_options.clone(),
    )
    .await
    .with_context(|| format!("was not able to create {} queue", ROUTER_OUTPUT_QUEUE))?;
    info!("test queue created");

    channel
        .queue_bind(
            ROUTER_OUTPUT_QUEUE,
            ROUTER_OUTPUT_EXCHANGE,
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .context("was not able to bind queue")?;
    info!("test queue bounded");

    Ok(())
}

async fn open_channel(connection: &Connection) -> Result<Channel> {
    Ok(connection
        .create_channel()
        .await
        .context("create_channel error")?)
}

async fn set_up(test_queue_options: &FieldTable) -> Result<Channel> {
    let connection = connect().await.context("amqp connect error")?;

    create_test_and_bind_queue(&connection, test_queue_options).await?;

    let channel = open_channel(&connection).await?;

    for queue in vec![
        ROUTER_INPUT_QUEUE,
        ROUTER_INPUT_DLX_QUEUE,
        ROUTER_OUTPUT_QUEUE,
    ] {
        channel
            .queue_purge(queue, QueuePurgeOptions::default())
            .await?;
        let count_before = has_message_on_output(&channel, queue).await?;
        assert_eq!(
            count_before, 0,
            "should not have any message after prune {}",
            queue
        );
    }

    Ok(channel)
}

struct AssertQueues {
    queue_with_massage: &'static str,
    queues_without_massage: Vec<&'static str>,
}

async fn try_wait_for_message(channel: &Channel, assert: AssertQueues) -> Result<(), Error> {
    let mut result: Result<()> = Err(anyhow!(
        "message was not received on queue {}",
        assert.queue_with_massage
    ));
    let ten_millis = time::Duration::from_millis(100);
    for _ in 0..100 {
        thread::sleep(ten_millis);
        let count = has_message_on_output(&channel, assert.queue_with_massage).await?;
        if count == 1 {
            result = Ok::<(), Error>(()); // https://github.com/rust-lang/rust/issues/63502
            for count in assert.queues_without_massage {
                let second_count = has_message_on_output(&channel, count).await?;
                assert_eq!(
                    second_count, 0,
                    "there should be no message on queue {} but it has count {}",
                    count, second_count
                );
            }
            break;
        } else if count > 1 {
            assert_eq!(
                count, 1,
                "received more messages then sent; count {} on queue {}",
                count, assert.queue_with_massage
            );
        }
    }

    result
}

#[allow(dead_code)]
fn execute(test_queue_options: &FieldTable, assert_queue: AssertQueues) -> Result<()> {
    async_global_executor::block_on(async {
        let channel = set_up(test_queue_options).await?;

        let payload = r#"{"type":"test type","specversion":"1.0","source":"http://www.google.com","id":"id","contenttype":"application/json","data":"test"}"#;
        channel
            .basic_publish(
                "", //default exchange
                ROUTER_INPUT_QUEUE,
                BasicPublishOptions {
                    mandatory: true,
                    immediate: false,
                },
                Vec::from(payload),
                BasicProperties::default().with_delivery_mode(2),
            )
            .await
            .context("publish failed")?
            .await
            .context("publish failed")?;

        try_wait_for_message(&channel, assert_queue).await
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use amq_protocol_types::{AMQPValue, LongString, ShortString};
    use env_logger::Env;

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        env_logger::from_env(Env::default().default_filter_or("debug")).init();
    }

    #[test]
    fn test_successful_routing() -> Result<()> {
        let test_queue_options = FieldTable::default();
        execute(
            &test_queue_options,
            AssertQueues {
                queue_with_massage: ROUTER_OUTPUT_QUEUE,
                queues_without_massage: vec![ROUTER_INPUT_QUEUE, ROUTER_INPUT_DLX_QUEUE],
            },
        )
    }

    #[test]
    fn test_failed_routing() -> Result<()> {
        let mut test_queue_options = FieldTable::default();
        test_queue_options.insert(ShortString::from("x-max-length"), AMQPValue::LongUInt(0));
        test_queue_options.insert(
            ShortString::from("x-overflow"),
            AMQPValue::LongString(LongString::from("reject-publish")),
        );
        execute(
            &test_queue_options,
            AssertQueues {
                queue_with_massage: ROUTER_INPUT_DLX_QUEUE,
                queues_without_massage: vec![ROUTER_INPUT_QUEUE, ROUTER_OUTPUT_QUEUE],
            },
        )
    }
}
