//! Error types for the Kubewarden Policy SDK

/// A specialized Result type for Kubewarden SDK operations
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for Kubewarden SDK operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error occurred during serialization
    #[error("error serializing {context}: {source}")]
    Serialization {
        /// Context describing what was being serialized
        context: String,
        /// The underlying serialization error
        #[source]
        source: serde_json::Error,
    },

    /// Error occurred during deserialization
    #[error("error deserializing {context}: {source}")]
    Deserialization {
        /// Context describing what was being deserialized
        context: String,
        /// The underlying deserialization error
        #[source]
        source: serde_json::Error,
    },

    /// Error occurred when calling a host capability
    #[error("error invoking host capability {operation}: {source}")]
    HostCall {
        /// The host capability operation that was invoked
        operation: String,
        /// The underlying error from the host call
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Validation error with a custom message
    #[error("{0}")]
    Validation(String),

    /// I/O error occurred
    #[error("error reading file {path}: {source}")]
    Io {
        /// Path to the file that caused the error
        path: String,
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Error occurred in logging system
    #[error("error in logging: {source}")]
    Logging {
        /// The underlying logging error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

// Implement From for common error types to enable ? operator
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Deserialization {
            context: "JSON data".to_string(),
            source: e,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io {
            path: "<unknown>".to_string(),
            source: e,
        }
    }
}

impl From<slog::Error> for Error {
    fn from(e: slog::Error) -> Self {
        Error::Logging {
            source: Box::new(e),
        }
    }
}
