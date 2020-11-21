#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

extern crate ctor;

use env_logger::Env;

use std::env;
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties, Result as LapinResult, Channel, Queue};
use anyhow::{Context, Result, Error};
use std::{thread, time};
use amq_protocol_types::{AMQPValue, ShortString};

const TEST_QUEUE: &'static str = "test_queue_router_output";
const ROUTER_INPUT_QUEUE: &'static str = "router_input";
const ROUTER_OUTPUT_EXCHANGE: &'static str = "router_output";

async fn connect() -> LapinResult<Connection> {
    let amqp_broker_uri: String = env::var("AMQP_BROKER_URL").unwrap_or(String::from("amqp://127.0.0.1:5672/%2f"));

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
    let result = channel.basic_get(queue, BasicGetOptions { no_ack: false })
        .await?;

    if result.is_some() {
        Ok(1)
    } else {
        Ok(0)
    }
}

async fn create_test_queue(channel: &Channel, options: &FieldTable) -> Result<Queue> {
    channel.queue_declare(TEST_QUEUE,
                          QueueDeclareOptions { nowait: false, auto_delete: false, durable: true, exclusive: false, passive: false },
                          options.clone())
        .await.with_context(|| format!("was not able to create {} queue", TEST_QUEUE))
}

async fn create_test_and_bind_queue(connection: &Connection, test_queue_options: &FieldTable) -> Result<()> {
    let mut channel = open_channel(connection).await?;
    let create_queue = create_test_queue(&channel, test_queue_options).await;
    if create_queue.is_err() {
        warn!("failed to queue_declare - try to delete and create again: {}", create_queue.err().unwrap());
        // channel closes after failure
        channel = open_channel(connection).await?;
        channel.queue_delete(TEST_QUEUE, QueueDeleteOptions::default()).await?;
        create_test_queue(&channel, test_queue_options).await?;
    }
    info!("test queue created");

    channel.queue_bind(TEST_QUEUE,
                       ROUTER_OUTPUT_EXCHANGE,
                       "",
                       QueueBindOptions::default(),
                       FieldTable::default())
        .await.context("was not able to bind queue")?;
    info!("test queue bounded");

    Ok(())
}


async fn open_channel(connection: &Connection) -> Result<Channel> {
    Ok(connection.create_channel()
        .await.context("create_channel error")?)
}

async fn set_up(test_queue_options: &FieldTable) -> Result<Channel> {
    let connection = connect()
        .await.context("amqp connect error")?;

    create_test_and_bind_queue(&connection, test_queue_options).await?;

    let channel = open_channel(&connection).await?;

    channel.queue_purge(TEST_QUEUE, QueuePurgeOptions::default()).await?;
    channel.queue_purge(ROUTER_INPUT_QUEUE, QueuePurgeOptions::default()).await?;


    let count_before = has_message_on_output(&channel, TEST_QUEUE).await?;
    assert_eq!(count_before, 0, "should not have any message after prune {}", TEST_QUEUE);
    let count_before = has_message_on_output(&channel, ROUTER_INPUT_QUEUE).await?;
    assert_eq!(count_before, 0, "should not have any message after prune on {}", ROUTER_INPUT_QUEUE);
    Ok(channel)
}

struct AssertQueues {
    queue_with_massage: &'static str,
    queue_without_massage: &'static str,
}

async fn try_wait_for_message(channel: &Channel, assert: AssertQueues) -> Result<(), Error> {
    let mut result: Result<()> = Err(anyhow!("message was not received on queue {}", assert.queue_with_massage));
    let ten_millis = time::Duration::from_millis(100);
    for _ in 0..100 {
        thread::sleep(ten_millis);
        let count = has_message_on_output(&channel, assert.queue_with_massage).await?;
        if count == 1 {
            result = Ok::<(), Error>(()); // https://github.com/rust-lang/rust/issues/63502
            let second_count = has_message_on_output(&channel, assert.queue_without_massage).await?;
            assert_eq!(second_count, 0, "there should be no message on queue {} but it has count {}", assert.queue_without_massage, second_count);
            break;
        } else if count > 1 {
            assert_eq!(count, 1, "received more messages then sent; count {} on queue {}", count, assert.queue_with_massage);
        }
    }

    result
}

fn execute(test_queue_options: &FieldTable, assert_queue: AssertQueues) -> Result<()> {
    async_global_executor::block_on(async {
        let channel = set_up(test_queue_options).await?;


        let payload = r#"{"type":"test type","specversion":"1.0","source":"http://www.google.com","id":"id","contenttype":"application/json","data":"test"}"#;
        channel.basic_publish("",//default exchange
                              ROUTER_INPUT_QUEUE,
                              BasicPublishOptions { mandatory: true, immediate: false },
                              Vec::from(payload),
                              BasicProperties::default().with_delivery_mode(2))
            .await.context("publish failed")?
            .await.context("publish failed")?;

        try_wait_for_message(&channel, assert_queue).await
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use amq_protocol_types::LongString;

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        env_logger::from_env(Env::default().default_filter_or("debug")).init();
    }

    #[test]
    fn test_successful_routing() -> Result<()> {
        let test_queue_options = FieldTable::default();
        execute(&test_queue_options, AssertQueues { queue_with_massage: TEST_QUEUE, queue_without_massage: ROUTER_INPUT_QUEUE })
    }

    // #[test()]
    // todo fails at the moment -> we have to fix this
    fn test_failed_routing() -> Result<()> {
        let mut test_queue_options = FieldTable::default();
        test_queue_options.insert(ShortString::from("x-max-length"), AMQPValue::LongUInt(0));
        test_queue_options.insert(ShortString::from("x-overflow"), AMQPValue::LongString(LongString::from("reject-publish")));
        execute(&test_queue_options, AssertQueues { queue_with_massage: ROUTER_INPUT_QUEUE, queue_without_massage: TEST_QUEUE })
    }
}
