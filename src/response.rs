use serde::{Deserialize, Serialize};

/// A ValidationResponse object holds the outcome of policy
/// evaluation.
#[derive(Deserialize, Serialize, Debug)]
pub struct ValidationResponse {
    /// True if the request has been accepted, false otherwise
    pub accepted: bool,
    /// Message shown to the user when the request is rejected
    pub message: Option<String>,
    /// Code shown to the user when the request is rejected
    pub code: Option<u16>,
    /// Mutated Object serialized using JSON format - used only by mutation policies
    pub mutated_object: Option<String>,
}
