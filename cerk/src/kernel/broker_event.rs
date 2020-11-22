use super::Config;
use crate::kernel::outgoing_processing_result::ProcessingResult;
use crate::kernel::CloudEventRoutingArgs;
use crate::runtime::channel::BoxedSender;
use crate::runtime::{InternalServerFn, InternalServerId};
use cloudevents::CloudEvent;
use std::fmt;

/// the unique identifier of the CloudEvent routing attempt
/// this id is generated on a receiver per CloudEvent and routing attempt.
///
/// If a CloudEvent has to be routed multiple times (e.g., delivery guarantee at least once) and the first routing fails (e.g., connection to outgoing queue was broken), then the message routing has to be retried.
/// For this retry a new  `CloudEventMessageRoutingId` has to be granted.
///
/// This id should not be tried to be interpreted or to be pared.
/// The generation is not defined globally and can be done differently by every port implementation.
pub type CloudEventMessageRoutingId = String;

/// Representation of all events which are exchanged between the components
pub enum BrokerEvent {
    /// The ScheduleInternalServer event tells the Scheduler to schedule a new internal server.
    /// One event is produced by the Kernel for each component.
    ///
    /// # Arguments
    ///
    /// * `InternalServerId` - id of the component that should be scheduled
    /// * `InternalServerFn` - start function of the component that should be scheduled
    ///
    ScheduleInternalServer(InternalServerId, InternalServerFn),

    /// The InernalServerScheduled event indicates to the receiver that a new internal server was successfully scheduled.
    /// The event gets produced by the scheduler after a component was scheduled (because of a ScheduleInternalServer event).
    /// The receiver is the Kernel.
    ///
    /// # Arguments
    ///
    /// * `InternalServerId` - id of the component that was scheduled
    /// * `BoxedSender` - channel inbox of the scheduled component
    ///
    InternalServerScheduled(InternalServerId, BoxedSender),

    /// The Init event indicates to the receiver that it should start interacting with the outside world.
    /// The event is produced by the Kernel when all components are scheduled.
    Init,

    /// The ConfigUpdated event indicates to the receiver that the config has changed and a configuration update should be applied.
    /// The event is produced by the router and send to the Kernel and then to the component.
    ///
    /// # Arguments
    ///
    /// * `Config` - the new configurations
    /// * `InternalServerId` - the component id for which the configurations are meant
    ///
    ConfigUpdated(Config, InternalServerId),

    /// The IncomingCloudEvent event indicates to the receiver that a new CloudEvent has been received from the outside world.
    /// The event is produced by an input port and is sent to the Kernel. The Kernel sends the same event to the router.
    ///
    /// # Arguments
    ///
    /// * `InternalServerId` - id of the component which received the CloudEvent
    /// * `CloudEventMessageRoutingId` - the unique identifier of the CloudEvent routing attempt
    /// * `CloudEvent` - the deserialized CloudEvent that the component has received
    /// * `CloudEventRoutingArgs` - routing arguments to define how a CloudEvent should be routed
    ///
    IncomingCloudEvent(
        InternalServerId,
        CloudEventMessageRoutingId,
        CloudEvent,
        CloudEventRoutingArgs,
    ),

    /// The `RoutingResult` is the result of a routing from one `IncomingCloudEvent`.
    /// The event is sent from the router to the kernel and there forwarded as `OutgoingCloudEvent` to the ports.
    ///
    /// The `RoutingResult` doesn't have to be sent if there are no destinations (`Vec<OutgoingCloudEvent>.len() == 0`)
    ///
    /// # Arguments
    ///
    /// * `CloudEventMessageRoutingId` - the unique identifier of the CloudEvent routing attempt
    /// * `InternalServerId` - the id of the component that received the event from the outside world
    /// * `Vec<OutgoingCloudEvent>` - the list of events that should be forwarded to the outgoing ports. This vec must only contain `BrokerEvent::OutgoingCloudEvent`. `CloudEventMessageRoutingId` in every message has to be the same as the main `CloudEventMessageRoutingId`.
    /// * `CloudEventRoutingArgs` - routing arguments to define how a CloudEvent should be routed - this config is used by the kernel; the args for the ports are inside the `Vec<OutgoingCloudEvent>`
    ///
    RoutingResult(
        CloudEventMessageRoutingId,
        InternalServerId,
        Vec<BrokerEvent>,
        CloudEventRoutingArgs,
    ),

    /// The OutgoingCloudEvent event indicates to the receiver that a CloudEvent has been routed and is ready to be forwarded to the outside world.
    /// The event is created by the router, send to the Kernel (in a badge as `RoutingResult`) and then to the output port(s).
    /// One event for every output port which should forward the data is created.
    ///
    /// # Arguments
    ///
    /// * `CloudEventMessageRoutingId` - the unique identifier of the CloudEvent routing attempt
    /// * `CloudEvent` - the CloudEvent which should be forwarded
    /// * `InternalServerId` - the id of the component that should send the event
    /// * `CloudEventRoutingArgs` - routing arguments to define how a CloudEvent should be routed
    ///
    OutgoingCloudEvent(
        CloudEventMessageRoutingId,
        CloudEvent,
        InternalServerId,
        CloudEventRoutingArgs,
    ),

    /// The OutgoingCloudEvent was processed.
    /// The OutgoingCloudEventProcessed notifies the kernel about the end of the processing and indicates whether the outcome was successful.
    /// This response is only used if the `CloudEventRoutingArgs` in the `OutgoingCloudEvent` event indicates  that a response is used  (`CloudEventRoutingArgs.delivery_guarantee.requires_acknowledgment()`).
    ///
    /// # Arguments
    /// * `InternalServerId` - the id of the component that processed the event (mostly sent to a queue)
    /// * `CloudEventMessageRoutingId` - the unique identifier of the CloudEvent routing attempt
    /// * `OutgoingProcessingResult` - result of the processing, was the processing successful? Error?
    ///
    OutgoingCloudEventProcessed(
        InternalServerId,
        CloudEventMessageRoutingId,
        ProcessingResult,
    ),

    /// The IncomingCloudEvent was processed.
    /// The IncomingCloudEventProcessed notifies the receiver port that the routing is completed and a response to the sender can be sent.
    /// This response is only used if the `CloudEventRoutingArgs` in the `IncomingCloudEvent` event indicates that a response is used (`CloudEventRoutingArgs.delivery_guarantee.requires_acknowledgment()`).
    ///
    /// # Arguments
    /// * `CloudEventMessageRoutingId` - the unique identifier of the CloudEvent routing attempt
    /// * `OutgoingProcessingResult` - result of the processing, was the processing successful? Error?
    ///
    IncomingCloudEventProcessed(CloudEventMessageRoutingId, ProcessingResult),

    /// The Batch event can be used to make sure a collection of events are processed by the MicroKernel in one batch to prevent race conditions.
    ///
    /// # Arguments
    ///
    /// * `Vec<BrokerEvent>` - a vector of `BrokerEvent`
    ///
    Batch(Vec<BrokerEvent>),
}

impl fmt::Display for BrokerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrokerEvent::Init => write!(f, "Init"),
            BrokerEvent::ScheduleInternalServer(id, _) => {
                write!(f, "ScheduleInternalServer server_id={}", id)
            }
            BrokerEvent::InternalServerScheduled(id, _) => {
                write!(f, "InternalServerScheduled server_id={}", id)
            }
            BrokerEvent::IncomingCloudEvent(id, _, _, _) => {
                write!(f, "IncomingCloudEvent source_id={}", id)
            }
            BrokerEvent::RoutingResult(id, _, _, _) => write!(f, "RoutingResult source_id={}", id),
            BrokerEvent::ConfigUpdated(_, id) => write!(f, "ConfigUpdated destination_id={}", id),
            BrokerEvent::OutgoingCloudEvent(_, _, id, _) => {
                write!(f, "OutgoingCloudEvent destination_id={}", id)
            }
            BrokerEvent::OutgoingCloudEventProcessed(_, _, state) => {
                write!(f, "OutgoingCloudEventProcessed state={}", state)
            }
            BrokerEvent::IncomingCloudEventProcessed(_, state) => {
                write!(f, "IncomingCloudEventProcessed state={}", state)
            }
            BrokerEvent::Batch(_) => write!(f, "Batch"),
        }
    }
}
