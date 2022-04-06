use anyhow::{anyhow, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use url::Url;

use crate::host_capabilities::CallbackRequestType;

type WapcVerifyFN = fn(&[u8]) -> Result<Vec<u8>>;

pub fn verify_image(image: &str, config: LatestVerificationConfig) -> Result<bool> {
    wapc_invoke_verify_image(image, config, wapc_invoke_verify)
}

// This function is needed to have an easier way to test the verification outcomes when doing
// testing. Tests do NOT run inside of a Wasm environment, hence we can't call the actual waPC
// functions.
fn wapc_invoke_verify_image(
    image: &str,
    config: LatestVerificationConfig,
    wapc_verify_fn: WapcVerifyFN,
) -> Result<bool> {
    let req = CallbackRequestType::SigstoreVerify {
        image: image.to_string(),
        config,
    };

    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;

    let response_raw = wapc_verify_fn(&msg)?;

    let verified = *response_raw
        .first()
        .ok_or_else(|| anyhow!("error parsing the callback response"))?
        != 0;
    Ok(verified)
}

fn wapc_invoke_verify(payload: &[u8]) -> Result<Vec<u8>> {
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "verify", payload)
        .map_err(|e| anyhow::anyhow!("erorr invoking wapc logging facility: {:?}", e))?;
    Ok(response_raw)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub image: String,
    pub config: VerificationConfigV1,
}

fn default_minimum_matches() -> u8 {
    1
}

/// Alias to the type that is currently used to store the
/// verification settings.
///
/// When a new version is created:
/// * Update this stype to point to the new version
/// * Implement `TryFrom` that goes from (v - 1) to (v)
pub type LatestVerificationConfig = VerificationConfigV1;

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerificationConfigV1 {
    pub all_of: Option<Vec<Signature>>,
    pub any_of: Option<AnyOf>,
}

/// Enum that holds all the known versions of the configuration file
///
/// An unsupported version is a object that has `apiVersion` with an
/// unknown value (e.g: 1000)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "apiVersion", rename_all = "camelCase", deny_unknown_fields)]
pub enum VersionedVerificationConfig {
    #[serde(rename = "v1")]
    V1(VerificationConfigV1),
    #[serde(other)]
    Unsupported,
}

/// Enum that distinguish between a well formed (but maybe unknown) version of
/// the verification config, and something which is "just wrong".
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VerificationConfig {
    Versioned(VersionedVerificationConfig),
    Invalid(serde_yaml::Value),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnyOf {
    #[serde(default = "default_minimum_matches")]
    pub minimum_matches: u8,
    pub signatures: Vec<Signature>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "kind", deny_unknown_fields)]
pub enum Signature {
    PubKey {
        owner: Option<String>,
        key: String,
        annotations: Option<HashMap<String, String>>,
    },
    GenericIssuer {
        issuer: String,
        subject: Subject,
        annotations: Option<HashMap<String, String>>,
    },
    GithubAction {
        owner: String,
        repo: Option<String>,
        annotations: Option<HashMap<String, String>>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Subject {
    Equal(String),
    #[serde(deserialize_with = "deserialize_subject_url_prefix")]
    UrlPrefix(Url),
}

fn deserialize_subject_url_prefix<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    let mut url = Url::deserialize(deserializer)?;
    if !url.path().ends_with('/') {
        // sanitize url prefix path by postfixing `/`, to prevent
        // `https://github.com/kubewarden` matching
        // `https://github.com/kubewarden-malicious/`
        url.set_path(format!("{}{}", url.path(), '/').as_str());
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wapc_invoke_verify_ok(_payload: &[u8]) -> Result<Vec<u8>> {
        let is_trusted_byte: u8 = true.into();
        let is_trusted_vec_byte = vec![is_trusted_byte];

        Ok(is_trusted_vec_byte)
    }

    fn wapc_invoke_verify_err(_payload: &[u8]) -> Result<Vec<u8>> {
        Err(anyhow!("error"))
    }

    #[test]
    fn test_wapc_invoke_verify_ok() {
        let config = VerificationConfigV1 {
            all_of: None,
            any_of: None,
        };
        let result = wapc_invoke_verify_image("test", config, wapc_invoke_verify_ok);
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_wapc_invoke_verify_err() {
        let config = VerificationConfigV1 {
            all_of: None,
            any_of: None,
        };
        let result = wapc_invoke_verify_image("test", config, wapc_invoke_verify_err);
        assert_eq!(result.is_err(), true);
    }
}
