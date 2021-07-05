extern crate k8s_openapi;

use anyhow::anyhow;

pub mod cluster_context;
pub mod logging;
pub mod metadata;
pub mod request;
pub mod response;
pub mod settings;
pub mod test;

use crate::metadata::ProtocolVersion;
use crate::response::*;

/// Create an acceptance response
pub fn accept_request() -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ValidationResponse {
        accepted: true,
        message: None,
        code: None,
        mutated_object: None,
    })?)
}

/// Create an acceptance response that mutates the original object
/// # Arguments
/// * `mutated_object` - the mutated Object
pub fn mutate_request(mutated_object: serde_json::Value) -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ValidationResponse {
        accepted: true,
        message: None,
        code: None,
        mutated_object: Some(mutated_object),
    })?)
}

/// Create a rejection response
/// # Arguments
/// * `message` -  message shown to the user
/// * `code`    -  code shown to the user
pub fn reject_request(message: Option<String>, code: Option<u16>) -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ValidationResponse {
        accepted: false,
        mutated_object: None,
        message,
        code,
    })?)
}

/// waPC guest function to register under the name `validate_settings`
/// # Example
///
/// ```
/// use kubewarden_policy_sdk::{validate_settings, settings::Validatable};
/// use serde::Deserialize;
/// use wapc_guest::register_function;
///
/// // This module settings require either `setting_a` or `setting_b`
/// // set. Both cannot be provided at the same time, and one has to be
/// // provided.
/// #[derive(Deserialize)]
/// struct Settings {
///   setting_a: Option<String>,
///   setting_b: Option<String>
/// }
///
/// impl Validatable for Settings {
///   fn validate(&self) -> Result<(), String> {
///     if self.setting_a.is_none() && self.setting_b.is_none() {
///       Err("either setting A or setting B has to be provided".to_string())
///     } else if self.setting_a.is_some() && self.setting_b.is_some() {
///       Err("setting A and setting B cannot be set at the same time".to_string())
///     } else {
///       Ok(())
///     }
///   }
/// }
///
/// register_function("validate_settings", validate_settings::<Settings>);
/// ```
pub fn validate_settings<T>(payload: &[u8]) -> wapc_guest::CallResult
where
    T: serde::de::DeserializeOwned + settings::Validatable,
{
    let settings: T = serde_json::from_slice::<T>(payload).map_err(|e| {
        anyhow!(
            "Error decoding validation payload {}: {:?}",
            String::from_utf8_lossy(payload),
            e
        )
    })?;

    let res = match settings.validate() {
        Ok(_) => settings::SettingsValidationResponse {
            valid: true,
            message: None,
        },
        Err(e) => settings::SettingsValidationResponse {
            valid: false,
            message: Some(e),
        },
    };

    Ok(serde_json::to_vec(&res)?)
}

/// Helper function that provides the `protocol_version` implementation
/// # Example
///
/// ```
/// extern crate wapc_guest as guest;
/// use guest::prelude::*;
/// use kubewarden_policy_sdk::protocol_version_guest;
///
/// #[no_mangle]
/// pub extern "C" fn wapc_init() {
///     register_function("protocol_version", protocol_version_guest);
///     // register other waPC functions
/// }
/// ```
pub fn protocol_version_guest(_payload: &[u8]) -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ProtocolVersion::default())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    #[test]
    fn test_mutate_request() -> Result<(), ()> {
        let mutated_object = json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "name": "security-context-demo-4"
            },
            "spec": {
                "containers": [
                {
                    "name": "sec-ctx-4",
                    "image": "gcr.io/google-samples/node-hello:1.0",
                    "securityContext": {
                        "capabilities": {
                            "add": ["NET_ADMIN", "SYS_TIME"],
                            "drop": ["BPF"]
                        }
                    }
                }
                ]
            }
        });
        let expected_object = mutated_object.clone();

        let reponse_raw = mutate_request(mutated_object).unwrap();
        let response: ValidationResponse = serde_json::from_slice(&reponse_raw).unwrap();

        assert_json_eq!(response.mutated_object, expected_object);

        Ok(())
    }

    #[test]
    fn test_accept_request() -> Result<(), ()> {
        let reponse_raw = accept_request().unwrap();
        let response: ValidationResponse = serde_json::from_slice(&reponse_raw).unwrap();

        assert!(response.mutated_object.is_none());
        Ok(())
    }

    #[test]
    fn test_reject_request() -> Result<(), ()> {
        let code = 500;
        let expected_code = code.clone();

        let message = String::from("internal error");
        let expected_message = message.clone();

        let reponse_raw = reject_request(Some(message), Some(code)).unwrap();
        let response: ValidationResponse = serde_json::from_slice(&reponse_raw).unwrap();

        assert!(response.mutated_object.is_none());
        assert_eq!(response.code, Some(expected_code));
        assert_eq!(response.message, Some(expected_message));
        Ok(())
    }

    #[test]
    fn try_protocol_version_guest() -> Result<(), ()> {
        let reponse = protocol_version_guest(&[0; 0]).unwrap();
        let version: ProtocolVersion = serde_json::from_slice(&reponse).unwrap();

        assert_eq!(version, ProtocolVersion::V1);
        Ok(())
    }
}
