use anyhow::{anyhow, Result};
use k8s_openapi::api::authorization::v1::SubjectAccessReviewStatus;
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

/// Describe the set of parameters used by the `can_i` function. The values in
/// this struct will be used to build the SubjectAccessReview resources sent to
/// the Kubernetes API to verify if the user is allowed to perform some operation
#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectAccessReviewRequest {
    /// User under test. The user should follow the partner
    /// system:serviceaccount:<namespace>:<user>. For example, for the service account "my-user"
    /// from the "default", the user should be: system:serviceaccount:default:my-user.
    /// This is equivalent to the `user` field in the SubjectAccessReview.spec.
    pub user: String,
    /// Resource group under which the resource is defined. Equivalent of the
    /// `group` field in the SubjectAccessReview.spec.resourceAttributes
    pub group: String,
    /// Namespace where the operation under test is being performed. Equivalent of the
    /// `namespace` field in the SubjectAccessReview.spec.resourceAttributes
    pub namespace: String,
    /// Resource under which the operation is being performed. Equivalent of the `resource` field
    /// in the SubjectAccessReview.spec.resourceAttributes
    pub resource: String,
    /// Verb that is being tested. Equivalent of the `verb` field in the
    /// SubjectAccessReview.spec.resourceAttributes
    pub verb: String,
    /// Disable caching of results obtained from Kubernetes API Server
    /// By default query results are cached for 5 seconds, that might cause
    /// stale data to be returned.
    /// However, making too many requests against the Kubernetes API Server
    /// might cause issues to the cluster
    pub disable_cache: bool,
}
/// Check if user has permissions to perform an action on resources. This is done
/// by sending a SubjectAccessReview to the Kubernetes authorization API.
pub fn can_i(request: &SubjectAccessReviewRequest) -> Result<SubjectAccessReviewStatus> {
    let msg = serde_json::to_vec(&request)
        .map_err(|e| anyhow!("error serializing the can_i request: {:?}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "kubernetes", "can_i", &msg)
        .map_err(|e| anyhow!("{}", e))?;

    serde_json::from_slice(&response_raw)
        .map_err(|e| anyhow!("error deserializing can_i response: {:?}", e))
}
