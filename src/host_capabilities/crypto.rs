use std::fmt;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::host_capabilities::crypto_v1::{
    CertificateVerificationRequest, CertificateVerificationResponse,
};

/// A x509 certificate
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Certificate {
    /// Which encoding is used by the certificate
    pub encoding: CertificateEncoding,
    /// Actual certificate
    pub data: Vec<u8>,
}

impl fmt::Display for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoding = match self.encoding {
            CertificateEncoding::Der => "DER",
            CertificateEncoding::Pem => "PEM",
        };

        let human_data = match self.encoding {
            CertificateEncoding::Pem => String::from_utf8_lossy(&self.data).to_string(),
            CertificateEncoding::Der => format!("(HEX) {:?}", hex::encode_upper(&self.data)),
        };

        write!(
            f,
            "Certificate(encoding: {}, data: {:?})",
            encoding, human_data
        )
    }
}

/// The encoding of the certificate
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum CertificateEncoding {
    #[allow(missing_docs)]
    // Be explicit about how the name should be handled
    // see https://github.com/kubewarden/policy-sdk-rust/issues/105
    #[serde(rename = "Der")]
    Der,

    #[allow(missing_docs)]
    // Be explicit about how the name should be handled
    // see https://github.com/kubewarden/policy-sdk-rust/issues/105
    #[serde(rename = "Pem")]
    Pem,
}

/// Used as return of verify_cert()
#[derive(Debug, Serialize)]
pub enum BoolWithReason {
    True,
    False(String),
}

impl From<BoolWithReason> for CertificateVerificationResponse {
    fn from(b: BoolWithReason) -> CertificateVerificationResponse {
        match b {
            BoolWithReason::True => CertificateVerificationResponse {
                trusted: true,
                reason: "".to_string(),
            },
            BoolWithReason::False(reason) => CertificateVerificationResponse {
                trusted: false,
                reason,
            },
        }
    }
}

/// Verify_cert verifies cert's trust against the passed cert_chain, and
/// expiration and validation time of the certificate.
/// Accepts 3 arguments:
/// * cert: PEM-encoded certificate to verify.
/// * cert_chain: list of PEM-encoded certs, ordered by trust usage
///   (intermediates first, root last). If empty, the Mozilla's CA is used.
/// * not_after: string in RFC 3339 time format, to check expiration against.
///   If None, certificate is assumed never expired.
pub fn verify_cert(
    cert: Certificate,
    cert_chain: Option<Vec<Certificate>>,
    not_after: Option<String>,
) -> Result<BoolWithReason> {
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
    match response.trusted {
        true => Ok(BoolWithReason::True),
        false => Ok(BoolWithReason::False(format!(
            "Certificate not trusted: {}",
            response.reason
        ))),
    }
}
