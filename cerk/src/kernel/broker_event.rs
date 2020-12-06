use super::Config;
use crate::kernel::outgoing_processing_result::ProcessingResult;
use crate::kernel::CloudEventRoutingArgs;
use crate::runtime::channel::BoxedSender;
use crate::runtime::{InternalServerFnRef, InternalServerId};
use cloudevents::Event;
use serde::Serialize;
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
    ScheduleInternalServer(ScheduleInternalServerStatic),

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
    IncomingCloudEvent(IncomingCloudEvent),

    /// The `RoutingResult` is the result of a routing from one `IncomingCloudEvent`.
    /// The event is sent from the router to the kernel and there forwarded as `OutgoingCloudEvent` to the ports.
    ///
    /// The `RoutingResult` doesn't have to be sent if there are no destinations (`Vec<OutgoingCloudEvent>.len() == 0`)
    RoutingResult(RoutingResult),

    /// The OutgoingCloudEvent event indicates to the receiver that a CloudEvent has been routed and is ready to be forwarded to the outside world.
    /// The event is created by the router, send to the Kernel (in a badge as `RoutingResult`) and then to the output port(s).
    /// One event for every output port which should forward the data is created.
    OutgoingCloudEvent(OutgoingCloudEvent),

    /// The OutgoingCloudEvent was processed.
    /// The OutgoingCloudEventProcessed notifies the kernel about the end of the processing and indicates whether the outcome was successful.
    /// This response is only used if the `CloudEventRoutingArgs` in the `OutgoingCloudEvent` event indicates  that a response is used  (`CloudEventRoutingArgs.delivery_guarantee.requires_acknowledgment()`).
    OutgoingCloudEventProcessed(OutgoingCloudEventProcessed),

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

    /// A health check port sends `HealthCheckRequest` to some components, they should response with `HealthCheckResponse`
    HealthCheckRequest(HealthCheckRequest),

    /// response for `HealthCheckRequest`, should go to a health check component
    HealthCheckResponse(HealthCheckResponse),
}

impl fmt::Display for BrokerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrokerEvent::Init => write!(f, "Init"),
            BrokerEvent::ScheduleInternalServer(event) => {
                write!(f, "ScheduleInternalServer server_id={}", event.id)
            }
            BrokerEvent::InternalServerScheduled(id, _) => {
                write!(f, "InternalServerScheduled server_id={}", id)
            }
            BrokerEvent::IncomingCloudEvent(event) => {
                write!(f, "IncomingCloudEvent receiver_id={}", event.incoming_id)
            }
            BrokerEvent::RoutingResult(event) => {
                write!(f, "RoutingResult receiver_id={}", event.incoming_id)
            }
            BrokerEvent::ConfigUpdated(_, id) => write!(f, "ConfigUpdated destination_id={}", id),
            BrokerEvent::OutgoingCloudEvent(event) => write!(
                f,
                "OutgoingCloudEvent destination_id={}",
                event.destination_id
            ),
            BrokerEvent::OutgoingCloudEventProcessed(event) => {
                write!(f, "OutgoingCloudEventProcessed result={}", event.result)
            }
            BrokerEvent::IncomingCloudEventProcessed(_, state) => {
                write!(f, "IncomingCloudEventProcessed state={}", state)
            }
            BrokerEvent::Batch(_) => write!(f, "Batch"),
            BrokerEvent::HealthCheckRequest(_) => write!(f, "HealthCheckRequest"),
            BrokerEvent::HealthCheckResponse(_) => write!(f, "HealthCheckResponse"),
        }
    }
}

/// Struct for `BrokerEvent::IncomingCloudEvent`
#[derive(Clone, Debug, PartialEq)]
pub struct IncomingCloudEvent {
    /// id of the component which received the CloudEvent
    pub incoming_id: InternalServerId,
    /// the unique identifier of the CloudEvent routing attempt
    pub routing_id: CloudEventMessageRoutingId,
    /// the deserialized CloudEvent that the component has received
    pub cloud_event: Event,
    /// routing arguments to define how a CloudEvent should be routed
    pub args: CloudEventRoutingArgs,
}

/// Struct for `BrokerEvent::RoutingResult`
#[derive(Clone, Debug, PartialEq)]
pub struct RoutingResult {
    /// the id of the component that received the event from the outside world
    pub incoming_id: InternalServerId,
    /// the unique identifier of the CloudEvent routing attempt
    pub routing_id: CloudEventMessageRoutingId,
    /// The list of events that should be forwarded to the outgoing ports.
    /// Multiple routing to the same destination_id with a `delivery_guarantee.requires_acknowledgment()` are currently not supported by the kernel.
    pub routing: Vec<OutgoingCloudEvent>,
    /// routing arguments to define how a CloudEvent should be routed - this config is used by the kernel; the args for the ports are inside the `Vec<OutgoingCloudEvent>`
    pub args: CloudEventRoutingArgs,
    /// outcome of the routing, was it successful?
    pub result: ProcessingResult,
}

/// Struct for `BrokerEvent::OutgoingCloudEvent`
#[derive(Clone, Debug, PartialEq)]
pub struct OutgoingCloudEvent {
    /// the unique identifier of the CloudEvent routing attempt
    pub routing_id: CloudEventMessageRoutingId,
    /// the CloudEvent which should be forwarded
    pub cloud_event: Event,
    /// the id of the component that should send the event
    pub destination_id: InternalServerId,
    /// routing arguments to define how a CloudEvent should be routed
    pub args: CloudEventRoutingArgs,
}

/// Struct for `BrokerEvent::OutgoingCloudEventProcessed`
#[derive(Clone, Debug, PartialEq)]
pub struct OutgoingCloudEventProcessed {
    /// the id of the component that processed the event (mostly sent to a queue)
    pub sender_id: InternalServerId,
    /// the unique identifier of the CloudEvent routing attempt
    pub routing_id: CloudEventMessageRoutingId,
    /// result of the processing, was the processing successful? Error?
    pub result: ProcessingResult,
}

/// Struct for `BrokerEvent::ScheduleInternalServer`
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleInternalServer<'a> {
    /// id of the service that should be scheduled
    pub id: InternalServerId,
    /// pointer to the start function
    pub function: InternalServerFnRef<'a>,
}

/// Struct for `BrokerEvent::HealthCheckRequest`
pub struct HealthCheckRequest {
    /// id of the health check
    pub id: String,
    /// the id of the component that created the request
    pub sender_id: InternalServerId,
    /// the port that should response to the health check
    pub destination_id: InternalServerId,
}

/// Struct for `BrokerEvent::HealthCheckResponse`
pub struct HealthCheckResponse {
    /// id of the health check
    pub id: String,
    /// the id of the component that responded to the health request
    pub sender_id: InternalServerId,
    /// routing destination of the health response (HealthCheckRequest.sender_id)
    pub destination_id: InternalServerId,
    /// status of the component
    pub status: HealthCheckStatus,
}

/// health check status
#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum HealthCheckStatus {
    /// the component is healthy and fully functional
    Healthy,
    /// the component is unhealthy, message to indicate the problem
    Unhealthy(String),
}

/// Fixed static lifetime for struct for `BrokerEvent::ScheduleInternalServer`
pub type ScheduleInternalServerStatic = ScheduleInternalServer<'static>;
