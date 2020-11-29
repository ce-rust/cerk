use super::DeliveryGuarantee;
use serde::{Serialize, Deserialize};

/// CloudEventRoutingArgs specifies how a CloudEvent should be routed
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudEventRoutingArgs {
    /// Message delivery guarantees with which the CloudEvent was received
    pub delivery_guarantee: DeliveryGuarantee,
}
