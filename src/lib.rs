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
