use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::host_capabilities::CallbackRequestType;

type WapcVerifyFN = fn(&[u8]) -> Result<Vec<u8>>;

pub fn verify_pub_keys_image(
    image: &str,
    pub_keys: Vec<String>,
    annotations: HashMap<String, String>,
) -> Result<bool> {
    wapc_invoke_verify_pub_keys_image(image, pub_keys, annotations, wapc_invoke_verify)
}

// This function is needed to have an easier way to test the verification outcomes when doing
// testing. Tests do NOT run inside of a Wasm environment, hence we can't call the actual waPC
// functions.
fn wapc_invoke_verify_pub_keys_image(
    image: &str,
    pub_keys: Vec<String>,
    annotations: HashMap<String, String>,
    wapc_verify_fn: WapcVerifyFN,
) -> Result<bool> {
    let req = CallbackRequestType::SigstorePubKeyVerify {
        image: image.to_string(),
        pub_keys,
        annotations,
    };

    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;

    let response_raw = wapc_verify_fn(&msg)?;

    //TODO this must be a json
    let verified = *response_raw
        .first()
        .ok_or_else(|| anyhow!("error parsing the callback response"))?
        != 0;
    Ok(verified)
}

fn wapc_invoke_verify(payload: &[u8]) -> Result<Vec<u8>> {
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "v1/verify/pubkeys", payload)
        .map_err(|e| anyhow::anyhow!("erorr invoking wapc logging facility: {:?}", e))?;
    Ok(response_raw)
}
