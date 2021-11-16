use anyhow::Result;
use slog::{Drain, OwnedKVList, Record};

use super::event;

/// A logging drain designed to integrate with [`slog::Drain`]
///
/// The drain can be easily added to a regular [`slog::Logger`]:
///
/// ```rust
/// use kubewarden_policy_sdk::logging;
/// use slog::{Logger, o};
///
/// let log = Logger::root(logging::KubewardenDrain::new(), o!("logger_key1" => "logger_value1"));
/// ```
///
/// The drain behaves differently based on the target architecture used at
/// build time.
///
/// The `wasm32` target architecture will cause the drain to use the [`wapc_guest::host_call`]
/// helper to propagate the log event from the Wasm guest to the native host
/// environment. This is the default behaviour for Kubewarden policies at
/// execution time.
///
/// Building for a non `wasm32` architecture will cause the drain to print the log
/// entries on the standard output.
/// This is useful for running test of policies via a regular `cargo test`.

#[derive(Default)]
pub struct KubewardenDrain {}

impl KubewardenDrain {
    /// Convenience function that creates a `KubewardenDrain` instance wrapped
    /// into a [`slog::Fuse`]
    pub fn new() -> slog::Fuse<KubewardenDrain> {
        let drain: KubewardenDrain = Default::default();
        drain.fuse()
    }
}

impl slog::Drain for KubewardenDrain {
    type Ok = ();
    type Err = anyhow::Error;

    #[cfg(not(target_arch = "wasm32"))]
    fn log(&self, rinfo: &Record, logger_values: &OwnedKVList) -> Result<()> {
        let event = event::new(rinfo, logger_values).unwrap();
        println!("{}", serde_json::to_string(&event)?);

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn log(&self, rinfo: &Record, logger_values: &OwnedKVList) -> Result<()> {
        let event = event::new(rinfo, logger_values).unwrap();
        let msg = serde_json::to_vec(&event).unwrap();
        wapc_guest::host_call("kubewarden", "tracing", "log", &msg)
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("error invoking wapc logging facility: {:?}", e))
    }
}
