/// This module contains a list of common types and functions that are used across the different
/// policy types.
use k8s_openapi::apimachinery::pkg::runtime::RawExtension;

#[derive(
    Clone, Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum PolicyMode {
    #[default]
    Protect,
    Monitor,
}

#[derive(
    Clone, Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, schemars::JsonSchema,
)]
pub enum FailurePolicy {
    #[default]
    /// "Fail" means that an error calling the webhook causes the admission to
    /// fail and the API request to be rejected.
    Fail,
    /// "Ignore" means that an error calling the webhook is ignored and the API
    /// request is allowed to continue.
    Ignore,
}

#[derive(
    Clone, Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, schemars::JsonSchema,
)]
pub enum MatchPolicy {
    #[default]
    ///     Equivalent: match a request if modifies a resource listed in rules, even via another API group or version.
    ///     For example, if deployments can be modified via apps/v1, apps/v1beta1, and extensions/v1beta1,
    ///     and "rules" only included apiGroups:["apps"], apiVersions:["v1"], resources: ["deployments"],
    ///     a request to apps/v1beta1 or extensions/v1beta1 would be converted to apps/v1 and sent to the webhook.
    Equivalent,
    /// Exact: match a request only if it exactly matches a specified rule.
    /// For example, if deployments can be modified via apps/v1, apps/v1beta1, and extensions/v1beta1,
    /// but "rules" only included apiGroups:["apps"], apiVersions:["v1"], resources: ["deployments"],
    /// a request to apps/v1beta1 or extensions/v1beta1 would not be sent to the webhook.
    Exact,
}

#[derive(
    Clone, Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, schemars::JsonSchema,
)]
pub enum SideEffects {
    #[default]
    None,
    NoneOnDryRun,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextAwareResource {
    pub api_version: String,
    pub kind: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, schemars::JsonSchema)]
pub struct BackgroundAudit(bool);

impl Default for BackgroundAudit {
    fn default() -> Self {
        BackgroundAudit(true)
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, schemars::JsonSchema)]
pub struct TimeoutSeconds(i32);

impl Default for TimeoutSeconds {
    fn default() -> Self {
        TimeoutSeconds(10)
    }
}

impl From<i32> for TimeoutSeconds {
    fn from(timeout: i32) -> Self {
        TimeoutSeconds(timeout)
    }
}

impl From<TimeoutSeconds> for i32 {
    fn from(timeout: TimeoutSeconds) -> Self {
        timeout.0
    }
}

impl From<&TimeoutSeconds> for i32 {
    fn from(timeout: &TimeoutSeconds) -> Self {
        timeout.0
    }
}

pub(crate) fn default_policy_server() -> String {
    "default".to_string()
}

pub(crate) fn default_settings() -> RawExtension {
    RawExtension(serde_json::json!({}))
}
