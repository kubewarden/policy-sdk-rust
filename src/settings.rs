use serde::{Deserialize, Serialize};

/// Trait that must be implemented by setting
/// object
pub trait Validatable {
    /// Ensures the values given by the user are valid
    fn validate(&self) -> Result<(), String>;
}

/// A SettingsValidationResponse object holds the outcome of settings
/// validation.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SettingsValidationResponse {
    /// True if the settings are valid
    pub valid: bool,
    /// Message shown to the user when the settings are not valid
    pub message: Option<String>,
}
