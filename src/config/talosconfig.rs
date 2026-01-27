// SPDX-License-Identifier: MIT OR Apache-2.0

//! Talosctl configuration file parser
//!
//! This module provides functionality to parse the talosctl config file
//! (typically `~/.talos/config`) which contains connection information for
//! multiple Talos clusters.
//!
//! # Example
//!
//! ```no_run
//! use talos_api_rs::config::TalosConfig;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load from default location (~/.talos/config)
//! let config = TalosConfig::load_default()?;
//!
//! // Get the active context
//! if let Some(context_name) = &config.context {
//!     if let Some(ctx) = config.contexts.get(context_name) {
//!         println!("Endpoints: {:?}", ctx.endpoints);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, TalosError};

/// Represents the entire talosctl configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TalosConfig {
    /// The currently active context name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Map of context names to their configurations
    pub contexts: HashMap<String, TalosContext>,
}

/// Configuration for a single Talos cluster context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TalosContext {
    /// List of control plane endpoints (IP addresses or DNS names)
    pub endpoints: Vec<String>,

    /// Optional list of specific node targets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<String>>,

    /// CA certificate in PEM format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca: Option<String>,

    /// Client certificate in PEM format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crt: Option<String>,

    /// Client private key in PEM format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

impl TalosConfig {
    /// Load configuration from the default location (~/.talos/config)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The home directory cannot be determined
    /// - The config file cannot be read
    /// - The config file is malformed
    #[allow(clippy::result_large_err)]
    pub fn load_default() -> Result<Self> {
        let config_path = Self::default_path()?;
        Self::load_from_path(&config_path)
    }

    /// Load configuration from a specific path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the talosconfig file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file is malformed YAML
    #[allow(clippy::result_large_err)]
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            TalosError::Config(format!(
                "Failed to read config file {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        Self::from_yaml(&content)
    }

    /// Parse configuration from YAML string
    ///
    /// # Arguments
    ///
    /// * `yaml` - YAML content as string
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML is malformed
    #[allow(clippy::result_large_err)]
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml)
            .map_err(|e| TalosError::Config(format!("Failed to parse config YAML: {}", e)))
    }

    /// Get the default config file path (~/.talos/config)
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined
    #[allow(clippy::result_large_err)]
    pub fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| TalosError::Config("Could not determine home directory".to_string()))?;

        Ok(home.join(".talos").join("config"))
    }

    /// Get the path to the config file, respecting TALOSCONFIG environment variable
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined when TALOSCONFIG is not set
    #[allow(clippy::result_large_err)]
    pub fn config_path() -> Result<PathBuf> {
        if let Ok(env_path) = std::env::var("TALOSCONFIG") {
            Ok(PathBuf::from(env_path))
        } else {
            Self::default_path()
        }
    }

    /// Get the currently active context
    ///
    /// # Returns
    ///
    /// Returns `None` if no active context is set or if the context doesn't exist
    pub fn active_context(&self) -> Option<&TalosContext> {
        self.context
            .as_ref()
            .and_then(|name| self.contexts.get(name))
    }

    /// Get a context by name
    ///
    /// # Arguments
    ///
    /// * `name` - The context name to retrieve
    pub fn get_context(&self, name: &str) -> Option<&TalosContext> {
        self.contexts.get(name)
    }

    /// List all available context names
    pub fn context_names(&self) -> Vec<&String> {
        self.contexts.keys().collect()
    }
}

impl TalosContext {
    /// Get the first endpoint, if any
    pub fn first_endpoint(&self) -> Option<&String> {
        self.endpoints.first()
    }

    /// Get the first node, if any
    pub fn first_node(&self) -> Option<&String> {
        self.nodes.as_ref().and_then(|nodes| nodes.first())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CONFIG: &str = r#"
context: my-cluster
contexts:
  my-cluster:
    endpoints:
      - 10.0.0.2
      - 10.0.0.3
    ca: |
      -----BEGIN CERTIFICATE-----
      MIIBcDCCARegAwIBAgIRAMK1...
      -----END CERTIFICATE-----
    crt: |
      -----BEGIN CERTIFICATE-----
      MIIBbjCCAROgAwIBAgIQdB...
      -----END CERTIFICATE-----
    key: |
      -----BEGIN ED25519 PRIVATE KEY-----
      MC4CAQAwBQYDK2VwBCIEIA...
      -----END ED25519 PRIVATE KEY-----
  another-cluster:
    endpoints:
      - 192.168.1.10
    nodes:
      - 192.168.1.11
      - 192.168.1.12
"#;

    #[test]
    fn test_parse_basic_config() {
        let config = TalosConfig::from_yaml(SAMPLE_CONFIG).unwrap();

        assert_eq!(config.context, Some("my-cluster".to_string()));
        assert_eq!(config.contexts.len(), 2);
    }

    #[test]
    fn test_active_context() {
        let config = TalosConfig::from_yaml(SAMPLE_CONFIG).unwrap();

        let active = config.active_context().unwrap();
        assert_eq!(active.endpoints, vec!["10.0.0.2", "10.0.0.3"]);
        assert!(active.ca.is_some());
        assert!(active.crt.is_some());
        assert!(active.key.is_some());
    }

    #[test]
    fn test_get_context() {
        let config = TalosConfig::from_yaml(SAMPLE_CONFIG).unwrap();

        let ctx = config.get_context("another-cluster").unwrap();
        assert_eq!(ctx.endpoints, vec!["192.168.1.10"]);
        assert_eq!(
            ctx.nodes,
            Some(vec!["192.168.1.11".to_string(), "192.168.1.12".to_string()])
        );
    }

    #[test]
    fn test_context_names() {
        let config = TalosConfig::from_yaml(SAMPLE_CONFIG).unwrap();

        let mut names = config.context_names();
        names.sort();

        assert_eq!(names, vec!["another-cluster", "my-cluster"]);
    }

    #[test]
    fn test_first_endpoint() {
        let config = TalosConfig::from_yaml(SAMPLE_CONFIG).unwrap();
        let ctx = config.get_context("my-cluster").unwrap();

        assert_eq!(ctx.first_endpoint(), Some(&"10.0.0.2".to_string()));
    }

    #[test]
    fn test_first_node() {
        let config = TalosConfig::from_yaml(SAMPLE_CONFIG).unwrap();
        let ctx = config.get_context("another-cluster").unwrap();

        assert_eq!(ctx.first_node(), Some(&"192.168.1.11".to_string()));
    }

    #[test]
    fn test_missing_context() {
        let config = TalosConfig::from_yaml(SAMPLE_CONFIG).unwrap();
        assert!(config.get_context("nonexistent").is_none());
    }

    #[test]
    fn test_minimal_config() {
        let minimal = r#"
contexts:
  minimal:
    endpoints:
      - 127.0.0.1:50000
"#;

        let config = TalosConfig::from_yaml(minimal).unwrap();
        assert_eq!(config.context, None);
        assert_eq!(config.contexts.len(), 1);

        let ctx = config.get_context("minimal").unwrap();
        assert_eq!(ctx.endpoints, vec!["127.0.0.1:50000"]);
        assert!(ctx.ca.is_none());
        assert!(ctx.nodes.is_none());
    }
}
