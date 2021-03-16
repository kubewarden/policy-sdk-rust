use anyhow::anyhow;
use jmespatch::{Expression, JmespathError, Rcvar, Variable};
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::{HashMap, HashSet};

/// ValidationRequest holds the data privided to the policy at evaluation time
#[derive(Deserialize, Debug, Clone)]
pub struct ValidationRequest<T> {
    /// The policy settings
    pub settings: T,

    /// Kubernetes' [AdmissionReview](https://kubernetes.io/docs/reference/access-authn-authz/extensible-admission-controllers/) request
    pub request: KubernetesAdmissionRequest,
}

/// Kubernetes' [AdmissionReview](https://kubernetes.io/docs/reference/access-authn-authz/extensible-admission-controllers/)
/// request.
///
/// Not all its fields are exposed as native attributes. These fields can
/// be found inside of the `extra_fields` attribute.
#[derive(Deserialize, Debug, Clone)]
pub struct KubernetesAdmissionRequest {
    /// Name of the resource
    pub name: String,

    /// Namespace of the resource
    pub namespace: String,

    /// Details about the user who made the request
    #[serde(alias = "userInfo")]
    pub user_info: UserInfo,

    /// Object describes the resources to be evaluated
    pub object: serde_json::Value,

    /// extra_fields contains all the fields of the AdmissionReview
    /// that are not exposed through native attributes
    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

/// UserInfo holds information about the user who made the request
#[derive(Deserialize, Debug, Clone)]
pub struct UserInfo {
    /// Name of the user
    pub username: String,

    /// List of groups the user belongs to
    pub groups: HashSet<String>,
}

}

impl<T> ValidationRequest<T>
where
    T: DeserializeOwned,
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

    /// Uses a [JMESPath](https://jmespath.org/) query against the `object`
    /// attribute of the `ValidationRequest`
    pub fn search(&self, expr: Expression) -> Result<Rcvar, JmespathError> {
        let data = Variable::from_serializable::<serde_json::Value>(self.request.object.clone())?;
        expr.search(data)
    }
}
