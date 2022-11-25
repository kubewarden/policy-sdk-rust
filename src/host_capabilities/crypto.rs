use crate::host_capabilities::crypto_v1::{
    CertificateVerificationRequest, CertificateVerificationResponse,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// A x509 certificate
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Certificate {
    /// Which encoding is used by the certificate
    pub encoding: CertificateEncoding,
    /// Actual certificate
    pub data: Vec<u8>,
}

/// The encoding of the certificate
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum CertificateEncoding {
    #[allow(missing_docs)]
    Der,
    #[allow(missing_docs)]
    Pem,
}

/// Verify_cert verifies cert against the passed cert_chain
pub fn verify_cert(
    cert: Certificate,
    cert_chain: Option<Vec<Certificate>>,
    not_after: Option<String>,
) -> Result<bool> {
    let req = CertificateVerificationRequest {
        cert,
        cert_chain,
        not_after,
    };
    let msg = serde_json::to_vec(&req).map_err(|e| {
        anyhow!(
            "error serializing the certificate verification request: {}",
            e
        )
    })?;
    let response_raw =
        wapc_guest::host_call("kubewarden", "crypto", "v1/is_certificate_trusted", &msg)
            .map_err(|e| anyhow!("{}", e))?;

    let response: CertificateVerificationResponse = serde_json::from_slice(&response_raw)?;

    Ok(response.trusted)
}
