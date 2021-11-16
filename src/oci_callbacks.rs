use anyhow::{anyhow, Result};

#[cfg(not(target_arch = "wasm32"))]
pub fn oci_manifest_digest(image: &str) -> Result<String> {
    //TODO do something better than that
    Err(anyhow!(
        "oci_manifest_digest for image {}: not implemented",
        image
    ))
}
#[cfg(target_arch = "wasm32")]
pub fn oci_manifest_digest(image: &str) -> Result<String> {
    match wapc_guest::host_call("kubewarden", "oci", "manifest_digest", image.as_bytes()) {
        Ok(d) => String::from_utf8(d)
            .map_err(|_| anyhow!("Cannot convert oci.manifest_digest result into String")),
        Err(e) => Err(anyhow!("Error invoking oci.oci_manifest_digest: {:?}", e)),
    }
}
