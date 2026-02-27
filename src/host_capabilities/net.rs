use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Response to host lookup requests
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LookupResponse {
    /// list of Ips that have been resolved
    pub ips: Vec<String>,
}

/// Lookup the addresses for a given hostname via DNS
pub fn lookup_host(host: &str) -> Result<LookupResponse> {
    let req = json!(host);
    let msg = serde_json::to_vec(&req).map_err(|e| Error::Serialization {
        context: "DNS lookup request".to_string(),
        source: e,
    })?;
    let response_raw = wapc_guest::host_call("kubewarden", "net", "v1/dns_lookup_host", &msg)
        .map_err(|e| Error::HostCall {
            operation: "net.dns_lookup_host".to_string(),
            source: e,
        })?;

    let response: LookupResponse = serde_json::from_slice(&response_raw)?;

    Ok(response)
}
