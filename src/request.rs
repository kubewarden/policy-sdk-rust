use anyhow::anyhow;
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::{HashMap, HashSet};

/// ValidationRequest holds the data provided to the policy at evaluation time
#[derive(Deserialize, Debug, Clone)]
pub struct ValidationRequest<T: Default> {
    /// The policy settings
    pub settings: T,

    /// Kubernetes' [AdmissionReview](https://kubernetes.io/docs/reference/access-authn-authz/extensible-admission-controllers/) request
    pub request: KubernetesAdmissionRequest,
}

/// Kubernetes' [AdmissionReview](https://kubernetes.io/docs/reference/access-authn-authz/extensible-admission-controllers/)
/// request.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct KubernetesAdmissionRequest {
    /// UID is an identifier for the individual request/response. It allows us to distinguish instances of requests which are
    /// otherwise identical (parallel requests, requests when earlier requests did not modify etc)
    /// The UID is meant to track the round trip (request/response) between the KAS and the WebHook, not the user request.
    /// It is suitable for correlating log entries between the webhook and apiserver, for either auditing or debugging.
    pub uid: String,

    /// Kind is the fully-qualified type of object being submitted (for example, v1.Pod or autoscaling.v1.Scale)
    pub kind: GroupVersionKind,

    /// Resource is the fully-qualified resource being requested (for example, v1.pods)
    pub resource: GroupVersionResource,

    /// SubResource is the subresource being requested, if any (for example, "status" or "scale")
    #[serde(alias = "subResource")]
    pub sub_resource: String,

    /// RequestKind is the fully-qualified type of the original API request (for example, v1.Pod or autoscaling.v1.Scale).
    /// If this is specified and differs from the value in "kind", an equivalent match and conversion was performed.
    ///
    /// For example, if deployments can be modified via apps/v1 and apps/v1beta1, and a webhook registered a rule of
    /// `apiGroups:["apps"], apiVersions:["v1"], resources: ["deployments"]` and `matchPolicy: Equivalent`,
    /// an API request to apps/v1beta1 deployments would be converted and sent to the webhook
    /// with `kind: {group:"apps", version:"v1", kind:"Deployment"}` (matching the rule the webhook registered for),
    /// and `requestKind: {group:"apps", version:"v1beta1", kind:"Deployment"}` (indicating the kind of the original API request).
    ///
    /// See documentation for the "matchPolicy" field in the webhook configuration type for more details.
    #[serde(alias = "requestKind")]
    pub request_kind: GroupVersionKind,

    /// RequestResource is the fully-qualified resource of the original API request (for example, v1.pods).
    /// If this is specified and differs from the value in "resource", an equivalent match and conversion was performed.
    ///
    /// For example, if deployments can be modified via apps/v1 and apps/v1beta1, and a webhook registered a rule of
    /// `apiGroups:["apps"], apiVersions:["v1"], resources: ["deployments"]` and `matchPolicy: Equivalent`,
    /// an API request to apps/v1beta1 deployments would be converted and sent to the webhook
    /// with `resource: {group:"apps", version:"v1", resource:"deployments"}` (matching the resource the webhook registered for),
    /// and `requestResource: {group:"apps", version:"v1beta1", resource:"deployments"}` (indicating the resource of the original API request).
    ///
    /// See documentation for the "matchPolicy" field in the webhook configuration type.
    #[serde(alias = "requestResource")]
    pub request_resource: GroupVersionKind,

    /// RequestSubResource is the name of the subresource of the original API request, if any (for example, "status" or "scale")
    /// If this is specified and differs from the value in "subResource", an equivalent match and conversion was performed.
    /// See documentation for the "matchPolicy" field in the webhook configuration type.
    #[serde(alias = "requestSubResource")]
    pub request_sub_resource: String,

    /// Name is the name of the object as presented in the request.  On a CREATE operation, the client may omit name and
    /// rely on the server to generate the name.  If that is the case, this field will contain an empty string.
    pub name: String,

    /// Namespace is the namespace associated with the request (if any).
    pub namespace: String,

    /// Operation is the operation being performed. This may be different than the operation
    /// requested. e.g. a patch can result in either a CREATE or UPDATE Operation.
    pub operation: String,

    /// UserInfo is information about the requesting user
    #[serde(alias = "userInfo")]
    pub user_info: UserInfo,

    /// Object is the object from the incoming request.
    pub object: serde_json::Value,

    /// OldObject is the existing object. Only populated for DELETE and UPDATE requests.
    #[serde(alias = "oldObject")]
    pub old_object: serde_json::Value,

    /// DryRun indicates that modifications will definitely not be persisted for this request.
    /// Defaults to false.
    #[serde(alias = "dryRun", default)]
    pub dry_run: bool,

    /// Options is the operation option structure of the operation being performed.
    /// e.g. `meta.k8s.io/v1.DeleteOptions` or `meta.k8s.io/v1.CreateOptions`. This may be
    /// different than the options the caller provided. e.g. for a patch request the performed
    /// Operation might be a CREATE, in which case the Options will a
    /// `meta.k8s.io/v1.CreateOptions` even though the caller provided `meta.k8s.io/v1.PatchOptions`.
    pub options: HashMap<String, serde_json::Value>,
}

/// GroupVersionKind unambiguously identifies a kind
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GroupVersionKind {
    pub group: String,
    pub version: String,
    pub kind: String,
}

/// GroupVersionResource unambiguously identifies a resource
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GroupVersionResource {
    pub group: String,
    pub version: String,
    pub kind: String,
}

/// UserInfo holds information about the user who made the request
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct UserInfo {
    /// The name that uniquely identifies this user among all active users.
    pub username: String,

    /// A unique value that identifies this user across time. If this user is
    /// deleted and another user by the same name is added, they will have
    /// different UIDs.
    pub uid: String,

    /// The names of groups this user is a part of.
    pub groups: HashSet<String>,

    /// Any additional information provided by the authenticator.
    pub extra: HashMap<String, serde_json::Value>,
}

impl<T> ValidationRequest<T>
where
    T: Default + DeserializeOwned,
{
    /// Crates a new `ValidationRequest` starting from the payload provided
    /// to the policy at invocation time.
    pub fn new(payload: &[u8]) -> anyhow::Result<Self> {
        serde_json::from_slice::<ValidationRequest<T>>(payload).map_err(|e| {
            anyhow!(
                "Error decoding validation payload {}: {:?}",
                String::from_utf8_lossy(payload),
                e
            )
        })
    }
}
