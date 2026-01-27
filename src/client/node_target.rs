// SPDX-License-Identifier: MIT OR Apache-2.0

//! Node targeting support for multi-node operations
//!
//! This module provides functionality to target specific nodes when making
//! Talos API calls. By default, API calls go to the endpoint you're connected to,
//! but you can use the `x-talos-node` gRPC metadata header to route requests
//! to specific nodes in the cluster.
//!
//! # Example
//!
//! ```ignore
//! use talos_api_rs::{TalosClient, TalosClientConfig};
//! use talos_api_rs::client::NodeTarget;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = TalosClient::new(TalosClientConfig::default()).await?;
//!
//! // Target a specific node
//! let target = NodeTarget::single("192.168.1.10");
//! let hostname = client.with_node(target).hostname().await?;
//!
//! // Target multiple nodes (cluster-wide)
//! let targets = NodeTarget::multiple(vec!["192.168.1.10", "192.168.1.11"]);
//! # Ok(())
//! # }
//! ```

use tonic::metadata::{Ascii, MetadataValue};
use tonic::Request;

/// The gRPC metadata key for node targeting
pub const NODE_METADATA_KEY: &str = "x-talos-node";

/// Represents a target node or set of nodes for API operations
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NodeTarget {
    /// No specific target - use the connected endpoint
    #[default]
    Default,
    /// Target a single node by IP or hostname
    Single(String),
    /// Target multiple nodes (for cluster-wide operations)
    Multiple(Vec<String>),
}

impl NodeTarget {
    /// Create a target for the default (connected) node
    #[must_use]
    pub fn none() -> Self {
        Self::Default
    }

    /// Create a target for a single node
    #[must_use]
    pub fn single(node: impl Into<String>) -> Self {
        Self::Single(node.into())
    }

    /// Create a target for multiple nodes
    #[must_use]
    pub fn multiple(nodes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Multiple(nodes.into_iter().map(Into::into).collect())
    }

    /// Create from a comma-separated string
    #[must_use]
    pub fn from_csv(csv: &str) -> Self {
        let nodes: Vec<String> = csv
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        match nodes.len() {
            0 => Self::Default,
            1 => Self::Single(nodes.into_iter().next().unwrap()),
            _ => Self::Multiple(nodes),
        }
    }

    /// Check if this is the default target
    #[must_use]
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }

    /// Check if this targets a single node
    #[must_use]
    pub fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Check if this targets multiple nodes
    #[must_use]
    pub fn is_multiple(&self) -> bool {
        matches!(self, Self::Multiple(_))
    }

    /// Get the nodes as a slice (empty for Default)
    #[must_use]
    pub fn nodes(&self) -> &[String] {
        match self {
            Self::Default => &[],
            Self::Single(node) => std::slice::from_ref(node),
            Self::Multiple(nodes) => nodes,
        }
    }

    /// Get the first node, if any
    #[must_use]
    pub fn first(&self) -> Option<&str> {
        match self {
            Self::Default => None,
            Self::Single(node) => Some(node),
            Self::Multiple(nodes) => nodes.first().map(String::as_str),
        }
    }

    /// Convert to comma-separated string for gRPC metadata
    #[must_use]
    pub fn to_csv(&self) -> Option<String> {
        match self {
            Self::Default => None,
            Self::Single(node) => Some(node.clone()),
            Self::Multiple(nodes) => Some(nodes.join(",")),
        }
    }

    /// Apply node targeting to a gRPC request
    pub fn apply_to_request<T>(&self, mut request: Request<T>) -> Request<T> {
        if let Some(node_value) = self.to_csv() {
            if let Ok(metadata_value) = node_value.parse::<MetadataValue<Ascii>>() {
                request
                    .metadata_mut()
                    .insert(NODE_METADATA_KEY, metadata_value);
            }
        }
        request
    }
}

impl From<&str> for NodeTarget {
    fn from(s: &str) -> Self {
        if s.is_empty() {
            Self::Default
        } else if s.contains(',') {
            Self::from_csv(s)
        } else {
            Self::Single(s.to_string())
        }
    }
}

impl From<String> for NodeTarget {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<Vec<String>> for NodeTarget {
    fn from(nodes: Vec<String>) -> Self {
        match nodes.len() {
            0 => Self::Default,
            1 => Self::Single(nodes.into_iter().next().unwrap()),
            _ => Self::Multiple(nodes),
        }
    }
}

impl From<Option<String>> for NodeTarget {
    fn from(opt: Option<String>) -> Self {
        match opt {
            Some(s) => Self::from(s),
            None => Self::Default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_target_default() {
        let target = NodeTarget::none();
        assert!(target.is_default());
        assert!(!target.is_single());
        assert!(!target.is_multiple());
        assert_eq!(target.nodes(), &[] as &[String]);
        assert_eq!(target.first(), None);
        assert_eq!(target.to_csv(), None);
    }

    #[test]
    fn test_node_target_single() {
        let target = NodeTarget::single("192.168.1.10");
        assert!(!target.is_default());
        assert!(target.is_single());
        assert!(!target.is_multiple());
        assert_eq!(target.nodes(), &["192.168.1.10".to_string()]);
        assert_eq!(target.first(), Some("192.168.1.10"));
        assert_eq!(target.to_csv(), Some("192.168.1.10".to_string()));
    }

    #[test]
    fn test_node_target_multiple() {
        let target = NodeTarget::multiple(vec!["192.168.1.10", "192.168.1.11"]);
        assert!(!target.is_default());
        assert!(!target.is_single());
        assert!(target.is_multiple());
        assert_eq!(
            target.nodes(),
            &["192.168.1.10".to_string(), "192.168.1.11".to_string()]
        );
        assert_eq!(target.first(), Some("192.168.1.10"));
        assert_eq!(
            target.to_csv(),
            Some("192.168.1.10,192.168.1.11".to_string())
        );
    }

    #[test]
    fn test_node_target_from_csv() {
        assert!(NodeTarget::from_csv("").is_default());
        assert_eq!(
            NodeTarget::from_csv("192.168.1.10"),
            NodeTarget::Single("192.168.1.10".to_string())
        );
        assert_eq!(
            NodeTarget::from_csv("192.168.1.10, 192.168.1.11"),
            NodeTarget::Multiple(vec!["192.168.1.10".to_string(), "192.168.1.11".to_string()])
        );
    }

    #[test]
    fn test_node_target_from_str() {
        let target: NodeTarget = "192.168.1.10".into();
        assert_eq!(target, NodeTarget::Single("192.168.1.10".to_string()));

        let target: NodeTarget = "192.168.1.10,192.168.1.11".into();
        assert!(target.is_multiple());
    }

    #[test]
    fn test_node_target_from_vec() {
        let target: NodeTarget = vec!["192.168.1.10".to_string()].into();
        assert!(target.is_single());

        let target: NodeTarget =
            vec!["192.168.1.10".to_string(), "192.168.1.11".to_string()].into();
        assert!(target.is_multiple());

        let target: NodeTarget = Vec::<String>::new().into();
        assert!(target.is_default());
    }

    #[test]
    fn test_apply_to_request() {
        let target = NodeTarget::single("192.168.1.10");
        let request = Request::new(());
        let request = target.apply_to_request(request);

        let metadata = request.metadata().get(NODE_METADATA_KEY);
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().to_str().unwrap(), "192.168.1.10");
    }

    #[test]
    fn test_apply_to_request_multiple() {
        let target = NodeTarget::multiple(vec!["10.0.0.1", "10.0.0.2"]);
        let request = Request::new(());
        let request = target.apply_to_request(request);

        let metadata = request.metadata().get(NODE_METADATA_KEY);
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().to_str().unwrap(), "10.0.0.1,10.0.0.2");
    }

    #[test]
    fn test_apply_to_request_default() {
        let target = NodeTarget::default();
        let request = Request::new(());
        let request = target.apply_to_request(request);

        let metadata = request.metadata().get(NODE_METADATA_KEY);
        assert!(metadata.is_none());
    }
}
