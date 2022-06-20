use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Mutated Object - used only by mutation policies
    pub mutated_object: Option<serde_json::Value>,
    /// AuditAnnotations is an unstructured key value map set by remote admission controller (e.g. error=image-blacklisted).
    /// MutatingAdmissionWebhook and ValidatingAdmissionWebhook admission controller will prefix the keys with
    /// admission webhook name (e.g. imagepolicy.example.com/error=image-blacklisted). AuditAnnotations will be provided by
    /// the admission webhook to add additional context to the audit log for this request.
    pub audit_annotations: Option<HashMap<String, String>>,
    /// warnings is a list of warning messages to return to the requesting API client.
    /// Warning messages describe a problem the client making the API request should correct or be aware of.
    /// Limit warnings to 120 characters if possible.
    /// Warnings over 256 characters and large numbers of warnings may be truncated.
    pub warnings: Option<Vec<String>>,
}
