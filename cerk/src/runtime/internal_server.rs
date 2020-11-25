use super::channel::{BoxedReceiver, BoxedSender};

/// Type for the port identifier, every port instance could be identified with this id.
pub type InternalServerId = String;

/// This is the function signature for every component, except the Kernel and the Scheduler.
/// The function gets executed by the scheduler.
///
/// A component implementation can be started multiple times with different ids and configurations.
/// E.g. a UNIX Socket Input Port could be started twice to listen to two different sockets.
///
/// # Arguments
///
/// * `id` - the id of the port
/// * `inbox` - The channel inbox, the component should listen for incomming messages and process them.
/// * `sender_to_kernel` - the channel inbox of the Kernel. Messages for the Kernel could be sent with this component.
///
/// # Example
/// ```
/// use cerk::kernel::BrokerEvent;
/// use cerk::runtime::{InternalServerId, InternalServerFn, InternalServerFnRefStatic};
/// use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
///
/// fn dummy_port(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender,) {
///   loop {
///     match inbox.receive() {
///       BrokerEvent::Init => print!("{} component initialized", id),
///       broker_event => print!("{} event {} not implemented", id, broker_event),
///     }
///   }
/// }
///
/// fn main() {
///     let port: InternalServerFn = dummy_port;
///     let port_ref: InternalServerFnRefStatic = &(dummy_port as InternalServerFn);
/// }
/// ```
pub type InternalServerFn =
    fn(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender);

/// reference for type InternalServerFn
pub type InternalServerFnRef<'a> = &'a InternalServerFn;

/// static reference for type InternalServerFn
pub type InternalServerFnRefStatic = &'static InternalServerFn;
