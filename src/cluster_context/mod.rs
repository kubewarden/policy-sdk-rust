//! This module provides a way for policies to gather information from a
//! Kubernetes cluster. The policies making use of this feature are called
//! "context-aware" policies.
//!
//! > **Warning:** currently (Jan 2022), this feature is still considered alpha.
//! > Because of that the API is subject to changes.
//!
//! Details about Kubernetes resources already existing inside of the cluster can
//! be obtained via the [`ClusterContext`] object.

extern crate wapc_guest as guest;

use anyhow::{anyhow, Result};

use k8s_openapi::api::core::v1::{Namespace, Service};
use k8s_openapi::api::networking::v1::Ingress;
use k8s_openapi::List;

pub mod client;

/// A `ClusterContext` allows a waPC guest policy to retrieve cluster
/// contextual information from a Kubernetes cluster.
///
/// Right now a set of well known resources is hardcoded, but the idea
/// is to generalize this so the SDK can support any kind of
/// Kubernetes resource and custom resource definition.
///
/// ## Usage inside of policies
///
/// When using `ClusterContext` inside of a policy, there's no need to
/// cache its instance via something like [`lazy_static`](https://crates.io/crates/lazy_static).
/// Allocating the `ClusterContext` is cheap, both in terms of cpu cycles and
/// memory.
///
/// An instance of `ClusterContext` can be created in this way:
///
/// ```
/// use kubewarden_policy_sdk::cluster_context::ClusterContext;
///
/// let cluster_cts = ClusterContext::default();
/// ```
///
/// ## Usage inside of unit tests
///
/// `ClusterContext` cannot behave the same way when its used inside of Wasm code
/// and native code (like when running policy tests via `cargo test`).
///
/// When running inside of unit tests, there's no communication with a real Kubernetes
/// cluster. The Kubernetes resources returned by the `ClusterContext` instance
/// are mocks defined inside of a [`client::TestClient`] object.
///
/// The implementation of `ClusterContext::default()` automatically allocates
/// a default [`client::TestClient`] object that returns successful empty responses.
///
/// If you want to provide ad-hoc responses (including communication errors), please
/// build the `ClusterContext` using the [`ClusterContext::new_with_client`] method.
///
pub struct ClusterContext {
    client: Box<dyn client::Client>,
}

impl Default for ClusterContext {
    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        use self::client::WapcClient;

        ClusterContext {
            client: Box::new(WapcClient {}),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        use self::client::TestClient;

        ClusterContext {
            client: Box::new(TestClient {}),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum NamespaceFilter {
    AllNamespaces,
    Namespace(String),
}

impl ClusterContext {
    /// This method is available only when the code is **not** build for the Wasm
    /// target.
    ///
    /// This method is meant to be used only when writing unit tests, so that
    /// an instance of [`client::TestClient`] can be used to provide mock results.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_with_client(client: Box<dyn client::Client>) -> Self {
        ClusterContext { client }
    }

    /// Return the list of `Ingress` resources that exist in the
    /// cluster.
    pub fn ingresses(&self, namespace: NamespaceFilter) -> Result<Vec<Ingress>> {
        // TODO (ereslibre): use macros to remove duplication and then
        // generalize
        Ok(self
            .client
            .ingresses()
            .map_err(|err| anyhow!("failed to call ingresses binding: {}", err))
            .and_then(|ingresses| {
                Ok(
                    serde_json::from_str::<List<Ingress>>(std::str::from_utf8(&ingresses)?)
                        .map_err(|err| anyhow!("failed to unmarshal ingress list: {}", err))?
                        .items,
                )
            })?
            .iter()
            .filter_map(|ingress| match &namespace {
                NamespaceFilter::AllNamespaces => Some(ingress.clone()),
                NamespaceFilter::Namespace(namespace_filter) => {
                    if let Some(ingress_namespace) = &ingress.metadata.namespace {
                        if namespace_filter == ingress_namespace {
                            Some(ingress.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            })
            .collect())
    }

    /// Return the list of `Namespace` resources that exist in the
    /// cluster.
    pub fn namespaces(&self) -> Result<Vec<Namespace>> {
        // TODO (ereslibre): use macros to remove duplication and then
        // generalize
        self.client
            .namespaces()
            .map_err(|err| anyhow!("failed to call namespaces binding: {}", err))
            .and_then(|namespaces| {
                Ok(
                    serde_json::from_str::<List<Namespace>>(std::str::from_utf8(&namespaces)?)
                        .map_err(|err| anyhow!("failed to unmarshal namespace list: {}", err))?
                        .items,
                )
            })
    }

    /// Return the list of `Service` resources that exist in the
    /// cluster.
    pub fn services(&self, namespace: NamespaceFilter) -> Result<Vec<Service>> {
        // TODO (ereslibre): use macros to remove duplication and then
        // generalize
        Ok(self
            .client
            .services()
            .map_err(|err| anyhow!("failed to call services binding: {}", err))
            .and_then(|services| {
                Ok(
                    serde_json::from_str::<List<Service>>(std::str::from_utf8(&services)?)
                        .map_err(|err| anyhow!("failed to unmarshal service list: {}", err))?
                        .items,
                )
            })?
            .iter()
            .filter_map(|service| match &namespace {
                NamespaceFilter::AllNamespaces => Some(service.clone()),
                NamespaceFilter::Namespace(namespace_filter) => {
                    if let Some(service_namespace) = &service.metadata.namespace {
                        if namespace_filter == service_namespace {
                            Some(service.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            })
            .collect())
    }
}

impl ClusterContext {
    /// Return a specific ingress object with a given name and a
    /// namespace filter. If the namespace filter is broad, more than
    /// one resource might be returned.
    pub fn ingress(&self, namespace: NamespaceFilter, name: &str) -> Result<Vec<Ingress>> {
        // TODO (ereslibre): use macros to remove duplication and then
        // generalize
        Ok(self
            .ingresses(namespace)?
            .into_iter()
            .filter(|ingress| ingress.metadata.name == Some(name.to_string()))
            .collect())
    }

    // Return a specific namespace with a given name.
    pub fn namespace(&self, name: &str) -> Result<Option<Namespace>> {
        // TODO (ereslibre): use macros to remove duplication and then
        // generalize
        Ok(self
            .namespaces()?
            .into_iter()
            .find(|namespace| namespace.metadata.name == Some(name.to_string())))
    }

    /// Return a specific service object with a given name and a
    /// namespace filter. If the namespace filter is broad, more than
    /// one resource might be returned.
    pub fn service(&self, namespace: NamespaceFilter, name: &str) -> Result<Vec<Service>> {
        // TODO (ereslibre): use macros to remove duplication and then
        // generalize
        Ok(self
            .services(namespace)?
            .into_iter()
            .filter(|service| service.metadata.name == Some(name.to_string()))
            .collect())
    }
}
