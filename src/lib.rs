use anyhow::anyhow;

pub mod request;
pub mod response;
pub mod settings;
pub mod test;

use crate::response::*;

/// Create an acceptance response
pub fn accept_request() -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ValidationResponse {
        accepted: true,
        message: None,
        code: None,
    })?)
}

/// Create a rejection response
/// # Arguments
/// * `message` -  message shown to the user
/// * `code`    -  code shown to the user
pub fn reject_request(message: Option<String>, code: Option<u16>) -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ValidationResponse {
        accepted: false,
        message,
        code,
    })?)
}

/// waPC guest function to register under the name `validate_settings`
/// # Example
//
/// ```
/// // Settings is a user defined Setting struct
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
