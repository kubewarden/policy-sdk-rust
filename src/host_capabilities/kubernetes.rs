use anyhow::{anyhow, Result};
use k8s_openapi::api::authorization::v1::{SubjectAccessReviewSpec, SubjectAccessReviewStatus};
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

impl From<SubjectAccessReview> for SubjectAccessReviewSpec {
    fn from(request: SubjectAccessReview) -> Self {
        SubjectAccessReviewSpec {
            user: Some(request.user),
            groups: request.groups,
            resource_attributes: Some(request.resource_attributes.into()),
            ..Default::default()
        }
    }
}

impl From<ResourceAttributes> for k8s_openapi::api::authorization::v1::ResourceAttributes {
    fn from(attrs: ResourceAttributes) -> Self {
        k8s_openapi::api::authorization::v1::ResourceAttributes {
            namespace: attrs.namespace,
            verb: Some(attrs.verb),
            group: attrs.group,
            resource: Some(attrs.resource),
            subresource: attrs.subresource,
            name: attrs.name,
            version: attrs.version,
            ..Default::default()
        }
    }
}

/// Describe the set of parameters used by the `can_i` function.
#[derive(Serialize, Deserialize, Debug)]
pub struct CanIRequest {
    /// The values in this struct will be used to build the SubjectAccessReview resources sent to the
    /// Kubernetes API to verify if the user is allowed to perform some operation
    pub subject_access_review: SubjectAccessReview,

    /// Disable caching of results obtained from Kubernetes API Server
    /// By default query results are cached for 5 seconds, that might cause
    /// stale data to be returned.
    /// However, making too many requests against the Kubernetes API Server
    /// might cause issues to the cluster
    pub disable_cache: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Hash, Clone)]
pub struct SubjectAccessReview {
    /// The groups you're testing for.
    pub groups: Option<Vec<String>>,

    /// Information for a resource access request
    pub resource_attributes: ResourceAttributes,

    /// User is the user you're testing for. If you specify "User" but not "Groups", then is it
    /// interpreted as "What if User were not a member of any groups
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Hash, Clone)]
pub struct ResourceAttributes {
    /// Group is the API Group of the Resource.  "*" means all.
    pub group: Option<String>,

    /// Name is the name of the resource being requested for a "get" or deleted for a "delete". ""
    /// (empty) means all.
    pub name: Option<String>,

    /// Namespace is the namespace of the action being requested.  Currently, there is no
    /// distinction between no namespace and all namespaces.
    /// - "" (empty) is empty for cluster-scoped resources
    /// - "" (empty) means "all" for namespace scoped resources
    pub namespace: Option<String>,

    /// Resource is one of the existing resource types.  "*" means all.
    pub resource: String,

    /// Subresource is one of the existing resource types.  "" means none.
    pub subresource: Option<String>,

    /// Verb is a kubernetes resource API verb, like: get, list, watch, create, update, delete,
    /// proxy.  "*" means all.
    pub verb: String,

    /// Version is the API Version of the Resource.  "*" means all.
    pub version: Option<std::string::String>,
}

/// Check if user has permissions to perform an action on resources. This is done
/// by sending a SubjectAccessReview to the Kubernetes authorization API.
pub fn can_i(request: CanIRequest) -> Result<SubjectAccessReviewStatus> {
    let msg = serde_json::to_vec(&request)
        .map_err(|e| anyhow!("error serializing the can_i request: {:?}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "kubernetes", "can_i", &msg)
        .map_err(|e| anyhow!("{}", e))?;

    serde_json::from_slice(&response_raw)
        .map_err(|e| anyhow!("error deserializing can_i response: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_access_review_spec_conversion() {
        let request = SubjectAccessReview {
            groups: Some(vec!["group1".to_owned(), "group2".to_owned()]),
            resource_attributes: ResourceAttributes {
                group: Some("apps".to_owned()),
                name: Some("my-deployment".to_owned()),
                namespace: Some("default".to_owned()),
                resource: "deployments".to_owned(),
                subresource: Some("scale".to_owned()),
                verb: "create".to_owned(),
                version: Some("v1".to_owned()),
            },
            user: "my-user".to_owned(),
        };

        assert_eq!(
            SubjectAccessReviewSpec::from(request),
            SubjectAccessReviewSpec {
                user: Some("my-user".to_owned()),
                groups: Some(vec!["group1".to_owned(), "group2".to_owned()]),
                resource_attributes: Some(
                    k8s_openapi::api::authorization::v1::ResourceAttributes {
                        group: Some("apps".to_owned()),
                        name: Some("my-deployment".to_owned()),
                        namespace: Some("default".to_owned()),
                        resource: Some("deployments".to_owned()),
                        subresource: Some("scale".to_owned()),
                        verb: Some("create".to_owned()),
                        version: Some("v1".to_owned()),
                        ..Default::default()
                    }
                ),
                ..Default::default()
            }
        );
    }
}
