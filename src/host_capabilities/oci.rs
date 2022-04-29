use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Response to manifest digest request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestDigestResponse {
    /// list of Ips that have been resolved
    pub digest: String,
}

/// Computes the digest of the OCI object referenced by `image`
pub fn manifest_digest(image: &str) -> Result<ManifestDigestResponse> {
    let req = json!(image);
    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "v1/manifest_digest", &msg)
        .map_err(|e| anyhow!("error invoking wapc oci.manifest_digest: {:?}", e))?;

    let response: ManifestDigestResponse = serde_json::from_slice(&response_raw)?;

    Ok(response)
}
