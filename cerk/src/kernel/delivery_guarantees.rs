use super::Config;
use std::convert::TryFrom;

/// Message delivery guarantees for the routing (defined per port channel)
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum DeliveryGuarantee {
    /// unspecified behaviour, the default
    Unspecified = 0,
    /// At Least Once the message should be received at the destination
    AtLeastOnce = 2,
}

impl DeliveryGuarantee {
    /// Does the selected delivery guarantee requires an acknowledgment?
    pub fn requires_acknowledgment(&self) -> bool {
        match self {
            DeliveryGuarantee::Unspecified => false,
            _ => true,
        }
    }
}

impl Default for DeliveryGuarantee {
    fn default() -> Self {
        DeliveryGuarantee::Unspecified
    }
}

impl TryFrom<Config> for DeliveryGuarantee {
    type Error = anyhow::Error;
    fn try_from(value: Config) -> Result<Self, Self::Error> {
        DeliveryGuarantee::try_from(&value)
    }
}

impl TryFrom<&Config> for DeliveryGuarantee {
    type Error = anyhow::Error;
    fn try_from(value: &Config) -> Result<Self, Self::Error> {
        if let Config::U8(number) = value {
            match number {
                0 => Ok(DeliveryGuarantee::Unspecified),
                2 => Ok(DeliveryGuarantee::AtLeastOnce),
                _ => bail!("number out of range"),
            }
        } else {
            bail!("Config not of type Config::U8")
        }
    }
}

impl From<DeliveryGuarantee> for Config {
    fn from(value: DeliveryGuarantee) -> Self {
        Config::U8(value as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn delivery_guarantee_to_config() {
        let delivery_guarantee = DeliveryGuarantee::AtLeastOnce;
        let config = Config::from(delivery_guarantee);
        assert_eq!(config, Config::U8(DeliveryGuarantee::AtLeastOnce as u8));
    }

    #[test]
    fn config_to_delivery_guarantee() -> Result<(), Box<dyn Error>> {
        let config = Config::U8(DeliveryGuarantee::AtLeastOnce as u8);
        let delivery_guarantee = DeliveryGuarantee::try_from(config)?;
        assert_eq!(delivery_guarantee, DeliveryGuarantee::AtLeastOnce);
        Ok(())
    }

    #[test]
    fn failed_config_to_delivery_guarantee() -> Result<(), Box<dyn Error>> {
        let config = Config::U8(99);
        let delivery_guarantee = DeliveryGuarantee::try_from(config);
        assert!(delivery_guarantee.is_err());
        Ok(())
    }
}
