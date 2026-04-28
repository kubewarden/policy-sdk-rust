#![cfg_attr(docsrs, feature(doc_cfg))]

use std::collections::HashMap;

pub use wapc_guest;

pub mod error;
pub mod host_capabilities;
pub mod logging;
pub mod metadata;
#[cfg(not(target_arch = "wasm32"))]
mod non_wasm;
pub mod request;
pub mod response;
pub mod settings;
pub mod test;

pub use crate::error::{Error, Result};
use crate::metadata::ProtocolVersion;
#[cfg(feature = "cluster-context")]
use crate::request::{ValidationRequest, is_resource, supported_pod_spec_objects_message};
use crate::response::*;

#[cfg_attr(docsrs, doc(cfg(feature = "crd")))]
#[cfg(feature = "crd")]
/// Rust types of Kubewarden CRDs
pub mod crd;

cfg_if::cfg_if! {
    if #[cfg(feature = "cluster-context")] {
        use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
        use k8s_openapi::api::batch::v1::{CronJob, Job};
        use k8s_openapi::api::core::v1::{Pod, PodSpec, ReplicationController};
    }
}

/// Create an acceptance response
pub fn accept_request() -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ValidationResponse {
        accepted: true,
        message: None,
        code: None,
        mutated_object: None,
        audit_annotations: None,
        warnings: None,
    })?)
}

/// Create an acceptance response that mutates the original object
/// # Arguments
/// * `mutated_object` - the mutated Object
pub fn mutate_request(mutated_object: serde_json::Value) -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ValidationResponse {
        accepted: true,
        message: None,
        code: None,
        mutated_object: Some(mutated_object),
        audit_annotations: None,
        warnings: None,
    })?)
}

#[cfg_attr(docsrs, doc(cfg(feature = "cluster-context")))]
#[cfg(feature = "cluster-context")]
/// Update the pod spec from the resource defined in the original object
/// and create an acceptance response.
/// # Arguments
/// * `validation_request` - the original admission request
/// * `pod_spec` - new PodSpec to be set in the response
pub fn mutate_pod_spec_from_request<T: std::default::Default>(
    validation_request: ValidationRequest<T>,
    pod_spec: PodSpec,
) -> wapc_guest::CallResult {
    let gvk = &validation_request.request.kind;
    match () {
        _ if is_resource::<Deployment>(gvk) => {
            let mut deployment =
                serde_json::from_value::<Deployment>(validation_request.request.object.clone())?;
            let mut deployment_spec = deployment.spec.unwrap_or_default();
            deployment_spec.template.spec = Some(pod_spec);
            deployment.spec = Some(deployment_spec);
            mutate_request(serde_json::to_value(deployment)?)
        }
        _ if is_resource::<ReplicaSet>(gvk) => {
            let mut replicaset =
                serde_json::from_value::<ReplicaSet>(validation_request.request.object.clone())?;
            let mut replicaset_spec = replicaset.spec.unwrap_or_default();
            let mut template = replicaset_spec.template.unwrap_or_default();
            template.spec = Some(pod_spec);
            replicaset_spec.template = Some(template);
            replicaset.spec = Some(replicaset_spec);
            mutate_request(serde_json::to_value(replicaset)?)
        }
        _ if is_resource::<StatefulSet>(gvk) => {
            let mut statefulset =
                serde_json::from_value::<StatefulSet>(validation_request.request.object.clone())?;
            let mut statefulset_spec = statefulset.spec.unwrap_or_default();
            statefulset_spec.template.spec = Some(pod_spec);
            statefulset.spec = Some(statefulset_spec);
            mutate_request(serde_json::to_value(statefulset)?)
        }
        _ if is_resource::<DaemonSet>(gvk) => {
            let mut daemonset =
                serde_json::from_value::<DaemonSet>(validation_request.request.object.clone())?;
            let mut daemonset_spec = daemonset.spec.unwrap_or_default();
            daemonset_spec.template.spec = Some(pod_spec);
            daemonset.spec = Some(daemonset_spec);
            mutate_request(serde_json::to_value(daemonset)?)
        }
        _ if is_resource::<ReplicationController>(gvk) => {
            let mut replication_controller = serde_json::from_value::<ReplicationController>(
                validation_request.request.object.clone(),
            )?;
            let mut replication_controller_spec = replication_controller.spec.unwrap_or_default();
            let mut template = replication_controller_spec.template.unwrap_or_default();
            template.spec = Some(pod_spec);
            replication_controller_spec.template = Some(template);
            replication_controller.spec = Some(replication_controller_spec);
            mutate_request(serde_json::to_value(replication_controller)?)
        }
        _ if is_resource::<CronJob>(gvk) => {
            let mut cronjob =
                serde_json::from_value::<CronJob>(validation_request.request.object.clone())?;
            let mut cronjob_spec = cronjob.spec.unwrap_or_default();
            let mut job_template_spec = cronjob_spec.job_template;
            let mut job_spec = job_template_spec.spec.unwrap_or_default();
            let mut pod_template_spec = job_spec.template;
            pod_template_spec.spec = Some(pod_spec);
            job_spec.template = pod_template_spec;
            job_template_spec.spec = Some(job_spec);
            cronjob_spec.job_template = job_template_spec;
            cronjob.spec = Some(cronjob_spec);
            mutate_request(serde_json::to_value(cronjob)?)
        }
        _ if is_resource::<Job>(gvk) => {
            let mut job = serde_json::from_value::<Job>(validation_request.request.object.clone())?;
            let mut job_spec = job.spec.unwrap_or_default();
            job_spec.template.spec = Some(pod_spec);
            job.spec = Some(job_spec);
            mutate_request(serde_json::to_value(job)?)
        }
        _ if is_resource::<Pod>(gvk) => {
            let mut pod = serde_json::from_value::<Pod>(validation_request.request.object.clone())?;
            pod.spec = Some(pod_spec);
            mutate_request(serde_json::to_value(pod)?)
        }
        _ => reject_request(Some(supported_pod_spec_objects_message()), None, None, None),
    }
}

/// Create a rejection response
/// # Arguments
/// * `message` -  message shown to the user
/// * `code` -  code shown to the user
/// * `audit_annotations` - an unstructured key value map set by remote admission controller (e.g. error=image-blacklisted). MutatingAdmissionWebhook and ValidatingAdmissionWebhook admission controller will prefix the keys with admission webhook name (e.g. imagepolicy.example.com/error=image-blacklisted). AuditAnnotations will be provided by the admission webhook to add additional context to the audit log for this request.
/// * `warnings` -  a list of warning messages to return to the requesting API client. Warning messages describe a problem the client making the API request should correct or be aware of. Limit warnings to 120 characters if possible. Warnings over 256 characters and large numbers of warnings may be truncated.
pub fn reject_request(
    message: Option<String>,
    code: Option<u16>,
    audit_annotations: Option<HashMap<String, String>>,
    warnings: Option<Vec<String>>,
) -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ValidationResponse {
        accepted: false,
        mutated_object: None,
        message,
        code,
        audit_annotations,
        warnings,
    })?)
}

/// waPC guest function to register under the name `validate_settings`
/// # Example
///
/// ```
/// use kubewarden_policy_sdk::{validate_settings, settings::Validatable};
/// use serde::Deserialize;
/// use wapc_guest::register_function;
///
/// // This module settings require either `setting_a` or `setting_b`
/// // set. Both cannot be provided at the same time, and one has to be
/// // provided.
/// #[derive(Deserialize)]
/// struct Settings {
///   setting_a: Option<String>,
///   setting_b: Option<String>
/// }
///
/// impl Validatable for Settings {
///   fn validate(&self) -> Result<(), String> {
///     if self.setting_a.is_none() && self.setting_b.is_none() {
///       Err("either setting A or setting B has to be provided".to_string())
///     } else if self.setting_a.is_some() && self.setting_b.is_some() {
///       Err("setting A and setting B cannot be set at the same time".to_string())
///     } else {
///       Ok(())
///     }
///   }
/// }
///
/// register_function("validate_settings", validate_settings::<Settings>);
/// ```
pub fn validate_settings<T>(payload: &[u8]) -> wapc_guest::CallResult
where
    T: serde::de::DeserializeOwned + settings::Validatable,
{
    let settings: T = serde_json::from_slice::<T>(payload).map_err(|e| Error::Deserialization {
        context: format!("validation payload {}", String::from_utf8_lossy(payload)),
        source: e,
    })?;

    let res = match settings.validate() {
        Ok(_) => settings::SettingsValidationResponse {
            valid: true,
            message: None,
        },
        Err(e) => settings::SettingsValidationResponse {
            valid: false,
            message: Some(e),
        },
    };

    Ok(serde_json::to_vec(&res)?)
}

/// Helper function that provides the `protocol_version` implementation
/// # Example
///
/// ```
/// extern crate wapc_guest as guest;
/// use guest::prelude::*;
/// use kubewarden_policy_sdk::protocol_version_guest;
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn wapc_init() {
///     register_function("protocol_version", protocol_version_guest);
///     // register other waPC functions
/// }
/// ```
pub fn protocol_version_guest(_payload: &[u8]) -> wapc_guest::CallResult {
    Ok(serde_json::to_vec(&ProtocolVersion::default())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    cfg_if::cfg_if! {
        if #[cfg(feature = "cluster-context")] {
            use crate::request::{GroupVersionKind, KubernetesAdmissionRequest};

            use jsonpath_lib as jsonpath;
            use serde::Serialize;
            use serde::ser::StdError;

            use k8s_openapi::api::batch::v1::{CronJob, CronJobSpec, JobSpec, JobTemplateSpec};
            use k8s_openapi::api::core::v1::PodTemplateSpec;
            use k8s_openapi::api::core::v1::{ReplicationController, ReplicationControllerSpec};
            use k8s_openapi::api::apps::v1::{
                DaemonSet, DaemonSetSpec, Deployment, DeploymentSpec, ReplicaSet, ReplicaSetSpec,
                StatefulSet, StatefulSetSpec,
            };
        }
    }

    #[test]
    fn test_mutate_request() -> std::result::Result<(), ()> {
        let mutated_object = json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "name": "security-context-demo-4"
            },
            "spec": {
                "containers": [
                {
                    "name": "sec-ctx-4",
                    "image": "gcr.io/google-samples/node-hello:1.0",
                    "securityContext": {
                        "capabilities": {
                            "add": ["NET_ADMIN", "SYS_TIME"],
                            "drop": ["BPF"]
                        }
                    }
                }
                ]
            }
        });
        let expected_object = mutated_object.clone();

        let response_raw = mutate_request(mutated_object).unwrap();
        let response: ValidationResponse = serde_json::from_slice(&response_raw).unwrap();

        assert_json_eq!(response.mutated_object, expected_object);

        Ok(())
    }

    #[test]
    fn test_accept_request() -> std::result::Result<(), ()> {
        let response_raw = accept_request().unwrap();
        let response: ValidationResponse = serde_json::from_slice(&response_raw).unwrap();

        assert!(response.mutated_object.is_none());
        assert!(response.audit_annotations.is_none());
        assert!(response.warnings.is_none());
        Ok(())
    }

    #[test]
    fn test_reject_request() -> std::result::Result<(), ()> {
        let code = 500;
        let expected_code = code;

        let message = String::from("internal error");
        let expected_message = message.clone();

        let warnings = vec![String::from("warning 1"), String::from("warning 2")];

        let mut audit_annotations: HashMap<String, String> = HashMap::new();
        audit_annotations.insert(
            String::from("imagepolicy.example.com/error"),
            String::from("image-blacklisted"),
        );

        let response_raw = reject_request(
            Some(message),
            Some(code),
            Some(audit_annotations.clone()),
            Some(warnings.clone()),
        )
        .unwrap();
        let response: ValidationResponse = serde_json::from_slice(&response_raw).unwrap();

        assert!(response.mutated_object.is_none());
        assert_eq!(response.code, Some(expected_code));
        assert_eq!(response.message, Some(expected_message));
        assert_eq!(response.audit_annotations, Some(audit_annotations));
        assert_eq!(response.warnings, Some(warnings));
        Ok(())
    }

    #[test]
    fn try_protocol_version_guest() -> std::result::Result<(), ()> {
        let response = protocol_version_guest(&[0; 0]).unwrap();
        let version: ProtocolVersion = serde_json::from_slice(&response).unwrap();

        assert_eq!(version, ProtocolVersion::V1);
        Ok(())
    }

    #[cfg(feature = "cluster-context")]
    fn create_validation_request<T: Serialize>(object: T, kind: &str) -> ValidationRequest<()> {
        let (group, version) = group_version_for_kind(kind);
        create_validation_request_with_gvk(object, group, version, kind)
    }

    #[cfg(feature = "cluster-context")]
    fn create_validation_request_with_gvk<T: Serialize>(
        object: T,
        group: &str,
        version: &str,
        kind: &str,
    ) -> ValidationRequest<()> {
        let value = serde_json::to_value(object).unwrap();
        ValidationRequest {
            settings: (),
            request: KubernetesAdmissionRequest {
                kind: GroupVersionKind {
                    group: group.to_owned(),
                    version: version.to_owned(),
                    kind: kind.to_owned(),
                },
                object: value,
                ..Default::default()
            },
        }
    }

    #[cfg(feature = "cluster-context")]
    fn group_version_for_kind(kind: &str) -> (&'static str, &'static str) {
        match kind {
            "Deployment" | "ReplicaSet" | "StatefulSet" | "DaemonSet" => ("apps", "v1"),
            "CronJob" | "Job" => ("batch", "v1"),
            "ReplicationController" | "Pod" => ("", "v1"),
            _ => ("", ""),
        }
    }

    #[cfg(feature = "cluster-context")]
    fn check_if_automount_service_account_token_is_true(
        raw_response: std::result::Result<Vec<u8>, Box<dyn StdError + Send + Sync>>,
    ) -> std::result::Result<(), ()> {
        assert!(raw_response.is_ok());
        let response: ValidationResponse = serde_json::from_slice(&raw_response.unwrap()).unwrap();
        assert!(response.accepted);

        assert!(
            response.mutated_object.is_some(),
            "Request should be mutated"
        );
        let automount_service_account_token = jsonpath::select(
            response.mutated_object.as_ref().unwrap(),
            "$.spec.template.spec.automountServiceAccountToken",
        )
        .unwrap();
        assert_eq!(
            automount_service_account_token,
            vec![true],
            "Request not mutated"
        );

        Ok(())
    }

    #[cfg(feature = "cluster-context")]
    #[test]
    fn test_mutate_pod_spec_from_request_with_deployment() -> std::result::Result<(), ()> {
        let deployment = Deployment {
            spec: Some(DeploymentSpec {
                template: PodTemplateSpec {
                    spec: Some(PodSpec {
                        automount_service_account_token: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation_request = create_validation_request(deployment, "Deployment");

        let new_pod_spec = PodSpec {
            automount_service_account_token: Some(true),
            ..Default::default()
        };

        let raw_response = mutate_pod_spec_from_request(validation_request, new_pod_spec);
        check_if_automount_service_account_token_is_true(raw_response)
    }

    #[cfg(feature = "cluster-context")]
    #[test]
    fn test_mutate_pod_spec_from_request_with_replicaset() -> std::result::Result<(), ()> {
        let replicaset = ReplicaSet {
            spec: Some(ReplicaSetSpec {
                template: Some(PodTemplateSpec {
                    spec: Some(PodSpec {
                        automount_service_account_token: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation_request = create_validation_request(replicaset, "ReplicaSet");

        let new_pod_spec = PodSpec {
            automount_service_account_token: Some(true),
            ..Default::default()
        };

        let raw_response = mutate_pod_spec_from_request(validation_request, new_pod_spec);
        check_if_automount_service_account_token_is_true(raw_response)
    }

    #[cfg(feature = "cluster-context")]
    #[test]
    fn test_mutate_pod_spec_from_request_with_statefulset() -> std::result::Result<(), ()> {
        let statefulset = StatefulSet {
            spec: Some(StatefulSetSpec {
                template: PodTemplateSpec {
                    spec: Some(PodSpec {
                        automount_service_account_token: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation_request = create_validation_request(statefulset, "StatefulSet");

        let new_pod_spec = PodSpec {
            automount_service_account_token: Some(true),
            ..Default::default()
        };
        let raw_response = mutate_pod_spec_from_request(validation_request, new_pod_spec);
        check_if_automount_service_account_token_is_true(raw_response)
    }

    #[cfg(feature = "cluster-context")]
    #[test]
    fn test_mutate_pod_spec_from_request_with_daemonset() -> std::result::Result<(), ()> {
        let daemonset = DaemonSet {
            spec: Some(DaemonSetSpec {
                template: PodTemplateSpec {
                    spec: Some(PodSpec {
                        automount_service_account_token: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation_request = create_validation_request(daemonset, "DaemonSet");

        let new_pod_spec = PodSpec {
            automount_service_account_token: Some(true),
            ..Default::default()
        };
        let raw_response = mutate_pod_spec_from_request(validation_request, new_pod_spec);
        check_if_automount_service_account_token_is_true(raw_response)
    }

    #[cfg(feature = "cluster-context")]
    #[test]
    fn test_mutate_pod_spec_from_request_with_replicationcontroller() -> std::result::Result<(), ()>
    {
        let replicationcontroller = ReplicationController {
            spec: Some(ReplicationControllerSpec {
                template: Some(PodTemplateSpec {
                    spec: Some(PodSpec {
                        automount_service_account_token: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation_request =
            create_validation_request(replicationcontroller, "ReplicationController");

        let new_pod_spec = PodSpec {
            automount_service_account_token: Some(true),
            ..Default::default()
        };
        let raw_response = mutate_pod_spec_from_request(validation_request, new_pod_spec);
        check_if_automount_service_account_token_is_true(raw_response)
    }

    #[cfg(feature = "cluster-context")]
    #[test]
    fn test_mutate_pod_spec_from_request_with_cronjob() -> std::result::Result<(), ()> {
        let cronjob = CronJob {
            spec: Some(CronJobSpec {
                job_template: JobTemplateSpec {
                    spec: Some(JobSpec {
                        template: PodTemplateSpec {
                            spec: Some(PodSpec {
                                automount_service_account_token: Some(false),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation_request = create_validation_request(cronjob, "CronJob");

        let new_pod_spec = PodSpec {
            automount_service_account_token: Some(true),
            ..Default::default()
        };

        let raw_response = mutate_pod_spec_from_request(validation_request, new_pod_spec);
        assert!(raw_response.is_ok());
        let response: ValidationResponse = serde_json::from_slice(&raw_response.unwrap()).unwrap();
        assert!(response.accepted);

        assert!(
            response.mutated_object.is_some(),
            "Request should be mutated"
        );
        let automount_service_account_token = jsonpath::select(
            response.mutated_object.as_ref().unwrap(),
            "$.spec.jobTemplate.spec.template.spec.automountServiceAccountToken",
        )
        .unwrap();
        assert_eq!(
            automount_service_account_token,
            vec![true],
            "Request not mutated"
        );

        Ok(())
    }

    #[cfg(feature = "cluster-context")]
    #[test]
    fn test_mutate_pod_spec_from_request_with_job() -> std::result::Result<(), ()> {
        let job = Job {
            spec: Some(JobSpec {
                template: PodTemplateSpec {
                    spec: Some(PodSpec {
                        automount_service_account_token: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation_request = create_validation_request(job, "Job");

        let new_pod_spec = PodSpec {
            automount_service_account_token: Some(true),
            ..Default::default()
        };
        let raw_response = mutate_pod_spec_from_request(validation_request, new_pod_spec);
        check_if_automount_service_account_token_is_true(raw_response)
    }

    #[cfg(feature = "cluster-context")]
    #[test]
    fn test_mutate_pod_spec_from_request_with_pod() -> std::result::Result<(), ()> {
        let pod = Pod {
            spec: Some(PodSpec {
                automount_service_account_token: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation_request = create_validation_request(pod, "Pod");

        let new_pod_spec = PodSpec {
            automount_service_account_token: Some(true),
            ..Default::default()
        };

        let raw_response = mutate_pod_spec_from_request(validation_request, new_pod_spec);
        assert!(raw_response.is_ok());
        let response: ValidationResponse = serde_json::from_slice(&raw_response.unwrap()).unwrap();
        assert!(response.accepted);

        assert!(
            response.mutated_object.is_some(),
            "Request should be mutated"
        );
        let automount_service_account_token = jsonpath::select(
            response.mutated_object.as_ref().unwrap(),
            "$.spec.automountServiceAccountToken",
        )
        .unwrap();
        assert_eq!(
            automount_service_account_token,
            vec![true],
            "Request not mutated"
        );

        Ok(())
    }

    #[cfg(feature = "cluster-context")]
    #[test]
    fn test_mutate_pod_spec_from_request_with_invalid_resource_type() -> std::result::Result<(), ()>
    {
        let pod = Pod {
            spec: Some(PodSpec {
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation_request = create_validation_request(pod, "InvalidType");

        let new_pod_spec = PodSpec {
            automount_service_account_token: Some(true),
            ..Default::default()
        };

        let raw_response = mutate_pod_spec_from_request(validation_request, new_pod_spec);
        assert!(raw_response.is_ok());
        let response: ValidationResponse = serde_json::from_slice(&raw_response.unwrap()).unwrap();
        assert!(!response.accepted);
        let error_message = response.message.unwrap_or_default();
        let expected_error_message = crate::request::supported_pod_spec_objects_message();
        assert_eq!(error_message, expected_error_message);

        Ok(())
    }

    #[cfg(feature = "cluster-context")]
    #[test]
    fn test_mutate_pod_spec_from_request_rejects_mismatched_group_version_kind() {
        let deployment = Deployment {
            spec: Some(DeploymentSpec {
                template: PodTemplateSpec {
                    spec: Some(PodSpec {
                        automount_service_account_token: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation_request =
            create_validation_request_with_gvk(deployment, "argocd.io", "v1", "Deployment");

        let new_pod_spec = PodSpec {
            automount_service_account_token: Some(true),
            ..Default::default()
        };

        let raw_response = mutate_pod_spec_from_request(validation_request, new_pod_spec);

        assert!(raw_response.is_ok());
        let response: ValidationResponse = serde_json::from_slice(&raw_response.unwrap()).unwrap();
        assert!(!response.accepted);
        assert_eq!(
            response.message.unwrap_or_default(),
            crate::request::supported_pod_spec_objects_message()
        );
    }
}
