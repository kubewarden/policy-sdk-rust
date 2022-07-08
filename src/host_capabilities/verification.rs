use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(test)]
use tests::mock_wapc as wapc_guest;

use crate::host_capabilities::CallbackRequestType;

/// VerificationResponse holds the response of a sigstore signatures verification
#[derive(Serialize, Deserialize, Clone)]
pub struct VerificationResponse {
    /// true if the image is trusted, which means verification was successfull
    pub is_trusted: bool,
    /// digest of the image that was verified
    pub digest: String,
}

/// KeylessInfo holds information about a keyless signature
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeylessInfo {
    /// the issuer identifier
    pub issuer: String,
    /// contains the information of the user used to authenticate against the OIDC provider
    pub subject: String,
}

/// KeylessPrefixInfo holds information about a keyless signature
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeylessPrefixInfo {
    /// the issuer identifier
    pub issuer: String,
    /// Valid prefix of the Subject field in the signature used to authenticate
    /// against the OIDC provider. It forms a valid URL on its own, and will get
    /// sanitized by appending `/` to protect against typosquatting
    pub url_prefix: String,
}

/// KeylessGithubActionsInfo holds information about a keyless signature
/// performed in GitHub Actions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeylessGithubActionsInfo {
    /// owner of the repository. E.g: octocat
    owner: String,
    /// Optional - repo of the GH Action workflow that signed the artifact. E.g: example-repo
    repo: Option<String>,
}

/// verify sigstore signatures of an image using public keys
/// # Arguments
/// * `image` -  image to be verified
/// * `pub_keys`  -  list of PEM encoded keys that must have been used to sign the OCI object
/// * `annotations` - annotations that must have been provided by all signers when they signed the OCI artifact
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

/// verify sigstore signatures of an image using keyless
/// # Arguments
/// * `image` -  image to be verified
/// * `keyless`  -  list of issuers and subjects
/// * `annotations` - annotations that must have been provided by all signers when they signed the OCI artifact
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

/// verify sigstore signatures of an image using keyless. Here, the provided
/// subject string is treated as a URL prefix, and sanitized to a valid URL on
/// itself by appending `/` to prevent typosquatting. Then, the provided subject
/// will satisfy the signature only if it is a prefix of the signature subject.
/// # Arguments
/// * `image` -  image to be verified
/// * `keyless`  -  list of issuers and subjects
/// * `annotations` - annotations that must have been provided by all signers when they signed the OCI artifact
pub fn verify_keyless_prefix_match(
    image: &str,
    keyless_prefix: Vec<KeylessPrefixInfo>,
    annotations: Option<HashMap<String, String>>,
) -> Result<VerificationResponse> {
    let req = CallbackRequestType::SigstoreKeylessPrefixVerify {
        image: image.to_string(),
        keyless_prefix,
        annotations,
    };

    verify(req)
}

/// verify sigstore signatures of an image using keyless signatures made via
/// Github Actions.
/// # Arguments
/// * `image` -  image to be verified
/// * `github_actions` - list of GitHub owners and repos
/// * `annotations` - annotations that must have been provided by all signers when they signed the OCI artifact
pub fn verify_keyless_github_actions(
    image: &str,
    github_actions: Vec<KeylessGithubActionsInfo>,
    annotations: Option<HashMap<String, String>>,
) -> Result<VerificationResponse> {
    let req = CallbackRequestType::SigstoreGithubActionsVerify {
        image: image.to_string(),
        github_actions,
        annotations,
    };

    verify(req)
}

fn verify(req: CallbackRequestType) -> Result<VerificationResponse> {
    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "v1/verify", &msg)
        .map_err(|e| anyhow!("{}", e))?;

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

    #[serial]
    #[test]
    fn verify_keyless_prefix_trusted() {
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
        let res = verify_keyless_prefix_match(
            "image",
            vec![KeylessPrefixInfo {
                url_prefix: "urlprefix".to_string(),
                issuer: "issuer".to_string(),
            }],
            None,
        );

        assert_eq!(res.unwrap().is_trusted, true)
    }

    #[serial]
    #[test]
    fn verify_keyless_prefix_not_trusted() {
        let ctx = mock_wapc::host_call_context();
        ctx.expect()
            .times(1)
            .returning(|_, _, _, _| Err(Box::new(core::fmt::Error {})));
        let res = verify_keyless_prefix_match(
            "image",
            vec![KeylessPrefixInfo {
                url_prefix: "urlprefix".to_string(),
                issuer: "issuer".to_string(),
            }],
            None,
        );

        assert!(res.is_err())
    }

    #[serial]
    #[test]
    fn verify_keyless_github_actions_trusted() {
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
        let res = verify_keyless_github_actions(
            "image",
            vec![KeylessGithubActionsInfo {
                owner: "owner".to_string(),
                repo: Some("repo".to_string()),
            }],
            None,
        );

        assert_eq!(res.unwrap().is_trusted, true)
    }

    #[serial]
    #[test]
    fn verify_keyless_github_actions_not_trusted() {
        let ctx = mock_wapc::host_call_context();
        ctx.expect()
            .times(1)
            .returning(|_, _, _, _| Err(Box::new(core::fmt::Error {})));
        let res = verify_keyless_github_actions(
            "image",
            vec![KeylessGithubActionsInfo {
                owner: "owner".to_string(),
                repo: Some("repo".to_string()),
            }],
            None,
        );

        assert!(res.is_err())
    }
}
