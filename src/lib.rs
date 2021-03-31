extern crate k8s_openapi;

use anyhow::anyhow;

pub mod cluster_context;
pub mod request;
pub mod response;
pub mod settings;
pub mod test;

use crate::response::*;

/// Create an acceptance response
/// # Arguments
/// * `mutated_object` - the mutated Object - used only by mutation policies
pub fn accept_request(mutated_object: Option<serde_json::Value>) -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ValidationResponse {
        accepted: true,
        message: None,
        code: None,
        mutated_object: mutated_object.map(|o| {
            serde_json::to_string(&o)
                .map_err(|e| anyhow!("cannot serialize mutated object: {:?}", e))
                .unwrap()
        }),
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
/// use chimera_kube_policy_sdk::{validate_settings, settings::Validatable};
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
