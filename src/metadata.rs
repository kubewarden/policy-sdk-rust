use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// ProtocolVersion describes the version of the communication protocol
/// used to exchange information between the policy and the policy evaluator.
///
/// Policies built with this SDK provide the right value via the `protocol_version_guest`
/// function.
#[derive(Deserialize, Serialize, Debug, Clone, FromPrimitive, PartialEq)]
pub enum ProtocolVersion {
    /// This is an invalid version
    #[serde(rename = "Unknown")]
    Unknown = 0,
    #[serde(rename = "v1")]
    V1,
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::V1
    }
}

impl TryFrom<Vec<u8>> for ProtocolVersion {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let version: ProtocolVersion = serde_json::from_slice(&value)
            .map_err(|e| anyhow::anyhow!("Cannot convert value to ProtocolVersion: {:?}", e))?;
        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_known_version() -> Result<(), ()> {
        let version = ProtocolVersion::try_from(b"\"v1\"".to_vec());
        assert!(version.is_ok());
        assert_eq!(version.unwrap(), ProtocolVersion::V1);

        Ok(())
    }

    #[test]
    fn try_from_unknown_version() -> Result<(), ()> {
        let version = ProtocolVersion::try_from(b"\"v100\"".to_vec());
        assert!(version.is_err());

        Ok(())
    }
}
