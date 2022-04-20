use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(test)]
use tests::mock_wapc as wapc_guest;

use crate::host_capabilities::CallbackRequestType;

#[derive(Serialize, Deserialize, Clone)]
pub struct VerificationResponse {
    pub is_trusted: bool,
    pub digest: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeylessInfo {
    pub issuer: String,
    pub subject: String,
}

pub fn verify_pub_keys_image(
    image: &str,
    pub_keys: Vec<String>,
    annotations: Option<HashMap<String, String>>,
) -> Result<VerificationResponse> {
    let req = CallbackRequestType::SigstorePubKeyVerify {
        image: image.to_string(),
        pub_keys,
        annotations,
    };

    verify(req)
}

pub fn verify_keyless_exact_match(
    image: &str,
    keyless: Vec<KeylessInfo>,
    annotations: Option<HashMap<String, String>>,
) -> Result<VerificationResponse> {
    let req = CallbackRequestType::SigstoreKeylessVerify {
        image: image.to_string(),
        keyless,
        annotations,
    };

    verify(req)
}

fn verify(req: CallbackRequestType) -> Result<VerificationResponse> {
    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "v1/verify", &msg)
        .map_err(|e| anyhow::anyhow!("error invoking wapc verify: {:?}", e))?;

    let response: VerificationResponse = serde_json::from_slice(&response_raw)?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::automock;
    use serial_test::serial;

    #[automock()]
    pub mod wapc {
        use wapc_guest::CallResult;

        // needed for creating mocks
        #[allow(dead_code)]
        pub fn host_call(_binding: &str, _ns: &str, _op: &str, _msg: &[u8]) -> CallResult {
            Ok(vec![u8::from(true)])
        }
    }

    // these tests need to run sequentially because mockall creates a global context to create the mocks
    #[serial]
    #[test]
    fn verify_pub_keys_trusted() {
        let ctx = mock_wapc::host_call_context();
        ctx.expect().times(1).returning(|_, _, _, _| {
            Ok(serde_json::to_vec(&{
                VerificationResponse {
                    is_trusted: true,
                    digest: "digest".to_string(),
                }
            })
            .unwrap())
        });
        let res = verify_pub_keys_image("image", vec!["key".to_string()], None);

        assert_eq!(res.unwrap().is_trusted, true)
    }

    #[serial]
    #[test]
    fn verify_pub_keys_not_trusted() {
        let ctx = mock_wapc::host_call_context();
        ctx.expect()
            .times(1)
            .returning(|_, _, _, _| Err(Box::new(core::fmt::Error {})));
        let res = verify_pub_keys_image("image", vec!["key".to_string()], None);

        assert!(res.is_err())
    }

    #[serial]
    #[test]
    fn verify_keyless_trusted() {
        let ctx = mock_wapc::host_call_context();
        ctx.expect().times(1).returning(|_, _, _, _| {
            Ok(serde_json::to_vec(&{
                VerificationResponse {
                    is_trusted: true,
                    digest: "digest".to_string(),
                }
            })
            .unwrap())
        });
        let res = verify_keyless_exact_match(
            "image",
            vec![KeylessInfo {
                subject: "subject".to_string(),
                issuer: "issuer".to_string(),
            }],
            None,
        );

        assert_eq!(res.unwrap().is_trusted, true)
    }

    #[serial]
    #[test]
    fn verify_keyless_not_trusted() {
        let ctx = mock_wapc::host_call_context();
        ctx.expect()
            .times(1)
            .returning(|_, _, _, _| Err(Box::new(core::fmt::Error {})));
        let res = verify_keyless_exact_match(
            "image",
            vec![KeylessInfo {
                subject: "subject".to_string(),
                issuer: "issuer".to_string(),
            }],
            None,
        );

        assert!(res.is_err())
    }
}
