use super::Config;
use crate::runtime::channel::BoxedSender;
use crate::runtime::{InternalServerFn, InternalServerId};
use cloudevents::v10::CloudEvent;
use std::fmt;

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

    /// The IncommingCloudEvent event indicates to the receiver that a new CloudEvent has been received from the outside world.
    /// The event is produced by an input port and is sent to the Kernel. The Kernel sends the same event to the router.
    ///
    /// # Arguments
    ///
    /// * `InternalServerId` - id of the component which received the CloudEvent
    /// * `CloudEvent` - the deserialized CloudEvent that the component has received
    ///
    IncommingCloudEvent(InternalServerId, CloudEvent),

    /// The OutgoingCloudEvent event indicates to the receiver that a CloudEvent has been routed and is ready to be forwarded to the outside world.
    /// The event is created by the router, send to the Kernel and then to the output port.
    /// One event for every output port which should forward the data is created.
    ///
    /// # Arguments
    ///
    /// * `CloudEvent` - the CloudEvent which should be forwarded
    /// * `InternalServerId` - the id of the component that should send the event
    ///
    OutgoingCloudEvent(CloudEvent, InternalServerId),

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
            BrokerEvent::IncommingCloudEvent(id, _) => {
                write!(f, "IncommingCloudEvent source_id={}", id)
            }
            BrokerEvent::ConfigUpdated(_, id) => write!(f, "ConfigUpdated destination_id={}", id),
            BrokerEvent::OutgoingCloudEvent(_, id) => {
                write!(f, "OutgoingCloudEvent destination_id={}", id)
            }
            BrokerEvent::Batch(_) => write!(f, "Batch"),
        }
    }
}
