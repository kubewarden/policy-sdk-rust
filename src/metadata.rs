use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

/// ProtocolVersion describes the version of the communication protocol
/// used to exchange information between the policy and the policy evaluator.
///
/// Policies built with this SDK provide the right value via the `protocol_version_guest`
/// function.
#[derive(Deserialize, Serialize, Debug, Clone, FromPrimitive, ToPrimitive, PartialEq, Eq)]
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

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = num::ToPrimitive::to_u64(self).ok_or(fmt::Error)?;
        write!(f, "{version}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_try_display() {
        let version = ProtocolVersion::V1;
        assert_eq!("1", format!("{version}"));

        let version = ProtocolVersion::Unknown;
        assert_eq!("0", format!("{version}"));
    }

    #[test]
    fn protocol_version_try_from_known_version() {
        let version = ProtocolVersion::try_from(b"\"v1\"".to_vec());
        assert!(version.is_ok());
        assert_eq!(version.unwrap(), ProtocolVersion::V1);
    }

    #[test]
    fn protocol_version_try_from_unknown_version() {
        let version = ProtocolVersion::try_from(b"\"v100\"".to_vec());
        assert!(version.is_err());
    }
}
