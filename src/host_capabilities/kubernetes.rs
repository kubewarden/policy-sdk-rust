use anyhow::{anyhow, Result};
use k8s_openapi::api::authorization::v1::{SubjectAccessReview, SubjectAccessReviewStatus};
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
        wapc_guest::host_call("kubewarden", "kubernetes", "list_resources_all", &msg)
            .map_err(|e| anyhow!("{}", e))?;

    serde_json::from_slice(&response_raw).map_err(|e| {
        anyhow!(
            "error deserializing list all resources response into Kubernetes resource: {:?}",
            e
        )
    })
}

/// Describe the set of parameters used by the `get_resource` function.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetResourceRequest {
    /// apiVersion of the resource (v1 for core group, groupName/groupVersions for other).
    pub api_version: String,
    /// Singular PascalCase name of the resource
    pub kind: String,
    /// The name of the resource
    pub name: String,
    /// The namespace used to search namespaced resources. Cluster level resources
    /// must set this parameter to `None`
    pub namespace: Option<String>,
    /// Disable caching of results obtained from Kubernetes API Server
    /// By default query results are cached for 5 seconds, that might cause
    /// stale data to be returned.
    /// However, making too many requests against the Kubernetes API Server
    /// might cause issues to the cluster
    pub disable_cache: bool,
}

/// Get a specific Kubernetes resource.
pub fn get_resource<T>(req: &GetResourceRequest) -> Result<T>
where
    T: serde::de::DeserializeOwned + Clone,
{
    let msg = serde_json::to_vec(req)
        .map_err(|e| anyhow!("error serializing the get resource request: {}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "kubernetes", "get_resource", &msg)
        .map_err(|e| anyhow!("{}", e))?;

    serde_json::from_slice(&response_raw).map_err(|e| {
        anyhow!(
            "error deserializing get resource response into Kubernetes resource: {:?}",
            e
        )
    })
}

/// Describe the set of parameters used by the `can_i` function.
#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectAccessReviewRequest {
    /// The SubjectAccessReview object to be sent to the Kubernetes API Server. This object's spec
    /// must be initialized with the details of the resource access being verified. The user
    /// specified in the spec must match the user being validated by the policy. For example, to
    /// validate a service account named my-user in the default namespace, the user field in the
    /// spec should be set to system:serviceaccount:default:my-user.
    /// This object is sent directly to the Kubernetes API Server. Therefore, its fields must
    /// follow to the official Kubernetes documentation.
    pub subject_access_review: SubjectAccessReview,
    /// Disable caching of results obtained from Kubernetes API Server
    /// By default query results are cached for 5 seconds, that might cause
    /// stale data to be returned.
    /// However, making too many requests against the Kubernetes API Server
    /// might cause issues to the cluster
    pub disable_cache: bool,
}
/// Check if user has permissions to perform an action on resources. This is done
/// by sending a SubjectAccessReview to the Kubernetes authorization API.
pub fn can_i(request: SubjectAccessReviewRequest) -> Result<SubjectAccessReviewStatus> {
    let msg = serde_json::to_vec(&request)
        .map_err(|e| anyhow!("error serializing the can_i request: {:?}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "kubernetes", "can_i", &msg)
        .map_err(|e| anyhow!("{}", e))?;

    serde_json::from_slice(&response_raw)
        .map_err(|e| anyhow!("error deserializing can_i response: {:?}", e))
}
