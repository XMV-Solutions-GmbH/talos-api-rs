// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for Container Image APIs.
//!
//! Provides functionality to list and pull container images in the CRI.
//!
//! # Example
//!
//! ```no_run
//! use talos_api_rs::{ImageListRequest, ImagePullRequest, ContainerdNamespace};
//!
//! // List all images in the system namespace
//! let list_req = ImageListRequest::new(ContainerdNamespace::System);
//!
//! // Pull a specific image into the CRI namespace
//! let pull_req = ImagePullRequest::new("ghcr.io/siderolabs/kubelet:v1.30.0")
//!     .with_namespace(ContainerdNamespace::Cri);
//! ```

use crate::api::generated::common::ContainerdNamespace as ProtoContainerdNamespace;
use crate::api::generated::machine::{
    ImageListRequest as ProtoImageListRequest, ImageListResponse as ProtoImageListResponse,
    ImagePull as ProtoImagePull, ImagePullRequest as ProtoImagePullRequest,
    ImagePullResponse as ProtoImagePullResponse,
};

// =============================================================================
// ContainerdNamespace
// =============================================================================

/// Containerd namespace for image operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContainerdNamespace {
    /// Unknown namespace.
    Unknown,
    /// System namespace (talos system containers).
    #[default]
    System,
    /// CRI namespace (Kubernetes workloads).
    Cri,
}

impl ContainerdNamespace {
    /// Convert to the protobuf enum value.
    #[must_use]
    pub fn as_proto_i32(&self) -> i32 {
        match self {
            Self::Unknown => ProtoContainerdNamespace::NsUnknown as i32,
            Self::System => ProtoContainerdNamespace::NsSystem as i32,
            Self::Cri => ProtoContainerdNamespace::NsCri as i32,
        }
    }
}

impl From<i32> for ContainerdNamespace {
    fn from(value: i32) -> Self {
        match value {
            x if x == ProtoContainerdNamespace::NsSystem as i32 => Self::System,
            x if x == ProtoContainerdNamespace::NsCri as i32 => Self::Cri,
            _ => Self::Unknown,
        }
    }
}

// =============================================================================
// ImageListRequest
// =============================================================================

/// Request to list container images.
#[derive(Debug, Clone, Default)]
pub struct ImageListRequest {
    /// Containerd namespace to list images from.
    pub namespace: ContainerdNamespace,
}

impl ImageListRequest {
    /// Create a new request to list images in a specific namespace.
    #[must_use]
    pub fn new(namespace: ContainerdNamespace) -> Self {
        Self { namespace }
    }

    /// Create a request to list system images.
    #[must_use]
    pub fn system() -> Self {
        Self::new(ContainerdNamespace::System)
    }

    /// Create a request to list CRI images (Kubernetes workloads).
    #[must_use]
    pub fn cri() -> Self {
        Self::new(ContainerdNamespace::Cri)
    }
}

impl From<ImageListRequest> for ProtoImageListRequest {
    fn from(req: ImageListRequest) -> Self {
        Self {
            namespace: req.namespace.as_proto_i32(),
        }
    }
}

// =============================================================================
// ImageInfo
// =============================================================================

/// Information about a container image.
#[derive(Debug, Clone)]
pub struct ImageInfo {
    /// Node that reported this image.
    pub node: Option<String>,
    /// Full image name (repository:tag or repository@digest).
    pub name: String,
    /// Image digest (sha256:...).
    pub digest: String,
    /// Image size in bytes.
    pub size: i64,
    /// When the image was created.
    pub created_at: Option<prost_types::Timestamp>,
}

impl ImageInfo {
    /// Get image size in megabytes.
    #[must_use]
    pub fn size_mb(&self) -> f64 {
        self.size as f64 / 1_048_576.0
    }

    /// Get image size in a human-readable format.
    #[must_use]
    pub fn size_human(&self) -> String {
        let size = self.size as f64;
        if size < 1024.0 {
            format!("{:.0} B", size)
        } else if size < 1_048_576.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1_073_741_824.0 {
            format!("{:.1} MB", size / 1_048_576.0)
        } else {
            format!("{:.2} GB", size / 1_073_741_824.0)
        }
    }

    /// Check if this is a digest-based reference (no tag).
    #[must_use]
    pub fn is_digest_reference(&self) -> bool {
        self.name.contains('@')
    }

    /// Extract the repository name (without tag or digest).
    #[must_use]
    pub fn repository(&self) -> &str {
        if let Some(pos) = self.name.find('@') {
            &self.name[..pos]
        } else if let Some(pos) = self.name.rfind(':') {
            // Be careful not to split on port numbers
            let before_colon = &self.name[..pos];
            if before_colon.contains('/') || !before_colon.contains('.') {
                &self.name[..pos]
            } else {
                &self.name
            }
        } else {
            &self.name
        }
    }

    /// Extract the tag (if present).
    #[must_use]
    pub fn tag(&self) -> Option<&str> {
        if self.name.contains('@') {
            return None;
        }
        if let Some(pos) = self.name.rfind(':') {
            let before_colon = &self.name[..pos];
            // Make sure it's not a port number
            if before_colon.contains('/') || !before_colon.contains('.') {
                return Some(&self.name[pos + 1..]);
            }
        }
        None
    }
}

impl From<ProtoImageListResponse> for ImageInfo {
    fn from(proto: ProtoImageListResponse) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            name: proto.name,
            digest: proto.digest,
            size: proto.size,
            created_at: proto.created_at,
        }
    }
}

// =============================================================================
// ImagePullRequest
// =============================================================================

/// Request to pull a container image.
#[derive(Debug, Clone)]
pub struct ImagePullRequest {
    /// Containerd namespace to pull the image into.
    pub namespace: ContainerdNamespace,
    /// Image reference to pull (e.g., "docker.io/library/nginx:latest").
    pub reference: String,
}

impl ImagePullRequest {
    /// Create a new request to pull an image.
    ///
    /// Uses the system namespace by default.
    #[must_use]
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            namespace: ContainerdNamespace::System,
            reference: reference.into(),
        }
    }

    /// Set the namespace to pull the image into.
    #[must_use]
    pub fn with_namespace(mut self, namespace: ContainerdNamespace) -> Self {
        self.namespace = namespace;
        self
    }

    /// Pull into the CRI namespace (Kubernetes workloads).
    #[must_use]
    pub fn for_cri(mut self) -> Self {
        self.namespace = ContainerdNamespace::Cri;
        self
    }
}

impl From<ImagePullRequest> for ProtoImagePullRequest {
    fn from(req: ImagePullRequest) -> Self {
        Self {
            namespace: req.namespace.as_proto_i32(),
            reference: req.reference,
        }
    }
}

// =============================================================================
// ImagePullResult
// =============================================================================

/// Result from pulling an image.
#[derive(Debug, Clone)]
pub struct ImagePullResult {
    /// Node that processed the pull request.
    pub node: Option<String>,
}

impl From<ProtoImagePull> for ImagePullResult {
    fn from(proto: ProtoImagePull) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
        }
    }
}

/// Response from pulling an image (may contain multiple node results).
#[derive(Debug, Clone)]
pub struct ImagePullResponse {
    /// Results from each node.
    pub results: Vec<ImagePullResult>,
}

impl ImagePullResponse {
    /// Check if the pull succeeded on all nodes.
    #[must_use]
    pub fn all_succeeded(&self) -> bool {
        !self.results.is_empty()
    }

    /// Get the list of nodes that processed the request.
    #[must_use]
    pub fn nodes(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter_map(|r| r.node.as_deref())
            .collect()
    }
}

impl From<ProtoImagePullResponse> for ImagePullResponse {
    fn from(proto: ProtoImagePullResponse) -> Self {
        Self {
            results: proto.messages.into_iter().map(Into::into).collect(),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_containerd_namespace_default() {
        let ns = ContainerdNamespace::default();
        assert_eq!(ns, ContainerdNamespace::System);
    }

    #[test]
    fn test_containerd_namespace_from_i32() {
        assert_eq!(ContainerdNamespace::from(0), ContainerdNamespace::Unknown);
        assert_eq!(ContainerdNamespace::from(1), ContainerdNamespace::System);
        assert_eq!(ContainerdNamespace::from(2), ContainerdNamespace::Cri);
        assert_eq!(ContainerdNamespace::from(999), ContainerdNamespace::Unknown);
    }

    #[test]
    fn test_image_list_request_constructors() {
        let req = ImageListRequest::system();
        assert_eq!(req.namespace, ContainerdNamespace::System);

        let req = ImageListRequest::cri();
        assert_eq!(req.namespace, ContainerdNamespace::Cri);
    }

    #[test]
    fn test_image_list_request_to_proto() {
        let req = ImageListRequest::cri();
        let proto: ProtoImageListRequest = req.into();
        assert_eq!(proto.namespace, ProtoContainerdNamespace::NsCri as i32);
    }

    #[test]
    fn test_image_info_size_human() {
        let info = ImageInfo {
            node: None,
            name: "test".to_string(),
            digest: "sha256:abc".to_string(),
            size: 500,
            created_at: None,
        };
        assert_eq!(info.size_human(), "500 B");

        let info = ImageInfo {
            size: 2048,
            ..info.clone()
        };
        assert_eq!(info.size_human(), "2.0 KB");

        let info = ImageInfo {
            size: 52_428_800,
            ..info.clone()
        };
        assert_eq!(info.size_human(), "50.0 MB");

        let info = ImageInfo {
            size: 2_147_483_648,
            ..info
        };
        assert_eq!(info.size_human(), "2.00 GB");
    }

    #[test]
    fn test_image_info_repository_and_tag() {
        // Standard image with tag
        let info = ImageInfo {
            node: None,
            name: "docker.io/library/nginx:1.25".to_string(),
            digest: "sha256:abc".to_string(),
            size: 0,
            created_at: None,
        };
        assert_eq!(info.repository(), "docker.io/library/nginx");
        assert_eq!(info.tag(), Some("1.25"));
        assert!(!info.is_digest_reference());

        // Image with digest reference
        let info = ImageInfo {
            name: "ghcr.io/siderolabs/kubelet@sha256:abc123".to_string(),
            ..info.clone()
        };
        assert_eq!(info.repository(), "ghcr.io/siderolabs/kubelet");
        assert_eq!(info.tag(), None);
        assert!(info.is_digest_reference());

        // Image without tag (implicit :latest)
        let info = ImageInfo {
            name: "nginx".to_string(),
            ..info
        };
        assert_eq!(info.repository(), "nginx");
        assert_eq!(info.tag(), None);
    }

    #[test]
    fn test_image_pull_request_builder() {
        let req = ImagePullRequest::new("nginx:latest").for_cri();
        assert_eq!(req.reference, "nginx:latest");
        assert_eq!(req.namespace, ContainerdNamespace::Cri);

        let req = ImagePullRequest::new("alpine:3.18").with_namespace(ContainerdNamespace::Cri);
        assert_eq!(req.namespace, ContainerdNamespace::Cri);
    }

    #[test]
    fn test_image_pull_request_to_proto() {
        let req = ImagePullRequest::new("ghcr.io/test/image:v1").for_cri();
        let proto: ProtoImagePullRequest = req.into();
        assert_eq!(proto.reference, "ghcr.io/test/image:v1");
        assert_eq!(proto.namespace, ProtoContainerdNamespace::NsCri as i32);
    }

    #[test]
    fn test_image_pull_response_nodes() {
        let response = ImagePullResponse {
            results: vec![
                ImagePullResult {
                    node: Some("node1".to_string()),
                },
                ImagePullResult {
                    node: Some("node2".to_string()),
                },
                ImagePullResult { node: None },
            ],
        };
        assert!(response.all_succeeded());
        assert_eq!(response.nodes(), vec!["node1", "node2"]);
    }
}
