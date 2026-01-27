// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for configuration-related operations.
//!
//! This module provides ergonomic builders and types for working with
//! Talos machine configuration.

use crate::api::machine::{
    apply_configuration_request::Mode as ProtoMode, ApplyConfiguration as ProtoApplyConfiguration,
    ApplyConfigurationRequest as ProtoRequest, ApplyConfigurationResponse as ProtoResponse,
};
use std::time::Duration;

/// Mode for applying configuration changes.
///
/// Determines how the node handles configuration updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ApplyMode {
    /// Reboot the node after applying configuration.
    Reboot,
    /// Automatically determine the best mode based on changes.
    #[default]
    Auto,
    /// Apply configuration without rebooting (if possible).
    NoReboot,
    /// Stage the configuration for next boot.
    Staged,
    /// Try the configuration temporarily; revert if not confirmed.
    Try,
}

impl From<ApplyMode> for i32 {
    fn from(mode: ApplyMode) -> Self {
        match mode {
            ApplyMode::Reboot => ProtoMode::Reboot as i32,
            ApplyMode::Auto => ProtoMode::Auto as i32,
            ApplyMode::NoReboot => ProtoMode::NoReboot as i32,
            ApplyMode::Staged => ProtoMode::Staged as i32,
            ApplyMode::Try => ProtoMode::Try as i32,
        }
    }
}

impl From<i32> for ApplyMode {
    fn from(value: i32) -> Self {
        match value {
            0 => ApplyMode::Reboot,
            1 => ApplyMode::Auto,
            2 => ApplyMode::NoReboot,
            3 => ApplyMode::Staged,
            4 => ApplyMode::Try,
            _ => ApplyMode::Auto, // Default fallback
        }
    }
}

impl std::fmt::Display for ApplyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyMode::Reboot => write!(f, "reboot"),
            ApplyMode::Auto => write!(f, "auto"),
            ApplyMode::NoReboot => write!(f, "no-reboot"),
            ApplyMode::Staged => write!(f, "staged"),
            ApplyMode::Try => write!(f, "try"),
        }
    }
}

/// Builder for creating configuration apply requests.
///
/// # Example
///
/// ```no_run
/// use talos_api_rs::resources::ApplyConfigurationRequest;
/// use talos_api_rs::resources::ApplyMode;
///
/// let request = ApplyConfigurationRequest::builder()
///     .config_yaml("machine:\n  type: worker")
///     .mode(ApplyMode::NoReboot)
///     .dry_run(true)
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct ApplyConfigurationRequest {
    /// Raw configuration data (YAML bytes)
    pub data: Vec<u8>,
    /// Mode for applying configuration
    pub mode: ApplyMode,
    /// If true, validate only without applying
    pub dry_run: bool,
    /// Timeout for try mode (optional)
    pub try_mode_timeout: Option<Duration>,
}

impl ApplyConfigurationRequest {
    /// Create a new builder for `ApplyConfigurationRequest`.
    #[must_use]
    pub fn builder() -> ApplyConfigurationRequestBuilder {
        ApplyConfigurationRequestBuilder::default()
    }

    /// Create a request from raw YAML configuration.
    #[must_use]
    pub fn from_yaml(yaml: impl AsRef<str>) -> Self {
        Self {
            data: yaml.as_ref().as_bytes().to_vec(),
            mode: ApplyMode::Auto,
            dry_run: false,
            try_mode_timeout: None,
        }
    }

    /// Create a request from raw bytes.
    #[must_use]
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            mode: ApplyMode::Auto,
            dry_run: false,
            try_mode_timeout: None,
        }
    }
}

impl From<ApplyConfigurationRequest> for ProtoRequest {
    fn from(req: ApplyConfigurationRequest) -> Self {
        ProtoRequest {
            data: req.data,
            mode: req.mode.into(),
            dry_run: req.dry_run,
            try_mode_timeout: req.try_mode_timeout.map(|d| prost_types::Duration {
                seconds: d.as_secs() as i64,
                nanos: d.subsec_nanos() as i32,
            }),
        }
    }
}

/// Builder for `ApplyConfigurationRequest`.
#[derive(Debug, Clone, Default)]
pub struct ApplyConfigurationRequestBuilder {
    data: Vec<u8>,
    mode: ApplyMode,
    dry_run: bool,
    try_mode_timeout: Option<Duration>,
}

impl ApplyConfigurationRequestBuilder {
    /// Set the configuration from a YAML string.
    #[must_use]
    pub fn config_yaml(mut self, yaml: impl AsRef<str>) -> Self {
        self.data = yaml.as_ref().as_bytes().to_vec();
        self
    }

    /// Set the configuration from raw bytes.
    #[must_use]
    pub fn config_bytes(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Set the configuration from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn config_file(mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        self.data = std::fs::read(path)?;
        Ok(self)
    }

    /// Set the apply mode.
    #[must_use]
    pub fn mode(mut self, mode: ApplyMode) -> Self {
        self.mode = mode;
        self
    }

    /// Enable dry-run mode (validate only, don't apply).
    #[must_use]
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Set the timeout for try mode.
    #[must_use]
    pub fn try_mode_timeout(mut self, timeout: Duration) -> Self {
        self.try_mode_timeout = Some(timeout);
        self
    }

    /// Build the request.
    #[must_use]
    pub fn build(self) -> ApplyConfigurationRequest {
        ApplyConfigurationRequest {
            data: self.data,
            mode: self.mode,
            dry_run: self.dry_run,
            try_mode_timeout: self.try_mode_timeout,
        }
    }
}

/// Result of applying a configuration.
#[derive(Debug, Clone)]
pub struct ApplyConfigurationResult {
    /// Node identifier (if available)
    pub node: Option<String>,
    /// Configuration validation warnings
    pub warnings: Vec<String>,
    /// Mode that was actually applied
    pub mode: ApplyMode,
    /// Human-readable description of the result
    pub mode_details: String,
}

impl From<ProtoApplyConfiguration> for ApplyConfigurationResult {
    fn from(proto: ProtoApplyConfiguration) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            warnings: proto.warnings,
            mode: proto.mode.into(),
            mode_details: proto.mode_details,
        }
    }
}

/// Response from applying configuration.
#[derive(Debug, Clone)]
pub struct ApplyConfigurationResponse {
    /// Results from each node
    pub results: Vec<ApplyConfigurationResult>,
}

impl From<ProtoResponse> for ApplyConfigurationResponse {
    fn from(proto: ProtoResponse) -> Self {
        Self {
            results: proto.messages.into_iter().map(Into::into).collect(),
        }
    }
}

impl ApplyConfigurationResponse {
    /// Check if all nodes applied the configuration successfully (no warnings).
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.results.iter().all(|r| r.warnings.is_empty())
    }

    /// Get all warnings from all nodes.
    #[must_use]
    pub fn all_warnings(&self) -> Vec<&str> {
        self.results
            .iter()
            .flat_map(|r| r.warnings.iter().map(String::as_str))
            .collect()
    }

    /// Get the first result (useful for single-node operations).
    #[must_use]
    pub fn first(&self) -> Option<&ApplyConfigurationResult> {
        self.results.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_mode_conversion() {
        assert_eq!(i32::from(ApplyMode::Reboot), 0);
        assert_eq!(i32::from(ApplyMode::Auto), 1);
        assert_eq!(i32::from(ApplyMode::NoReboot), 2);
        assert_eq!(i32::from(ApplyMode::Staged), 3);
        assert_eq!(i32::from(ApplyMode::Try), 4);

        assert_eq!(ApplyMode::from(0), ApplyMode::Reboot);
        assert_eq!(ApplyMode::from(1), ApplyMode::Auto);
        assert_eq!(ApplyMode::from(2), ApplyMode::NoReboot);
        assert_eq!(ApplyMode::from(3), ApplyMode::Staged);
        assert_eq!(ApplyMode::from(4), ApplyMode::Try);
    }

    #[test]
    fn test_builder_pattern() {
        let request = ApplyConfigurationRequest::builder()
            .config_yaml("machine:\n  type: worker")
            .mode(ApplyMode::NoReboot)
            .dry_run(true)
            .try_mode_timeout(Duration::from_secs(60))
            .build();

        assert_eq!(request.data, b"machine:\n  type: worker");
        assert_eq!(request.mode, ApplyMode::NoReboot);
        assert!(request.dry_run);
        assert_eq!(request.try_mode_timeout, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_from_yaml() {
        let request = ApplyConfigurationRequest::from_yaml("test: config");
        assert_eq!(request.data, b"test: config");
        assert_eq!(request.mode, ApplyMode::Auto);
        assert!(!request.dry_run);
    }

    #[test]
    fn test_proto_conversion() {
        let request = ApplyConfigurationRequest::builder()
            .config_yaml("test")
            .mode(ApplyMode::Staged)
            .dry_run(true)
            .try_mode_timeout(Duration::from_secs(120))
            .build();

        let proto: ProtoRequest = request.into();
        assert_eq!(proto.data, b"test");
        assert_eq!(proto.mode, ProtoMode::Staged as i32);
        assert!(proto.dry_run);
        assert!(proto.try_mode_timeout.is_some());
    }

    #[test]
    fn test_apply_mode_display() {
        assert_eq!(ApplyMode::Reboot.to_string(), "reboot");
        assert_eq!(ApplyMode::Auto.to_string(), "auto");
        assert_eq!(ApplyMode::NoReboot.to_string(), "no-reboot");
        assert_eq!(ApplyMode::Staged.to_string(), "staged");
        assert_eq!(ApplyMode::Try.to_string(), "try");
    }
}
