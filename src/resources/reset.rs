// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for reset operations.
//!
//! The Reset API is used to reset/wipe a Talos node. This is typically used
//! for destroying clusters or removing nodes from a cluster.

use crate::api::machine::{
    reset_request::WipeMode as ProtoWipeMode, Reset as ProtoReset,
    ResetPartitionSpec as ProtoPartitionSpec, ResetRequest as ProtoRequest,
    ResetResponse as ProtoResponse,
};

/// Mode for wiping disks during reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WipeMode {
    /// Wipe all disks (system and user).
    #[default]
    All,
    /// Wipe only the system disk.
    SystemDisk,
    /// Wipe only user disks.
    UserDisks,
}

impl From<WipeMode> for i32 {
    fn from(mode: WipeMode) -> Self {
        match mode {
            WipeMode::All => ProtoWipeMode::All as i32,
            WipeMode::SystemDisk => ProtoWipeMode::SystemDisk as i32,
            WipeMode::UserDisks => ProtoWipeMode::UserDisks as i32,
        }
    }
}

impl From<i32> for WipeMode {
    fn from(value: i32) -> Self {
        match value {
            0 => WipeMode::All,
            1 => WipeMode::SystemDisk,
            2 => WipeMode::UserDisks,
            _ => WipeMode::All,
        }
    }
}

impl std::fmt::Display for WipeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WipeMode::All => write!(f, "all"),
            WipeMode::SystemDisk => write!(f, "system-disk"),
            WipeMode::UserDisks => write!(f, "user-disks"),
        }
    }
}

/// Specification for a partition to wipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetPartitionSpec {
    /// Partition label.
    pub label: String,
    /// Whether to wipe the partition.
    pub wipe: bool,
}

impl ResetPartitionSpec {
    /// Create a new partition spec.
    #[must_use]
    pub fn new(label: impl Into<String>, wipe: bool) -> Self {
        Self {
            label: label.into(),
            wipe,
        }
    }

    /// Create a partition spec that will be wiped.
    #[must_use]
    pub fn wipe(label: impl Into<String>) -> Self {
        Self::new(label, true)
    }
}

impl From<ResetPartitionSpec> for ProtoPartitionSpec {
    fn from(spec: ResetPartitionSpec) -> Self {
        ProtoPartitionSpec {
            label: spec.label,
            wipe: spec.wipe,
        }
    }
}

/// Request to reset a Talos node.
///
/// # Example
///
/// ```no_run
/// use talos_api_rs::resources::{ResetRequest, WipeMode};
///
/// // Graceful reset with reboot (recommended for cluster nodes)
/// let request = ResetRequest::graceful();
///
/// // Quick reset without etcd leave (for standalone nodes)
/// let request = ResetRequest::builder()
///     .graceful(false)
///     .reboot(true)
///     .wipe_mode(WipeMode::SystemDisk)
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct ResetRequest {
    /// If true, node will gracefully leave etcd before reset.
    pub graceful: bool,
    /// If true, node will reboot after reset (otherwise halt).
    pub reboot: bool,
    /// Specific system partitions to wipe.
    pub system_partitions_to_wipe: Vec<ResetPartitionSpec>,
    /// Specific user disks to wipe.
    pub user_disks_to_wipe: Vec<String>,
    /// Wipe mode (all, system-disk, user-disks).
    pub mode: WipeMode,
}

impl ResetRequest {
    /// Create a new builder for customizing the reset request.
    #[must_use]
    pub fn builder() -> ResetRequestBuilder {
        ResetRequestBuilder::default()
    }

    /// Create a graceful reset request.
    ///
    /// This will:
    /// - Gracefully leave etcd (if control-plane)
    /// - Reboot after reset
    /// - Wipe all disks
    #[must_use]
    pub fn graceful() -> Self {
        Self {
            graceful: true,
            reboot: true,
            mode: WipeMode::All,
            ..Default::default()
        }
    }

    /// Create a forceful reset request (no etcd leave).
    ///
    /// This will:
    /// - Skip etcd graceful leave
    /// - Reboot after reset
    /// - Wipe all disks
    #[must_use]
    pub fn force() -> Self {
        Self {
            graceful: false,
            reboot: true,
            mode: WipeMode::All,
            ..Default::default()
        }
    }

    /// Create a halt request (reset without reboot).
    #[must_use]
    pub fn halt() -> Self {
        Self {
            graceful: true,
            reboot: false,
            mode: WipeMode::All,
            ..Default::default()
        }
    }
}

impl From<ResetRequest> for ProtoRequest {
    fn from(req: ResetRequest) -> Self {
        ProtoRequest {
            graceful: req.graceful,
            reboot: req.reboot,
            system_partitions_to_wipe: req
                .system_partitions_to_wipe
                .into_iter()
                .map(Into::into)
                .collect(),
            user_disks_to_wipe: req.user_disks_to_wipe,
            mode: req.mode.into(),
        }
    }
}

/// Builder for `ResetRequest`.
#[derive(Debug, Clone, Default)]
pub struct ResetRequestBuilder {
    graceful: bool,
    reboot: bool,
    system_partitions_to_wipe: Vec<ResetPartitionSpec>,
    user_disks_to_wipe: Vec<String>,
    mode: WipeMode,
}

impl ResetRequestBuilder {
    /// Set whether to gracefully leave etcd.
    #[must_use]
    pub fn graceful(mut self, graceful: bool) -> Self {
        self.graceful = graceful;
        self
    }

    /// Set whether to reboot after reset.
    #[must_use]
    pub fn reboot(mut self, reboot: bool) -> Self {
        self.reboot = reboot;
        self
    }

    /// Set the wipe mode.
    #[must_use]
    pub fn wipe_mode(mut self, mode: WipeMode) -> Self {
        self.mode = mode;
        self
    }

    /// Add a system partition to wipe.
    #[must_use]
    pub fn wipe_partition(mut self, spec: ResetPartitionSpec) -> Self {
        self.system_partitions_to_wipe.push(spec);
        self
    }

    /// Add a user disk to wipe.
    #[must_use]
    pub fn wipe_user_disk(mut self, disk: impl Into<String>) -> Self {
        self.user_disks_to_wipe.push(disk.into());
        self
    }

    /// Build the reset request.
    #[must_use]
    pub fn build(self) -> ResetRequest {
        ResetRequest {
            graceful: self.graceful,
            reboot: self.reboot,
            system_partitions_to_wipe: self.system_partitions_to_wipe,
            user_disks_to_wipe: self.user_disks_to_wipe,
            mode: self.mode,
        }
    }
}

/// Result of a reset operation for a single node.
#[derive(Debug, Clone)]
pub struct ResetResult {
    /// Node hostname (if available from metadata).
    pub node: Option<String>,
    /// Actor ID that initiated the reset.
    pub actor_id: String,
}

impl From<ProtoReset> for ResetResult {
    fn from(proto: ProtoReset) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            actor_id: proto.actor_id,
        }
    }
}

/// Response from a reset operation.
#[derive(Debug, Clone)]
pub struct ResetResponse {
    /// Results from each node.
    pub results: Vec<ResetResult>,
}

impl From<ProtoResponse> for ResetResponse {
    fn from(proto: ProtoResponse) -> Self {
        Self {
            results: proto.messages.into_iter().map(Into::into).collect(),
        }
    }
}

impl ResetResponse {
    /// Check if the reset was initiated successfully.
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.results.is_empty()
    }

    /// Get the first result (useful for single-node operations).
    #[must_use]
    pub fn first(&self) -> Option<&ResetResult> {
        self.results.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wipe_mode_conversion() {
        assert_eq!(i32::from(WipeMode::All), 0);
        assert_eq!(i32::from(WipeMode::SystemDisk), 1);
        assert_eq!(i32::from(WipeMode::UserDisks), 2);

        assert_eq!(WipeMode::from(0), WipeMode::All);
        assert_eq!(WipeMode::from(1), WipeMode::SystemDisk);
        assert_eq!(WipeMode::from(2), WipeMode::UserDisks);
    }

    #[test]
    fn test_wipe_mode_display() {
        assert_eq!(WipeMode::All.to_string(), "all");
        assert_eq!(WipeMode::SystemDisk.to_string(), "system-disk");
        assert_eq!(WipeMode::UserDisks.to_string(), "user-disks");
    }

    #[test]
    fn test_reset_request_graceful() {
        let request = ResetRequest::graceful();
        assert!(request.graceful);
        assert!(request.reboot);
        assert_eq!(request.mode, WipeMode::All);
    }

    #[test]
    fn test_reset_request_force() {
        let request = ResetRequest::force();
        assert!(!request.graceful);
        assert!(request.reboot);
        assert_eq!(request.mode, WipeMode::All);
    }

    #[test]
    fn test_reset_request_halt() {
        let request = ResetRequest::halt();
        assert!(request.graceful);
        assert!(!request.reboot);
    }

    #[test]
    fn test_reset_request_builder() {
        let request = ResetRequest::builder()
            .graceful(true)
            .reboot(false)
            .wipe_mode(WipeMode::SystemDisk)
            .wipe_partition(ResetPartitionSpec::wipe("STATE"))
            .wipe_user_disk("/dev/sdb")
            .build();

        assert!(request.graceful);
        assert!(!request.reboot);
        assert_eq!(request.mode, WipeMode::SystemDisk);
        assert_eq!(request.system_partitions_to_wipe.len(), 1);
        assert_eq!(request.user_disks_to_wipe.len(), 1);
    }

    #[test]
    fn test_proto_conversion() {
        let request = ResetRequest::builder()
            .graceful(true)
            .reboot(true)
            .wipe_mode(WipeMode::UserDisks)
            .build();

        let proto: ProtoRequest = request.into();
        assert!(proto.graceful);
        assert!(proto.reboot);
        assert_eq!(proto.mode, ProtoWipeMode::UserDisks as i32);
    }

    #[test]
    fn test_partition_spec() {
        let spec = ResetPartitionSpec::wipe("STATE");
        assert_eq!(spec.label, "STATE");
        assert!(spec.wipe);

        let spec2 = ResetPartitionSpec::new("EPHEMERAL", false);
        assert_eq!(spec2.label, "EPHEMERAL");
        assert!(!spec2.wipe);
    }

    #[test]
    fn test_reset_response_is_success() {
        let response = ResetResponse {
            results: vec![ResetResult {
                node: Some("node1".to_string()),
                actor_id: "actor-123".to_string(),
            }],
        };
        assert!(response.is_success());

        let empty = ResetResponse { results: vec![] };
        assert!(!empty.is_success());
    }
}
