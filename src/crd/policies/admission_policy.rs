/// This module contains all the definitions of all Kubewarden policy CRDs
/// that are used to define the policy groups.
use k8s_openapi::{
    api::admissionregistration::v1::{MatchCondition, RuleWithOperations},
    apimachinery::pkg::{apis::meta::v1::LabelSelector, runtime::RawExtension},
};

use crate::crd::policies::common::{
    default_policy_server, default_settings, BackgroundAudit, FailurePolicy, MatchPolicy,
    PolicyMode, SideEffects, TimeoutSeconds,
};

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    k8s_openapi_derive::CustomResourceDefinition,
    serde::Deserialize,
    serde::Serialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[custom_resource_definition(
    group = "policies.kubewarden.io",
    version = "v1",
    plural = "admissionpolicies",
    generate_schema,
    has_subresources = "v1"
)]
pub struct AdmissionPolicySpec {
    /// BackgroundAudit indicates whether a policy should be used or skipped when
    /// performing audit checks. If false, the policy cannot produce meaningful
    /// evaluation results during audit checks and will be skipped.
    /// The default is "true".
    pub background_audit: Option<BackgroundAudit>,

    /// FailurePolicy defines how unrecognized errors and timeout errors from the
    /// policy are handled.
    pub failure_policy: Option<FailurePolicy>,

    /// MatchConditions are a list of conditions that must be met for a request to be
    /// validated. Match conditions filter requests that have already been matched by
    /// the rules, namespaceSelector, and objectSelector. An empty list of
    /// matchConditions matches all requests. There are a maximum of 64 match
    /// conditions allowed. If a parameter object is provided, it can be accessed via
    /// the params handle in the same manner as validation expressions. The exact
    /// matching logic is (in order):
    ///  - If ANY matchCondition evaluates to FALSE, the policy is skipped
    ///  - If ALL matchConditions evaluate to TRUE, the policy is evaluated
    ///  - If any matchCondition evaluates to an error (but none are FALSE):
    ///     - If failurePolicy=Fail, reject the request
    ///     - If failurePolicy=Ignore, the policy is skipped.
    ///
    /// Only available if the feature gate AdmissionWebhookMatchConditions is enabled.
    pub match_conditions: Option<Vec<MatchCondition>>,

    /// matchPolicy defines how the "rules" list is used to match incoming requests.
    pub match_policy: Option<MatchPolicy>,

    /// Mode defines the execution mode of this policy. Can be set to
    /// either "protect" or "monitor". If it's empty, it is defaulted to
    /// "protect".
    /// Transitioning this setting from "monitor" to "protect" is
    /// allowed, but is disallowed to transition from "protect" to
    /// "monitor". To perform this transition, the policy should be
    /// recreated in "monitor" mode instead.
    pub mode: Option<PolicyMode>,

    /// Module is the location of the WASM module to be loaded. Can be a
    /// local file (file://), a remote file served by an HTTP server
    /// (http://, https://), or an artifact served by an OCI-compatible
    /// registry (registry://).
    /// If prefix is missing, it will default to registry:// and use that
    /// internally.
    pub module: String,

    /// Mutating indicates whether a policy has the ability to mutate
    /// incoming requests or not.
    #[serde(default)]
    pub mutating: bool,

    /// ObjectSelector decides whether to run the webhook based on if the
    /// object has matching labels. objectSelector is evaluated against both
    /// the oldObject and newObject that would be sent to the webhook, and
    /// is considered to match if either object matches the selector. A null
    /// object (oldObject in the case of create, or newObject in the case of
    /// delete) or an object that cannot have labels (like a
    /// DeploymentRollback or a PodProxyOptions object) is not considered to
    /// match.
    /// Use the object selector only if the webhook is opt-in, because end
    /// users may skip the admission webhook by setting the labels.
    /// Default to the empty LabelSelector, which matches everything.
    pub object_selector: Option<LabelSelector>,

    /// identifies an existing PolicyServer resource
    #[serde(default = "default_policy_server")]
    pub policy_server: String,

    /// Rules describes what operations on what resources/subresources the webhook cares about.
    /// The webhook cares about an operation if it matches any Rule.
    pub rules: Option<Vec<RuleWithOperations>>,

    /// Settings is a free-form object that contains the policy configuration
    /// values.
    #[serde(default = "default_settings")]
    pub settings: RawExtension,

    /// SideEffects states whether this webhook has side effects.
    /// Acceptable values are: None, NoneOnDryRun.
    /// Webhooks with side effects MUST implement a reconciliation system, since a request may be
    /// rejected by a future step in the admission change and the side effects therefore need to be undone.
    pub side_effects: Option<SideEffects>,

    /// TimeoutSeconds specifies the timeout for this webhook. After the timeout passes,
    /// the webhook call will be ignored or the API call will fail based on the
    /// failure policy.
    /// The timeout value must be between 1 and 30 seconds.
    /// Default to 10 seconds.
    pub timeout_seconds: Option<TimeoutSeconds>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const YAML_NO_DEFAULTS: &str = r#"
apiVersion: policies.kubewarden.io/v1
kind: AdmissionPolicy
metadata:
  name: psp-capabilities
  namespace: default
spec:
  policyServer: reserved-instance-for-tenant-a
  module: registry://ghcr.io/kubewarden/policies/psp-capabilities:v0.1.9
  rules:
    - apiGroups: [""]
      apiVersions: ["v1"]
      resources: ["pods"]
      operations:
        - CREATE
        - UPDATE
  mutating: true
  settings:
    allowed_capabilities:
      - CHOWN
    required_drop_capabilities:
      - NET_ADMIN
"#;

    const YAML_WITH_DEFAULTS: &str = r#"
apiVersion: policies.kubewarden.io/v1
kind: AdmissionPolicy
metadata:
  name: default-values
  namespace: default
spec:
  module: registry://ghcr.io/kubewarden/policies/foo:v1.0.0
  rules:
    - apiGroups: [""]
      apiVersions: ["v1"]
      resources: ["pods"]
      operations:
        - CREATE
        - UPDATE

"#;

    #[test]
    fn test_admission_policy_spec() {
        let policy: AdmissionPolicy =
            serde_yaml::from_str(YAML_NO_DEFAULTS).expect("cannot deserialize AdmissionPolicy");
        assert_eq!(
            policy.metadata.name.unwrap(),
            "psp-capabilities".to_string()
        );
        assert_eq!(policy.metadata.namespace.unwrap(), "default".to_string());

        let spec = policy.spec.expect("should have spec");

        assert_eq!(
            spec.policy_server,
            "reserved-instance-for-tenant-a".to_string()
        );
        assert_eq!(
            spec.settings.0,
            serde_json::json!({
                "allowed_capabilities": ["CHOWN"],
                "required_drop_capabilities": ["NET_ADMIN"]
            })
        );
        assert!(spec.mutating);
    }

    #[test]
    fn test_admission_policy_spec_defaults() {
        let policy: AdmissionPolicy =
            serde_yaml::from_str(YAML_WITH_DEFAULTS).expect("cannot deserialize AdmissionPolicy");
        assert_eq!(policy.metadata.name.unwrap(), "default-values".to_string());
        assert_eq!(policy.metadata.namespace.unwrap(), "default".to_string());

        let spec = policy.spec.expect("should have spec");

        assert_eq!(spec.policy_server, "default".to_string());
        assert_eq!(spec.settings.0, serde_json::json!({}));
        assert!(!spec.mutating);
    }

    #[test]
    // make sure serde fails with an error
    fn test_admission_policy_spec_does_not_have_ctx_aware() {
        let yaml = r#"
apiVersion: policies.kubewarden.io/v1
kind: AdmissionPolicy
metadata:
  name: default-values
  namespace: default
spec:
  policyServer: reserved-instance-for-tenant-a
  module: registry://ghcr.io/kubewarden/policies/psp-capabilities:v0.1.9
  rules:
    - apiGroups: [""]
      apiVersions: ["v1"]
      resources: ["pods"]
      operations:
        - CREATE
        - UPDATE
  mutating: true
  settings:
    allowed_capabilities:
      - CHOWN
    required_drop_capabilities:
      - NET_ADMIN
  contextAwareResources:
    - apiVersion: "apps/v1"
      kind: "deployment"
    - apiVersion: "v1"
      kind: "pod"
  namespaceSelector:
    matchExpressions:
      - key: environment
        operator: In
        values:
        - prod
        - staging
"#;

        let err = serde_yaml::from_str::<AdmissionPolicy>(yaml).unwrap_err();
        assert!(err
            .to_string()
            .contains("unknown field `contextAwareResources`"));
    }
}
