/*!

Helpers for the [lapin crate](https://crates.io/crates/lapin)

*/

use amq_protocol::protocol::{AMQPErrorKind, AMQPSoftError};
use amq_protocol_types::FieldTable;
use anyhow::Result;
use lapin::options::{QueueDeclareOptions, QueueDeleteOptions, ExchangeDeclareOptions, ExchangeDeleteOptions};
use lapin::{Channel, Connection, Queue, ExchangeKind};

/// same as queue_declare but on PRECONDITIONFAILED it does recreate the queue
pub async fn assert_queue(
    connection: &Connection,
    channel: &mut Channel,
    queue: &str,
    options: QueueDeclareOptions,
    arguments: FieldTable,
) -> Result<Queue> {
    let queue_declare = |c: &Channel| c.queue_declare(queue, options.clone(), arguments.clone());
    let mut result = queue_declare(channel).await;
    if let Err(lapin::Error::ProtocolError(protocol_error)) = result.as_ref() {
        if let AMQPErrorKind::Soft(soft_error) = protocol_error.kind() {
            match soft_error {
                AMQPSoftError::PRECONDITIONFAILED => {
                    *channel = connection.create_channel().await?;
                    channel
                        .queue_delete(queue, QueueDeleteOptions::default())
                        .await?;
                    result = Ok(queue_declare(channel).await?);
                }
                _ => (),
            }
        }
    }
    Ok(result?)
}

/// same as exchange_declare but on PRECONDITIONFAILED it does recreate the queue
pub async fn assert_exchange(
    connection: &Connection,
    channel: &mut Channel,
    exchange: &str,
    kind: ExchangeKind,
    options: ExchangeDeclareOptions,
    arguments: FieldTable,
) -> Result<()> {
    let exchange_declare = |c: &Channel| c.exchange_declare(exchange, kind.clone(), options.clone(), arguments.clone());
    let mut result = exchange_declare(channel).await;
    if let Err(lapin::Error::ProtocolError(protocol_error)) = result.as_ref() {
        if let AMQPErrorKind::Soft(soft_error) = protocol_error.kind() {
            match soft_error {
                AMQPSoftError::PRECONDITIONFAILED => {
                    *channel = connection.create_channel().await?;
                    channel
                        .exchange_delete(exchange, ExchangeDeleteOptions::default())
                        .await?;
                    exchange_declare(channel).await?;
                    result = Ok(());
                }
                _ => (),
            }
        }
    }
    result?;
    Ok(())
}
