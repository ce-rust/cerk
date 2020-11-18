#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

extern crate ctor;

use env_logger::Env;

use std::env;
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties, Result as LapinResult, Channel};
use anyhow::{Context, Result, Error};
use std::{thread, time};

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

async fn has_message_on_output(channel: &Channel) -> Result<u32> {
    // todo does not work as intended, result.unwrap().message_count is always 0
    let result = channel.basic_get(TEST_QUEUE, BasicGetOptions { no_ack: false })
        .await?;

    if result.is_some() {
        Ok(1)
    } else {
        Ok(0)
    }
}

async fn create_test_queue(channel: &Channel) -> Result<()> {
    channel.queue_declare(TEST_QUEUE,
                          QueueDeclareOptions { nowait: false, auto_delete: false, durable: true, exclusive: false, passive: false },
                          FieldTable::default())
        .await.with_context(|| format!("was not able to create {} queue", TEST_QUEUE))?;
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


async fn set_up() -> Result<Channel> {
    let connection = connect()
        .await.context("amqp connect error")?;

    let channel = connection.create_channel()
        .await.context("create_channel error")?;

    create_test_queue(&channel).await?;

    let purge_count = channel.queue_purge(TEST_QUEUE, QueuePurgeOptions::default()).await?;
    info!("purge_count {}", purge_count);


    let count_before = has_message_on_output(&channel).await?;
    assert_eq!(count_before, 0, "should not have any message after prune");
    Ok(channel)
}

async fn try_wait_for_message(channel: &Channel) -> Result<(), Error> {
    let mut result: Result<()> = Err(anyhow!("message was not received on queue {}", ROUTER_INPUT_QUEUE));
    let ten_millis = time::Duration::from_millis(100);
    for _ in 0..100 {
        thread::sleep(ten_millis);
        let count = has_message_on_output(&channel).await?;
        if count == 1 {
            result = Ok::<(), Error>(()); // https://github.com/rust-lang/rust/issues/63502
            break;
        } else if count > 1 {
            assert_eq!(count, 1, "received more messages then sent");
        }
    }
    //assert_eq!(count_message_on_output(&channel).await?, 1);
    result
}

fn execute() -> Result<()> {
    async_global_executor::block_on(async {
        let channel = set_up().await?;


        let payload = r#"{"type":"test type","specversion":"1.0","source":"http://www.google.com","id":"id","contenttype":"application/json","data":"test"}"#;
        channel.basic_publish("",//default exchange
                              ROUTER_INPUT_QUEUE,
                              BasicPublishOptions { mandatory: true, immediate: false },
                              Vec::from(payload),
                              BasicProperties::default().with_delivery_mode(2))
            .await.context("publish failed")?
            .await.context("publish failed")?;

        try_wait_for_message(&channel).await
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        env_logger::from_env(Env::default().default_filter_or("debug")).init();
    }

    #[test]
    fn test_rabbitmq() -> Result<()> {
        execute()
    }
}

fn main() {}