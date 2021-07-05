//! This module provides a way to propagate log events
//! from a kubewarden policy to the host runtime.
//!
//! The logging infrastructure relies on the popular [slog](https://crates.io/crates/slog)
//! crate. slog allows both structured and unstructured logging.
//!
//! # Usage
//!
//! Nothing special has to be done compared to using `slog` inside of a regular
//! Rust project.
//!
//! All the logs are sent to a [`slog::Logger`]. The logger then relies on a
//! [`slog::Drain`] to manipulate and dispatch them. For Kubewarden policies,
//! the drain must be an instance of [`KubewardenDrain`](crate::logging::KubewardenDrain).
//!
//! Log events can be generated using the macros provided by the [`slog::log`]
//! module.
//!
//! ## Example
//!
//! This code snippet can be placed inside of the `validate` function of a
//! Kubewarden policy:
//!
//! ```rust
//! use kubewarden_policy_sdk::{logging, accept_request};
//! use slog::{Drain, Logger, o, info};
//!
//! fn validate(payload: &[u8]) -> wapc_guest::CallResult {
//!   let drain = logging::KubewardenDrain::new().fuse();
//!   let log = Logger::root(drain, o!("logger_key1" => "logger_value1"));
//!   info!(log, "just a message");
//!   info!(log, "{} at work", "interpolation");
//!   info!(log, "structured log"; "string_val" => "string", "number" => 42, "enabled" => true);
//!
//!   // policy evaluation goes on...
//!   accept_request()
//! }
//! ```
mod drain;
mod event;
mod ser;

pub use drain::KubewardenDrain;
