use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::host_capabilities::CallbackRequestType;

/// TODO: write docs
pub fn verify_image(image: &str, config: Config) -> Result<bool> {
    let req = CallbackRequestType::SigstoreVerify {
        image: image.to_string(),
        config,
    };

    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;

    let response_raw = wapc_invoke_verify(&msg)?;

    let verified: bool = serde_json::from_slice(&response_raw)
        .map_err(|e| anyhow!("Cannot decode verification response: {}", e))?;
    Ok(verified)
}

#[cfg(target_arch = "wasm32")]
fn wapc_invoke_verify(payload: &[u8]) -> Result<Vec<u8>> {
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "verify", &msg)
        .map_err(|e| anyhow::anyhow!("erorr invoking wapc logging facility: {:?}", e))?;
    response_raw.into()
}

#[cfg(not(target_arch = "wasm32"))]
fn wapc_invoke_verify(_payload: &[u8]) -> Result<Vec<u8>> {
    //TODO find a way to allow users to provide different results
    //a global maybe?
    Ok(br#"true"#.to_vec())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub image: String,
    pub config: Config,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigV1 {
    pub all_of: Option<Vec<Signature>>,
    pub any_of: Option<AnyOf>,
}

/// Enum that holds all the known versions of the configuration file
///
/// An unsupported version is a object that has `apiVersion` with an
/// unknown value (e.g: 1000)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "apiVersion", rename_all = "camelCase", deny_unknown_fields)]
pub enum VersionedConfig {
    #[serde(rename = "v1")]
    V1(ConfigV1),
    #[serde(other)]
    Unsupported,
}

/// Enum that distinguish between a well formed (but maybe unknown) version of
/// the verification config, and something which is "just wrong".
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Config {
    Versioned(VersionedConfig),
    Invalid(),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnyOf {
    #[serde(default = "default_minimum_matches")]
    pub minimum_matches: u8,
    pub signatures: Vec<Signature>,
}

fn default_minimum_matches() -> u8 {
    1
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
    UrlPrefix(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    //TODO: write tests

    // In this case, the settings of the policy defined by the user is going
    // to embed the configuration object:
    //
    // ```yaml
    // somethingUnrelated: hello world
    // verificationConfig:
    //   apiVersion: v1
    //   allOf:
    //     - kind: genericIssuer
    //       issuer: https://token.actions.githubusercontent.com
    //       subject:
    //          urlPrefix: https://github.com/kubewarden # should deserialize path to kubewarden/
    //     - kind: genericIssuer
    //       issuer: https://yourdomain.com/oauth2
    //       subject:
    //          urlPrefix: https://github.com/kubewarden/ # should deserialize path to kubewarden/
    //
    // ```
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct SettingsExample {
        something_unrelated: String,
        verification_config: Config,
    }

    // In this case, the settings of the policy defined by the user is going
    // to use some primites of the SDK Verification::Config
    //
    // ```yaml
    // somethingUnrelated: hello world
    // verificationConfig:
    //   allOf:
    //     - kind: genericIssuer
    //       issuer: https://token.actions.githubusercontent.com
    //       subject:
    //          urlPrefix: https://github.com/kubewarden # should deserialize path to kubewarden/
    //     - kind: genericIssuer
    //       issuer: https://yourdomain.com/oauth2
    //       subject:
    //          urlPrefix: https://github.com/kubewarden/ # should deserialize path to kubewarden/
    //
    // ```
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct SettingsCustomExample {
        something_unrelated: String,
        verification_config: ConfigV1,
    }
}
