// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cluster discovery helpers for working with Talos clusters.
//!
//! This module provides utilities for discovering cluster members and their health status.
//!
//! # Example
//!
//! ```no_run
//! use talos_api_rs::client::discovery::ClusterDiscovery;
//! use talos_api_rs::client::TalosClientConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Discover cluster from a single known endpoint
//! let discovery = ClusterDiscovery::from_endpoint("https://192.168.1.100:50000")
//!     .with_ca_cert("/path/to/ca.crt")
//!     .with_client_cert("/path/to/client.crt", "/path/to/client.key")
//!     .build();
//!
//! let members = discovery.discover_members().await?;
//! for member in &members {
//!     println!("{}: {} ({:?})", member.name, member.endpoint, member.role);
//! }
//!
//! // Check health of all members
//! let health = discovery.check_cluster_health().await?;
//! println!("Healthy: {}/{}", health.healthy_count(), health.total_count());
//! # Ok(())
//! # }
//! ```

use crate::client::{TalosClient, TalosClientConfig};
use crate::error::Result;
use std::collections::HashMap;
use std::time::Duration;

/// Role of a node in the Talos cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeRole {
    /// Control plane node (runs etcd, API server, etc.)
    ControlPlane,
    /// Worker node (runs workloads only)
    Worker,
    /// Unknown role
    Unknown,
}

impl std::fmt::Display for NodeRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ControlPlane => write!(f, "controlplane"),
            Self::Worker => write!(f, "worker"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Information about a discovered cluster member.
#[derive(Debug, Clone)]
pub struct ClusterMember {
    /// Node name/hostname
    pub name: String,
    /// gRPC endpoint URL
    pub endpoint: String,
    /// Node role
    pub role: NodeRole,
    /// Whether this node is an etcd member
    pub is_etcd_member: bool,
}

impl ClusterMember {
    /// Create a new cluster member
    #[must_use]
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>, role: NodeRole) -> Self {
        Self {
            name: name.into(),
            endpoint: endpoint.into(),
            role,
            is_etcd_member: role == NodeRole::ControlPlane,
        }
    }

    /// Check if this is a control plane node
    #[must_use]
    pub fn is_control_plane(&self) -> bool {
        self.role == NodeRole::ControlPlane
    }

    /// Check if this is a worker node
    #[must_use]
    pub fn is_worker(&self) -> bool {
        self.role == NodeRole::Worker
    }
}

/// Health status of a single node.
#[derive(Debug, Clone)]
pub struct NodeHealth {
    /// Node name
    pub name: String,
    /// Node endpoint
    pub endpoint: String,
    /// Whether the node is reachable and healthy
    pub is_healthy: bool,
    /// Talos version if healthy
    pub version: Option<String>,
    /// Error message if unhealthy
    pub error: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
}

impl NodeHealth {
    /// Create a healthy node health status
    #[must_use]
    pub fn healthy(
        name: impl Into<String>,
        endpoint: impl Into<String>,
        version: impl Into<String>,
        response_time_ms: u64,
    ) -> Self {
        Self {
            name: name.into(),
            endpoint: endpoint.into(),
            is_healthy: true,
            version: Some(version.into()),
            error: None,
            response_time_ms: Some(response_time_ms),
        }
    }

    /// Create an unhealthy node health status
    #[must_use]
    pub fn unhealthy(
        name: impl Into<String>,
        endpoint: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            endpoint: endpoint.into(),
            is_healthy: false,
            version: None,
            error: Some(error.into()),
            response_time_ms: None,
        }
    }
}

/// Health status of the entire cluster.
#[derive(Debug, Clone)]
pub struct ClusterHealth {
    /// Health status of each node
    pub nodes: Vec<NodeHealth>,
    /// Overall cluster status
    pub is_healthy: bool,
}

impl ClusterHealth {
    /// Create cluster health from individual node health
    #[must_use]
    pub fn from_nodes(nodes: Vec<NodeHealth>) -> Self {
        let is_healthy = !nodes.is_empty() && nodes.iter().all(|n| n.is_healthy);
        Self { nodes, is_healthy }
    }

    /// Get the number of healthy nodes
    #[must_use]
    pub fn healthy_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_healthy).count()
    }

    /// Get the total number of nodes
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get unhealthy nodes
    #[must_use]
    pub fn unhealthy_nodes(&self) -> Vec<&NodeHealth> {
        self.nodes.iter().filter(|n| !n.is_healthy).collect()
    }

    /// Get healthy nodes
    #[must_use]
    pub fn healthy_nodes(&self) -> Vec<&NodeHealth> {
        self.nodes.iter().filter(|n| n.is_healthy).collect()
    }

    /// Get the average response time of healthy nodes
    #[must_use]
    pub fn avg_response_time_ms(&self) -> Option<u64> {
        let times: Vec<u64> = self
            .nodes
            .iter()
            .filter_map(|n| n.response_time_ms)
            .collect();

        if times.is_empty() {
            None
        } else {
            Some(times.iter().sum::<u64>() / times.len() as u64)
        }
    }
}

/// Builder for cluster discovery operations.
#[derive(Debug, Clone)]
pub struct ClusterDiscoveryBuilder {
    /// Initial endpoint to connect to
    endpoint: String,
    /// CA certificate path
    ca_cert: Option<String>,
    /// Client certificate path
    client_cert: Option<String>,
    /// Client key path
    client_key: Option<String>,
    /// Connection timeout
    connect_timeout: Duration,
    /// Request timeout for health checks
    request_timeout: Duration,
    /// Skip TLS verification
    insecure: bool,
}

impl ClusterDiscoveryBuilder {
    /// Create a new builder with the given endpoint
    #[must_use]
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            ca_cert: None,
            client_cert: None,
            client_key: None,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(5),
            insecure: false,
        }
    }

    /// Set CA certificate path
    #[must_use]
    pub fn with_ca_cert(mut self, path: impl Into<String>) -> Self {
        self.ca_cert = Some(path.into());
        self
    }

    /// Set client certificate and key paths
    #[must_use]
    pub fn with_client_cert(
        mut self,
        cert_path: impl Into<String>,
        key_path: impl Into<String>,
    ) -> Self {
        self.client_cert = Some(cert_path.into());
        self.client_key = Some(key_path.into());
        self
    }

    /// Set connection timeout
    #[must_use]
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set request timeout for health checks
    #[must_use]
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Skip TLS verification (insecure)
    #[must_use]
    pub fn insecure(mut self) -> Self {
        self.insecure = true;
        self
    }

    /// Build the cluster discovery instance
    #[must_use]
    pub fn build(self) -> ClusterDiscovery {
        ClusterDiscovery {
            endpoint: self.endpoint,
            ca_cert: self.ca_cert,
            client_cert: self.client_cert,
            client_key: self.client_key,
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
            insecure: self.insecure,
        }
    }
}

/// Cluster discovery helper for Talos clusters.
///
/// Provides utilities for discovering cluster members and checking their health.
#[derive(Debug, Clone)]
pub struct ClusterDiscovery {
    endpoint: String,
    ca_cert: Option<String>,
    client_cert: Option<String>,
    client_key: Option<String>,
    connect_timeout: Duration,
    request_timeout: Duration,
    insecure: bool,
}

impl ClusterDiscovery {
    /// Create a new discovery instance from an endpoint
    #[must_use]
    pub fn from_endpoint(endpoint: impl Into<String>) -> ClusterDiscoveryBuilder {
        ClusterDiscoveryBuilder::new(endpoint)
    }

    /// Create a client config for connecting to a specific endpoint
    fn create_config(&self, endpoint: &str) -> TalosClientConfig {
        let mut config = TalosClientConfig::new(endpoint)
            .with_connect_timeout(self.connect_timeout)
            .with_request_timeout(self.request_timeout);

        if let Some(ref ca) = self.ca_cert {
            config = config.with_ca(ca);
        }

        if let (Some(ref cert), Some(ref key)) = (&self.client_cert, &self.client_key) {
            config = config.with_client_cert(cert).with_client_key(key);
        }

        if self.insecure {
            config = config.insecure();
        }

        config
    }

    /// Connect to the primary endpoint and get a client
    async fn connect_primary(&self) -> Result<TalosClient> {
        let config = self.create_config(&self.endpoint);
        TalosClient::new(config).await
    }

    /// Discover cluster members via etcd member list
    ///
    /// This connects to the initial endpoint and queries the etcd member list
    /// to discover all control plane nodes.
    pub async fn discover_members(&self) -> Result<Vec<ClusterMember>> {
        let client = self.connect_primary().await?;

        // Use EtcdMemberList to discover control plane nodes
        let etcd_response = client
            .etcd_member_list(crate::resources::EtcdMemberListRequest::new())
            .await?;

        let mut members = Vec::new();

        for result in &etcd_response.results {
            for member in &result.members {
                // Extract endpoint from member's client URLs
                let endpoint = member
                    .client_urls
                    .first()
                    .map(|url| {
                        // Convert etcd client URL to Talos API endpoint
                        // etcd typically uses port 2379, Talos API uses 50000
                        url.replace(":2379", ":50000")
                            .replace("http://", "https://")
                    })
                    .unwrap_or_else(|| self.endpoint.clone());

                members.push(ClusterMember {
                    name: member.hostname.clone(),
                    endpoint,
                    role: NodeRole::ControlPlane,
                    is_etcd_member: true,
                });
            }
        }

        Ok(members)
    }

    /// Check health of a single endpoint
    ///
    /// Tries the Version API first, falls back to Hostname API if unavailable.
    /// This is necessary because Docker-based Talos clusters don't implement
    /// the Version API.
    async fn check_endpoint_health(&self, name: &str, endpoint: &str) -> NodeHealth {
        let config = self.create_config(endpoint);
        let start = std::time::Instant::now();

        match TalosClient::new(config).await {
            Ok(client) => {
                // Try Version API first
                let mut version_client = client.version();
                let version_req = crate::api::version::VersionRequest { client: false };

                match version_client.version(version_req).await {
                    Ok(response) => {
                        let elapsed = start.elapsed().as_millis() as u64;
                        NodeHealth::healthy(name, endpoint, &response.get_ref().tag, elapsed)
                    }
                    Err(version_err) => {
                        // Version API failed - try Hostname API as fallback
                        // This is common in Docker-based clusters where Version is unimplemented
                        let mut machine_client = client.machine();
                        match machine_client.hostname(()).await {
                            Ok(response) => {
                                let elapsed = start.elapsed().as_millis() as u64;
                                // Extract hostname from response
                                let hostname = response
                                    .get_ref()
                                    .messages
                                    .first()
                                    .map(|m| m.hostname.as_str())
                                    .unwrap_or("unknown");
                                // Mark as healthy with hostname instead of version
                                NodeHealth::healthy(
                                    name,
                                    endpoint,
                                    format!("(hostname: {})", hostname),
                                    elapsed,
                                )
                            }
                            Err(_) => {
                                // Both APIs failed - report the version error
                                NodeHealth::unhealthy(name, endpoint, version_err.to_string())
                            }
                        }
                    }
                }
            }
            Err(e) => NodeHealth::unhealthy(name, endpoint, e.to_string()),
        }
    }

    /// Check health of all discovered cluster members
    ///
    /// This first discovers members, then checks health of each one.
    pub async fn check_cluster_health(&self) -> Result<ClusterHealth> {
        let members = self.discover_members().await?;
        self.check_members_health(&members).await
    }

    /// Check health of specific cluster members
    ///
    /// Useful when you already have a list of members.
    pub async fn check_members_health(&self, members: &[ClusterMember]) -> Result<ClusterHealth> {
        let mut health_results = Vec::with_capacity(members.len());

        for member in members {
            let health = self
                .check_endpoint_health(&member.name, &member.endpoint)
                .await;
            health_results.push(health);
        }

        Ok(ClusterHealth::from_nodes(health_results))
    }

    /// Check health of multiple endpoints directly
    ///
    /// Useful when you have a list of endpoint URLs but not member info.
    pub async fn check_endpoints_health(&self, endpoints: &[String]) -> Result<ClusterHealth> {
        let mut health_results = Vec::with_capacity(endpoints.len());

        for endpoint in endpoints {
            let health = self.check_endpoint_health(endpoint, endpoint).await;
            health_results.push(health);
        }

        Ok(ClusterHealth::from_nodes(health_results))
    }

    /// Get a map of endpoint to version for healthy nodes
    pub async fn get_cluster_versions(&self) -> Result<HashMap<String, String>> {
        let health = self.check_cluster_health().await?;

        Ok(health
            .nodes
            .into_iter()
            .filter_map(|n| n.version.map(|v| (n.endpoint, v)))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_role_display() {
        assert_eq!(format!("{}", NodeRole::ControlPlane), "controlplane");
        assert_eq!(format!("{}", NodeRole::Worker), "worker");
        assert_eq!(format!("{}", NodeRole::Unknown), "unknown");
    }

    #[test]
    fn test_cluster_member_new() {
        let member = ClusterMember::new(
            "node1",
            "https://192.168.1.100:50000",
            NodeRole::ControlPlane,
        );
        assert_eq!(member.name, "node1");
        assert_eq!(member.endpoint, "https://192.168.1.100:50000");
        assert!(member.is_control_plane());
        assert!(!member.is_worker());
        assert!(member.is_etcd_member);
    }

    #[test]
    fn test_cluster_member_worker() {
        let member = ClusterMember::new("worker1", "https://192.168.1.200:50000", NodeRole::Worker);
        assert!(!member.is_control_plane());
        assert!(member.is_worker());
        assert!(!member.is_etcd_member);
    }

    #[test]
    fn test_node_health_healthy() {
        let health = NodeHealth::healthy("node1", "https://192.168.1.100:50000", "v1.9.0", 42);
        assert!(health.is_healthy);
        assert_eq!(health.version, Some("v1.9.0".to_string()));
        assert_eq!(health.response_time_ms, Some(42));
        assert!(health.error.is_none());
    }

    #[test]
    fn test_node_health_unhealthy() {
        let health =
            NodeHealth::unhealthy("node1", "https://192.168.1.100:50000", "connection refused");
        assert!(!health.is_healthy);
        assert!(health.version.is_none());
        assert!(health.response_time_ms.is_none());
        assert_eq!(health.error, Some("connection refused".to_string()));
    }

    #[test]
    fn test_cluster_health_all_healthy() {
        let nodes = vec![
            NodeHealth::healthy("node1", "endpoint1", "v1.9.0", 10),
            NodeHealth::healthy("node2", "endpoint2", "v1.9.0", 20),
        ];
        let health = ClusterHealth::from_nodes(nodes);

        assert!(health.is_healthy);
        assert_eq!(health.healthy_count(), 2);
        assert_eq!(health.total_count(), 2);
        assert_eq!(health.unhealthy_nodes().len(), 0);
    }

    #[test]
    fn test_cluster_health_partial_healthy() {
        let nodes = vec![
            NodeHealth::healthy("node1", "endpoint1", "v1.9.0", 10),
            NodeHealth::unhealthy("node2", "endpoint2", "timeout"),
        ];
        let health = ClusterHealth::from_nodes(nodes);

        assert!(!health.is_healthy);
        assert_eq!(health.healthy_count(), 1);
        assert_eq!(health.total_count(), 2);
        assert_eq!(health.unhealthy_nodes().len(), 1);
    }

    #[test]
    fn test_cluster_health_avg_response_time() {
        let nodes = vec![
            NodeHealth::healthy("node1", "endpoint1", "v1.9.0", 10),
            NodeHealth::healthy("node2", "endpoint2", "v1.9.0", 20),
            NodeHealth::healthy("node3", "endpoint3", "v1.9.0", 30),
        ];
        let health = ClusterHealth::from_nodes(nodes);

        assert_eq!(health.avg_response_time_ms(), Some(20));
    }

    #[test]
    fn test_cluster_health_no_response_time() {
        let nodes = vec![NodeHealth::unhealthy("node1", "endpoint1", "error")];
        let health = ClusterHealth::from_nodes(nodes);

        assert_eq!(health.avg_response_time_ms(), None);
    }

    #[test]
    fn test_cluster_discovery_builder() {
        let discovery = ClusterDiscovery::from_endpoint("https://192.168.1.100:50000")
            .with_ca_cert("/path/to/ca.crt")
            .with_client_cert("/path/to/client.crt", "/path/to/client.key")
            .with_connect_timeout(Duration::from_secs(5))
            .with_request_timeout(Duration::from_secs(3))
            .build();

        assert_eq!(discovery.endpoint, "https://192.168.1.100:50000");
        assert_eq!(discovery.ca_cert, Some("/path/to/ca.crt".to_string()));
        assert_eq!(
            discovery.client_cert,
            Some("/path/to/client.crt".to_string())
        );
        assert_eq!(
            discovery.client_key,
            Some("/path/to/client.key".to_string())
        );
        assert_eq!(discovery.connect_timeout, Duration::from_secs(5));
        assert_eq!(discovery.request_timeout, Duration::from_secs(3));
        assert!(!discovery.insecure);
    }

    #[test]
    fn test_cluster_discovery_builder_insecure() {
        let discovery = ClusterDiscovery::from_endpoint("https://192.168.1.100:50000")
            .insecure()
            .build();

        assert!(discovery.insecure);
    }
}
