use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod verification;

/// Describes the different kinds of request a waPC guest can make to
/// our host.
#[derive(Serialize, Deserialize, Debug)]
pub enum CallbackRequestType {
    /// Require the computation of the manifest digest of an OCI object (be
    /// it an image or anything else that can be stored into an OCI registry)
    OciManifestDigest {
        /// String pointing to the object (e.g.: `registry.testing.lan/busybox:1.0.0`)
        image: String,
    },
    /// Require the verification of the manifest digest of an OCI object (be
    /// it an image or anything else that can be stored into an OCI registry)
    /// to be signed by Sigstore
    SigstorePubKeyVerify {
        /// String pointing to the object (e.g.: `registry.testing.lan/busybox:1.0.0`)
        image: String,
        /// The configuration to use at verification time
        pub_keys: Vec<String>,
        annotations: HashMap<String, String>,
    },
}
