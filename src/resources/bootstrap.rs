// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for bootstrap operations.
//!
//! Bootstrap initializes the etcd cluster on the first control-plane node.
//! This operation should only be called ONCE per cluster.

use crate::api::machine::{
    Bootstrap as ProtoBootstrap, BootstrapRequest as ProtoRequest,
    BootstrapResponse as ProtoResponse,
};

/// Request to bootstrap the etcd cluster.
///
/// Bootstrap initializes the etcd cluster on the first control-plane node.
/// This should only be called ONCE when creating a new cluster.
///
/// # Example
///
/// ```no_run
/// use talos_api_rs::resources::BootstrapRequest;
///
/// // Standard bootstrap (new cluster)
/// let request = BootstrapRequest::new();
///
/// // Recovery from etcd snapshot
/// let recovery_request = BootstrapRequest::builder()
///     .recover_etcd(true)
///     .build();
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct BootstrapRequest {
    /// Enable etcd recovery from a snapshot.
    /// The snapshot must be uploaded via `EtcdRecover` RPC before calling bootstrap.
    pub recover_etcd: bool,
    /// Skip hash verification on the etcd snapshot.
    /// Enable this when recovering from a data directory copy.
    pub recover_skip_hash_check: bool,
}

impl BootstrapRequest {
    /// Create a new standard bootstrap request (no recovery).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for customizing the bootstrap request.
    #[must_use]
    pub fn builder() -> BootstrapRequestBuilder {
        BootstrapRequestBuilder::default()
    }

    /// Create a recovery bootstrap request.
    ///
    /// Use this when restoring from an etcd snapshot.
    #[must_use]
    pub fn recovery() -> Self {
        Self {
            recover_etcd: true,
            recover_skip_hash_check: false,
        }
    }

    /// Create a recovery bootstrap request that skips hash verification.
    ///
    /// Use this when recovering from a data directory copy.
    #[must_use]
    pub fn recovery_skip_hash() -> Self {
        Self {
            recover_etcd: true,
            recover_skip_hash_check: true,
        }
    }
}

impl From<BootstrapRequest> for ProtoRequest {
    fn from(req: BootstrapRequest) -> Self {
        ProtoRequest {
            recover_etcd: req.recover_etcd,
            recover_skip_hash_check: req.recover_skip_hash_check,
        }
    }
}

/// Builder for `BootstrapRequest`.
#[derive(Debug, Clone, Copy, Default)]
pub struct BootstrapRequestBuilder {
    recover_etcd: bool,
    recover_skip_hash_check: bool,
}

impl BootstrapRequestBuilder {
    /// Enable etcd recovery from a snapshot.
    #[must_use]
    pub fn recover_etcd(mut self, recover: bool) -> Self {
        self.recover_etcd = recover;
        self
    }

    /// Skip hash verification on the etcd snapshot.
    #[must_use]
    pub fn recover_skip_hash_check(mut self, skip: bool) -> Self {
        self.recover_skip_hash_check = skip;
        self
    }

    /// Build the bootstrap request.
    #[must_use]
    pub fn build(self) -> BootstrapRequest {
        BootstrapRequest {
            recover_etcd: self.recover_etcd,
            recover_skip_hash_check: self.recover_skip_hash_check,
        }
    }
}

/// Result of a bootstrap operation for a single node.
#[derive(Debug, Clone)]
pub struct BootstrapResult {
    /// Node hostname (if available from metadata)
    pub node: Option<String>,
}

impl From<ProtoBootstrap> for BootstrapResult {
    fn from(proto: ProtoBootstrap) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
        }
    }
}

/// Response from a bootstrap operation.
#[derive(Debug, Clone)]
pub struct BootstrapResponse {
    /// Results from each node (typically just one for bootstrap)
    pub results: Vec<BootstrapResult>,
}

impl From<ProtoResponse> for BootstrapResponse {
    fn from(proto: ProtoResponse) -> Self {
        Self {
            results: proto.messages.into_iter().map(Into::into).collect(),
        }
    }
}

impl BootstrapResponse {
    /// Check if the bootstrap succeeded.
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.results.is_empty()
    }

    /// Get the first result (useful for single-node operations).
    #[must_use]
    pub fn first(&self) -> Option<&BootstrapResult> {
        self.results.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_request_new() {
        let request = BootstrapRequest::new();
        assert!(!request.recover_etcd);
        assert!(!request.recover_skip_hash_check);
    }

    #[test]
    fn test_bootstrap_request_recovery() {
        let request = BootstrapRequest::recovery();
        assert!(request.recover_etcd);
        assert!(!request.recover_skip_hash_check);
    }

    #[test]
    fn test_bootstrap_request_recovery_skip_hash() {
        let request = BootstrapRequest::recovery_skip_hash();
        assert!(request.recover_etcd);
        assert!(request.recover_skip_hash_check);
    }

    #[test]
    fn test_bootstrap_request_builder() {
        let request = BootstrapRequest::builder()
            .recover_etcd(true)
            .recover_skip_hash_check(true)
            .build();

        assert!(request.recover_etcd);
        assert!(request.recover_skip_hash_check);
    }

    #[test]
    fn test_proto_conversion() {
        let request = BootstrapRequest::builder()
            .recover_etcd(true)
            .recover_skip_hash_check(false)
            .build();

        let proto: ProtoRequest = request.into();
        assert!(proto.recover_etcd);
        assert!(!proto.recover_skip_hash_check);
    }

    #[test]
    fn test_bootstrap_response_is_success() {
        let response = BootstrapResponse {
            results: vec![BootstrapResult {
                node: Some("controlplane-1".to_string()),
            }],
        };
        assert!(response.is_success());

        let empty_response = BootstrapResponse { results: vec![] };
        assert!(!empty_response.is_success());
    }
}
