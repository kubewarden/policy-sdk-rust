// Guest-side wrappers for the `cache` host capability (RFC 0024).
//
// These let a policy author store and retrieve arbitrary data in a
// policy-controlled cache, choosing the key, the value, and the TTL, instead of
// relying on the framework's fixed-TTL internal caches.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Payload of a `kubewarden.cache.set` request.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheSetRequest {
    /// The unique identifier for the data being stored.
    key: String,
    /// The arbitrary data to be stored in the cache.
    value: Vec<u8>,
    /// The lifespan of the data, in seconds.
    ttl: u64,
}

/// Payload of a `kubewarden.cache.get` request.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheGetRequest {
    /// The key of the data to retrieve.
    key: String,
}

/// Response of a `kubewarden.cache.set` operation.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheSetResponse {
    /// `0` on success, non-zero otherwise.
    code: u32,
    /// Human readable description of the outcome.
    message: String,
}

/// Response of a `kubewarden.cache.get` operation.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheGetResponse {
    /// `0` on success, non-zero otherwise.
    code: u32,
    /// Human readable description of the outcome.
    message: String,
    /// The data stored under the requested key. `None` on a cache miss.
    #[serde(default)]
    value: Option<Vec<u8>>,
}

/// Store `value` under `key`, keeping it for `ttl` seconds.
///
/// The key is chosen by the policy author. Keys reserved for Kubewarden's
/// internal caches (those starting with `kubewarden_internal_`) are rejected by
/// the host and surface here as an [`Error::Validation`].
pub fn set(key: &str, value: &[u8], ttl: u64) -> Result<()> {
    let req = CacheSetRequest {
        key: key.to_owned(),
        value: value.to_owned(),
        ttl,
    };
    let msg = serde_json::to_vec(&req).map_err(|e| Error::Serialization {
        context: "cache.set request".to_string(),
        source: e,
    })?;

    let response_raw =
        wapc_guest::host_call("kubewarden", "cache", "set", &msg).map_err(|e| Error::HostCall {
            operation: "cache.set".to_string(),
            source: e,
        })?;

    let response: CacheSetResponse =
        serde_json::from_slice(&response_raw).map_err(|e| Error::Deserialization {
            context: "cache.set response".to_string(),
            source: e,
        })?;

    if response.code != 0 {
        return Err(Error::Validation(format!(
            "cache.set failed: {}",
            response.message
        )));
    }

    Ok(())
}

/// Retrieve the value stored under `key`.
///
/// Returns `Ok(None)` on a cache miss; a miss is not an error.
pub fn get(key: &str) -> Result<Option<Vec<u8>>> {
    let req = CacheGetRequest {
        key: key.to_owned(),
    };
    let msg = serde_json::to_vec(&req).map_err(|e| Error::Serialization {
        context: "cache.get request".to_string(),
        source: e,
    })?;

    let response_raw =
        wapc_guest::host_call("kubewarden", "cache", "get", &msg).map_err(|e| Error::HostCall {
            operation: "cache.get".to_string(),
            source: e,
        })?;

    let response: CacheGetResponse =
        serde_json::from_slice(&response_raw).map_err(|e| Error::Deserialization {
            context: "cache.get response".to_string(),
            source: e,
        })?;

    if response.code != 0 {
        return Err(Error::Validation(format!(
            "cache.get failed: {}",
            response.message
        )));
    }

    Ok(response.value)
}
