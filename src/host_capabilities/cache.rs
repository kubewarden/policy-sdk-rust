use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Request to store a value in the cache
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheSetRequest {
    key: String,
    /// Lifespan of the entry, in seconds
    ttl: u64,
    value: Vec<u8>,
}

/// Request to retrieve a value from the cache
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheGetRequest {
    key: String,
}

/// Response to a cache set request
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheSetResponse {
    /// `0` on success, non-zero otherwise
    code: u32,
    message: String,
}

/// Response to a cache get request
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheGetResponse {
    /// `0` on success, non-zero otherwise
    code: u32,
    message: String,
    /// Stored value, or `None` on a cache miss
    #[serde(default)]
    value: Option<Vec<u8>>,
}

/// Store `value` under `key`, keeping it for `ttl` seconds.
///
/// Keys reserved for Kubewarden's internal caches (those starting with
/// `kubewarden_internal_`) are rejected by the host.
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

/// Retrieve the value stored under `key`. Returns `Ok(None)` on a cache miss.
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
