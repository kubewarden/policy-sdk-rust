use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::{Namespace, Service};
use k8s_openapi::api::networking::v1::Ingress;
use k8s_openapi::List;
use wapc_guest as guest;

pub trait Client {
    /// Get list of namespaces
    fn namespaces(&self) -> Result<Vec<u8>>;

    /// Get list of ingresses
    fn ingresses(&self) -> Result<Vec<u8>>;

    /// Get list of services
    fn services(&self) -> Result<Vec<u8>>;
}

pub struct WapcClient {}

impl Client for WapcClient {
    fn namespaces(&self) -> Result<Vec<u8>> {
        guest::host_call("kubernetes", "namespaces", "list", &Vec::new())
            .map_err(|e| anyhow!("{}", e))
    }

    fn ingresses(&self) -> Result<Vec<u8>> {
        guest::host_call("kubernetes", "ingresses", "list", &Vec::new())
            .map_err(|e| anyhow!("{}", e))
    }

    fn services(&self) -> Result<Vec<u8>> {
        guest::host_call("kubernetes", "services", "list", &Vec::new())
            .map_err(|e| anyhow!("{}", e))
    }
}

/// Fake client used when running unit tests. This should be used when writing
/// code that doesn't target wasm32
pub struct TestClient {}

impl Client for TestClient {
    fn namespaces(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&List::<Namespace>::default()).unwrap())
    }

    fn ingresses(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&List::<Ingress>::default()).unwrap())
    }

    fn services(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&List::<Service>::default()).unwrap())
    }
}
