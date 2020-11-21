use super::DeliveryGuarantee;

/// CloudEventRoutingArgs specifies how a CloudEvent should be routed
#[derive(Default, Debug, Clone)]
pub struct CloudEventRoutingArgs {
    /// Message delivery guarantees with which the CloudEvent was received
    pub delivery_guarantee: DeliveryGuarantee,
}
