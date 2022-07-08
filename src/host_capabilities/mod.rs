use crate::host_capabilities::verification::{
    KeylessGithubActionsInfo, KeylessInfo, KeylessPrefixInfo,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod net;
pub mod oci;
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
    /// to be signed by Sigstore, using public keys mode
    SigstorePubKeyVerify {
        /// String pointing to the object (e.g.: `registry.testing.lan/busybox:1.0.0`)
        image: String,
        /// List of PEM encoded keys that must have been used to sign the OCI object
        pub_keys: Vec<String>,
        /// Optional - Annotations that must have been provided by all signers when they signed the OCI artifact
        annotations: Option<HashMap<String, String>>,
    },

    // Require the verification of the manifest digest of an OCI object to be
    // signed by Sigstore, using keyless mode
    SigstoreKeylessVerify {
        /// String pointing to the object (e.g.: `registry.testing.lan/busybox:1.0.0`)
        image: String,
        /// List of keyless signatures that must be found
        keyless: Vec<KeylessInfo>,
        /// Optional - Annotations that must have been provided by all signers when they signed the OCI artifact
        annotations: Option<HashMap<String, String>>,
    },

    // Require the verification of the manifest digest of an OCI object to be
    // signed by Sigstore using keyless mode, where the passed subject is a URL
    // prefix of the subject to match
    SigstoreKeylessPrefixVerify {
        /// String pointing to the object (e.g.: `registry.testing.lan/busybox:1.0.0`)
        image: String,
        /// List of keyless signatures that must be found
        keyless_prefix: Vec<KeylessPrefixInfo>,
        /// Optional - Annotations that must have been provided by all signers when they signed the OCI artifact
        annotations: Option<HashMap<String, String>>,
    },

    // Require the verification of the manifest digest of an OCI object to be
    // signed by Sigstore using keyless mode and performed in GitHub Actions
    SigstoreGithubActionsVerify {
        /// String pointing to the object (e.g.: `registry.testing.lan/busybox:1.0.0`)
        image: String,
        /// List of keyless signatures performed in GitHub Actions that must be found
        github_actions: Vec<KeylessGithubActionsInfo>,
        /// Optional - Annotations that must have been provided by all signers when they signed the OCI artifact
        annotations: Option<HashMap<String, String>>,
    },

    /// Lookup the addresses for a given hostname via DNS
    DNSLookupHost { host: String },
}
