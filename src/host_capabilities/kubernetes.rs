use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Describe the set of parameters used by the `list_resources_by_namespace`
/// function.
#[derive(Serialize, Deserialize, Debug)]
pub struct ListResourcesByNamespaceRequest {
    /// apiVersion of the resource (v1 for core group, groupName/groupVersions for other).
    pub api_version: String,
    /// Singular PascalCase name of the resource
    pub kind: String,
    /// Namespace scoping the search
    pub namespace: String,
    /// A selector to restrict the list of returned objects by their labels.
    /// Defaults to everything if `None`
    pub label_selector: Option<String>,
    /// A selector to restrict the list of returned objects by their fields.
    /// Defaults to everything if `None`
    pub field_selector: Option<String>,
}

/// Get all the Kubernetes resources defined inside of the given
/// namespace
/// Note: cannot be used for cluster-wide resources
pub fn list_resources_by_namespace<T>(
    req: &ListResourcesByNamespaceRequest,
) -> Result<k8s_openapi::List<T>>
where
    T: k8s_openapi::ListableResource + serde::de::DeserializeOwned + Clone,
{
    let msg = serde_json::to_vec(req).map_err(|e| {
        anyhow!(
            "error serializing the list resources by namespace request: {}",
            e
        )
    })?;
    let response_raw = wapc_guest::host_call(
        "kubewarden",
        "kubernetes",
        "list_resources_by_namespace",
        &msg,
    )
    .map_err(|e| anyhow!("{}", e))?;

    serde_json::from_slice(&response_raw).map_err(|e| {
        anyhow!(
            "error deserializing list resources by namespace response into Kubernetes resource: {:?}",
            e
        )
    })
}

/// Describe the set of parameters used by the `list_all_resources` function.
#[derive(Serialize, Deserialize, Debug)]
pub struct ListAllResourcesRequest {
    /// apiVersion of the resource (v1 for core group, groupName/groupVersions for other).
    pub api_version: String,
    /// Singular PascalCase name of the resource
    pub kind: String,
    /// A selector to restrict the list of returned objects by their labels.
    /// Defaults to everything if `None`
    pub label_selector: Option<String>,
    /// A selector to restrict the list of returned objects by their fields.
    /// Defaults to everything if `None`
    pub field_selector: Option<String>,
}

/// Get all the Kubernetes resources defined inside of the cluster.
/// Note: this has be used for cluster-wide resources
pub fn list_all_resources<T>(req: &ListAllResourcesRequest) -> Result<k8s_openapi::List<T>>
where
    T: k8s_openapi::ListableResource + serde::de::DeserializeOwned + Clone,
{
    let msg = serde_json::to_vec(req)
        .map_err(|e| anyhow!("error serializing the list all resources request: {}", e))?;
    let response_raw =
        wapc_guest::host_call("kubewarden", "kubernetes", "list_all_resources", &msg)
            .map_err(|e| anyhow!("{}", e))?;

    serde_json::from_slice(&response_raw).map_err(|e| {
        anyhow!(
            "error deserializing list all resources response into Kubernetes resource: {:?}",
            e
        )
    })
}
