use std::collections::HashSet;

/// Trait that must be provided by the Policy settings
/// Used to define special users who are not affected by the policy
pub trait Trusties {
    /// List of users not affected by the policy
    fn trusted_users(&self) -> HashSet<String>;
    /// List of groups not affected by the policy
    fn trusted_groups(&self) -> HashSet<String>;
}
