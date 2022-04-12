use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::host_capabilities::CallbackRequestType;

type WapcVerifyFN = fn(&[u8]) -> Result<Vec<u8>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeylessInfo {
    pub issuer: String,
    pub subject: String,
}

pub fn verify_pub_keys_image(
    image: &str,
    pub_keys: Vec<String>,
    annotations: Option<HashMap<String, String>>,
) -> Result<bool> {
    invoke_verify_pub_keys_image(image, pub_keys, annotations, wapc_invoke_verify)
}

pub fn verify_keyless_exact_match(
    image: &str,
    keyless: Vec<KeylessInfo>,
    annotations: Option<HashMap<String, String>>,
) -> Result<bool> {
    invoke_verify_keyless_exact_match(image, keyless, annotations, wapc_invoke_verify)
}

// This function is needed to have an easier way to test the verification outcomes when doing
// testing. Tests do NOT run inside of a Wasm environment, hence we can't call the actual waPC
// functions.
fn invoke_verify_pub_keys_image(
    image: &str,
    pub_keys: Vec<String>,
    annotations: Option<HashMap<String, String>>,
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

// This function is needed to have an easier way to test the verification outcomes when doing
// testing. Tests do NOT run inside of a Wasm environment, hence we can't call the actual waPC
// functions.
fn invoke_verify_keyless_exact_match(
    image: &str,
    keyless: Vec<KeylessInfo>,
    annotations: Option<HashMap<String, String>>,
    wapc_verify_fn: WapcVerifyFN,
) -> Result<bool> {
    let req = CallbackRequestType::SigstoreKeylessVerify {
        image: image.to_string(),
        keyless,
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
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "v1/verify", payload)
        .map_err(|e| anyhow::anyhow!("error invoking wapc verify: {:?}", e))?;
    Ok(response_raw)
}

//TODO tests?
