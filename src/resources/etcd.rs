// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for etcd operations.
//!
//! This module provides high-level types for interacting with the etcd cluster
//! running on Talos control plane nodes.

use crate::api::generated::machine::{
    EtcdAlarm as ProtoEtcdAlarm, EtcdAlarmDisarm as ProtoEtcdAlarmDisarm,
    EtcdAlarmDisarmResponse as ProtoEtcdAlarmDisarmResponse,
    EtcdAlarmListResponse as ProtoEtcdAlarmListResponse,
    EtcdDefragment as ProtoEtcdDefragment, EtcdDefragmentResponse as ProtoEtcdDefragmentResponse,
    EtcdForfeitLeadership as ProtoEtcdForfeitLeadership,
    EtcdForfeitLeadershipRequest as ProtoEtcdForfeitLeadershipRequest,
    EtcdForfeitLeadershipResponse as ProtoEtcdForfeitLeadershipResponse,
    EtcdLeaveCluster as ProtoEtcdLeaveCluster,
    EtcdLeaveClusterRequest as ProtoEtcdLeaveClusterRequest,
    EtcdLeaveClusterResponse as ProtoEtcdLeaveClusterResponse,
    EtcdMember as ProtoEtcdMember, EtcdMemberAlarm as ProtoEtcdMemberAlarm,
    EtcdMemberListRequest as ProtoEtcdMemberListRequest,
    EtcdMemberListResponse as ProtoEtcdMemberListResponse,
    EtcdMemberStatus as ProtoEtcdMemberStatus, EtcdMembers as ProtoEtcdMembers,
    EtcdRemoveMemberByIdRequest as ProtoEtcdRemoveMemberByIdRequest,
    EtcdRemoveMemberByIdResponse as ProtoEtcdRemoveMemberByIdResponse,
    EtcdStatus as ProtoEtcdStatus, EtcdStatusResponse as ProtoEtcdStatusResponse,
};

// =============================================================================
// EtcdMemberList
// =============================================================================

/// Request to list etcd cluster members.
#[derive(Debug, Clone, Default)]
pub struct EtcdMemberListRequest {
    /// If true, query only the local node's view of the cluster.
    pub query_local: bool,
}

impl EtcdMemberListRequest {
    /// Create a new request to list etcd members.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Query only the local node's view.
    #[must_use]
    pub fn local() -> Self {
        Self { query_local: true }
    }
}

impl From<EtcdMemberListRequest> for ProtoEtcdMemberListRequest {
    fn from(req: EtcdMemberListRequest) -> Self {
        Self {
            query_local: req.query_local,
        }
    }
}

/// An etcd cluster member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EtcdMember {
    /// Member ID (unique identifier).
    pub id: u64,
    /// Human-readable hostname.
    pub hostname: String,
    /// URLs the member exposes to peers.
    pub peer_urls: Vec<String>,
    /// URLs the member exposes to clients.
    pub client_urls: Vec<String>,
    /// Whether this member is a learner.
    pub is_learner: bool,
}

impl From<ProtoEtcdMember> for EtcdMember {
    fn from(proto: ProtoEtcdMember) -> Self {
        Self {
            id: proto.id,
            hostname: proto.hostname,
            peer_urls: proto.peer_urls,
            client_urls: proto.client_urls,
            is_learner: proto.is_learner,
        }
    }
}

/// Result from a single node for member list request.
#[derive(Debug, Clone)]
pub struct EtcdMembersResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// List of etcd members.
    pub members: Vec<EtcdMember>,
}

impl From<ProtoEtcdMembers> for EtcdMembersResult {
    fn from(proto: ProtoEtcdMembers) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            members: proto.members.into_iter().map(EtcdMember::from).collect(),
        }
    }
}

/// Response from etcd member list request.
#[derive(Debug, Clone)]
pub struct EtcdMemberListResponse {
    /// Results from each node.
    pub results: Vec<EtcdMembersResult>,
}

impl From<ProtoEtcdMemberListResponse> for EtcdMemberListResponse {
    fn from(proto: ProtoEtcdMemberListResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(EtcdMembersResult::from)
                .collect(),
        }
    }
}

impl EtcdMemberListResponse {
    /// Get all unique members across all nodes.
    #[must_use]
    pub fn all_members(&self) -> Vec<&EtcdMember> {
        let mut seen_ids = std::collections::HashSet::new();
        let mut members = Vec::new();

        for result in &self.results {
            for member in &result.members {
                if seen_ids.insert(member.id) {
                    members.push(member);
                }
            }
        }

        members
    }

    /// Find a member by hostname.
    #[must_use]
    pub fn find_by_hostname(&self, hostname: &str) -> Option<&EtcdMember> {
        self.all_members()
            .into_iter()
            .find(|m| m.hostname == hostname)
    }

    /// Find a member by ID.
    #[must_use]
    pub fn find_by_id(&self, id: u64) -> Option<&EtcdMember> {
        self.all_members().into_iter().find(|m| m.id == id)
    }
}

// =============================================================================
// EtcdRemoveMemberByID
// =============================================================================

/// Request to remove an etcd member by ID.
///
/// Use this to remove members that no longer have an associated Talos node.
/// For nodes that are still running, use [`EtcdLeaveClusterRequest`] instead.
#[derive(Debug, Clone)]
pub struct EtcdRemoveMemberByIdRequest {
    /// The member ID to remove.
    pub member_id: u64,
}

impl EtcdRemoveMemberByIdRequest {
    /// Create a new request to remove a member by ID.
    #[must_use]
    pub fn new(member_id: u64) -> Self {
        Self { member_id }
    }
}

impl From<EtcdRemoveMemberByIdRequest> for ProtoEtcdRemoveMemberByIdRequest {
    fn from(req: EtcdRemoveMemberByIdRequest) -> Self {
        Self {
            member_id: req.member_id,
        }
    }
}

/// Result from removing a member by ID.
#[derive(Debug, Clone)]
pub struct EtcdRemoveMemberByIdResult {
    /// Node that processed this request.
    pub node: Option<String>,
}

/// Response from removing a member by ID.
#[derive(Debug, Clone)]
pub struct EtcdRemoveMemberByIdResponse {
    /// Results from each node.
    pub results: Vec<EtcdRemoveMemberByIdResult>,
}

impl From<ProtoEtcdRemoveMemberByIdResponse> for EtcdRemoveMemberByIdResponse {
    fn from(proto: ProtoEtcdRemoveMemberByIdResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(|m| EtcdRemoveMemberByIdResult {
                    node: m.metadata.map(|meta| meta.hostname),
                })
                .collect(),
        }
    }
}

impl EtcdRemoveMemberByIdResponse {
    /// Check if the operation was successful (at least one node responded).
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.results.is_empty()
    }
}

// =============================================================================
// EtcdLeaveCluster
// =============================================================================

/// Request for a node to leave the etcd cluster gracefully.
///
/// This should be called on the node that is being removed.
#[derive(Debug, Clone, Default)]
pub struct EtcdLeaveClusterRequest;

impl EtcdLeaveClusterRequest {
    /// Create a new request.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl From<EtcdLeaveClusterRequest> for ProtoEtcdLeaveClusterRequest {
    fn from(_req: EtcdLeaveClusterRequest) -> Self {
        Self {}
    }
}

/// Result from leaving the cluster.
#[derive(Debug, Clone)]
pub struct EtcdLeaveClusterResult {
    /// Node that left the cluster.
    pub node: Option<String>,
}

impl From<ProtoEtcdLeaveCluster> for EtcdLeaveClusterResult {
    fn from(proto: ProtoEtcdLeaveCluster) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
        }
    }
}

/// Response from leaving the cluster.
#[derive(Debug, Clone)]
pub struct EtcdLeaveClusterResponse {
    /// Results from each node.
    pub results: Vec<EtcdLeaveClusterResult>,
}

impl From<ProtoEtcdLeaveClusterResponse> for EtcdLeaveClusterResponse {
    fn from(proto: ProtoEtcdLeaveClusterResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(EtcdLeaveClusterResult::from)
                .collect(),
        }
    }
}

impl EtcdLeaveClusterResponse {
    /// Check if the operation was successful.
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.results.is_empty()
    }
}

// =============================================================================
// EtcdForfeitLeadership
// =============================================================================

/// Request to forfeit etcd leadership.
///
/// Causes the current leader to step down and trigger a new election.
#[derive(Debug, Clone, Default)]
pub struct EtcdForfeitLeadershipRequest;

impl EtcdForfeitLeadershipRequest {
    /// Create a new request.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl From<EtcdForfeitLeadershipRequest> for ProtoEtcdForfeitLeadershipRequest {
    fn from(_req: EtcdForfeitLeadershipRequest) -> Self {
        Self {}
    }
}

/// Result from forfeiting leadership.
#[derive(Debug, Clone)]
pub struct EtcdForfeitLeadershipResult {
    /// Node that processed this request.
    pub node: Option<String>,
    /// The member that forfeited leadership.
    pub member: String,
}

impl From<ProtoEtcdForfeitLeadership> for EtcdForfeitLeadershipResult {
    fn from(proto: ProtoEtcdForfeitLeadership) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            member: proto.member,
        }
    }
}

/// Response from forfeiting leadership.
#[derive(Debug, Clone)]
pub struct EtcdForfeitLeadershipResponse {
    /// Results from each node.
    pub results: Vec<EtcdForfeitLeadershipResult>,
}

impl From<ProtoEtcdForfeitLeadershipResponse> for EtcdForfeitLeadershipResponse {
    fn from(proto: ProtoEtcdForfeitLeadershipResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(EtcdForfeitLeadershipResult::from)
                .collect(),
        }
    }
}

// =============================================================================
// EtcdStatus
// =============================================================================

/// Status of an etcd member.
#[derive(Debug, Clone)]
pub struct EtcdMemberStatus {
    /// Member ID.
    pub member_id: u64,
    /// etcd protocol version.
    pub protocol_version: String,
    /// etcd storage version.
    pub storage_version: String,
    /// Database size in bytes.
    pub db_size: i64,
    /// Database size in use.
    pub db_size_in_use: i64,
    /// Current leader ID.
    pub leader: u64,
    /// Raft index.
    pub raft_index: u64,
    /// Raft term.
    pub raft_term: u64,
    /// Raft applied index.
    pub raft_applied_index: u64,
    /// Any errors reported.
    pub errors: Vec<String>,
    /// Whether this member is a learner.
    pub is_learner: bool,
}

impl From<ProtoEtcdMemberStatus> for EtcdMemberStatus {
    fn from(proto: ProtoEtcdMemberStatus) -> Self {
        Self {
            member_id: proto.member_id,
            protocol_version: proto.protocol_version,
            storage_version: proto.storage_version,
            db_size: proto.db_size,
            db_size_in_use: proto.db_size_in_use,
            leader: proto.leader,
            raft_index: proto.raft_index,
            raft_term: proto.raft_term,
            raft_applied_index: proto.raft_applied_index,
            errors: proto.errors,
            is_learner: proto.is_learner,
        }
    }
}

impl EtcdMemberStatus {
    /// Check if this member is the leader.
    #[must_use]
    pub fn is_leader(&self) -> bool {
        self.member_id == self.leader
    }

    /// Check if this member has any errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the database size in human-readable format.
    #[must_use]
    pub fn db_size_human(&self) -> String {
        humanize_bytes(self.db_size as u64)
    }
}

/// Result from status request.
#[derive(Debug, Clone)]
pub struct EtcdStatusResult {
    /// Node that returned this status.
    pub node: Option<String>,
    /// Member status.
    pub member_status: Option<EtcdMemberStatus>,
}

impl From<ProtoEtcdStatus> for EtcdStatusResult {
    fn from(proto: ProtoEtcdStatus) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            member_status: proto.member_status.map(EtcdMemberStatus::from),
        }
    }
}

/// Response from status request.
#[derive(Debug, Clone)]
pub struct EtcdStatusResponse {
    /// Results from each node.
    pub results: Vec<EtcdStatusResult>,
}

impl From<ProtoEtcdStatusResponse> for EtcdStatusResponse {
    fn from(proto: ProtoEtcdStatusResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(EtcdStatusResult::from)
                .collect(),
        }
    }
}

impl EtcdStatusResponse {
    /// Get the first member status.
    #[must_use]
    pub fn first(&self) -> Option<&EtcdMemberStatus> {
        self.results.first().and_then(|r| r.member_status.as_ref())
    }
}

// =============================================================================
// EtcdAlarm
// =============================================================================

/// Types of etcd alarms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtcdAlarmType {
    /// No alarm.
    None,
    /// No space alarm (database is full).
    NoSpace,
    /// Corruption detected.
    Corrupt,
}

impl From<i32> for EtcdAlarmType {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::NoSpace,
            2 => Self::Corrupt,
            _ => Self::None,
        }
    }
}

impl std::fmt::Display for EtcdAlarmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EtcdAlarmType::None => write!(f, "none"),
            EtcdAlarmType::NoSpace => write!(f, "NOSPACE"),
            EtcdAlarmType::Corrupt => write!(f, "CORRUPT"),
        }
    }
}

/// Alarm for an etcd member.
#[derive(Debug, Clone)]
pub struct EtcdMemberAlarm {
    /// Member ID with the alarm.
    pub member_id: u64,
    /// Type of alarm.
    pub alarm: EtcdAlarmType,
}

impl From<ProtoEtcdMemberAlarm> for EtcdMemberAlarm {
    fn from(proto: ProtoEtcdMemberAlarm) -> Self {
        Self {
            member_id: proto.member_id,
            alarm: EtcdAlarmType::from(proto.alarm),
        }
    }
}

/// Result from alarm list request.
#[derive(Debug, Clone)]
pub struct EtcdAlarmResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// Alarms for each member.
    pub member_alarms: Vec<EtcdMemberAlarm>,
}

impl From<ProtoEtcdAlarm> for EtcdAlarmResult {
    fn from(proto: ProtoEtcdAlarm) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            member_alarms: proto
                .member_alarms
                .into_iter()
                .map(EtcdMemberAlarm::from)
                .collect(),
        }
    }
}

/// Response from alarm list request.
#[derive(Debug, Clone)]
pub struct EtcdAlarmListResponse {
    /// Results from each node.
    pub results: Vec<EtcdAlarmResult>,
}

impl From<ProtoEtcdAlarmListResponse> for EtcdAlarmListResponse {
    fn from(proto: ProtoEtcdAlarmListResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(EtcdAlarmResult::from)
                .collect(),
        }
    }
}

impl EtcdAlarmListResponse {
    /// Check if there are any active alarms.
    #[must_use]
    pub fn has_alarms(&self) -> bool {
        self.results
            .iter()
            .any(|r| r.member_alarms.iter().any(|a| a.alarm != EtcdAlarmType::None))
    }

    /// Get all active alarms.
    #[must_use]
    pub fn active_alarms(&self) -> Vec<&EtcdMemberAlarm> {
        self.results
            .iter()
            .flat_map(|r| r.member_alarms.iter())
            .filter(|a| a.alarm != EtcdAlarmType::None)
            .collect()
    }
}

// =============================================================================
// EtcdAlarmDisarm
// =============================================================================

/// Result from disarming alarms.
#[derive(Debug, Clone)]
pub struct EtcdAlarmDisarmResult {
    /// Node that processed this request.
    pub node: Option<String>,
    /// Alarms that were disarmed.
    pub member_alarms: Vec<EtcdMemberAlarm>,
}

impl From<ProtoEtcdAlarmDisarm> for EtcdAlarmDisarmResult {
    fn from(proto: ProtoEtcdAlarmDisarm) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            member_alarms: proto
                .member_alarms
                .into_iter()
                .map(EtcdMemberAlarm::from)
                .collect(),
        }
    }
}

/// Response from disarming alarms.
#[derive(Debug, Clone)]
pub struct EtcdAlarmDisarmResponse {
    /// Results from each node.
    pub results: Vec<EtcdAlarmDisarmResult>,
}

impl From<ProtoEtcdAlarmDisarmResponse> for EtcdAlarmDisarmResponse {
    fn from(proto: ProtoEtcdAlarmDisarmResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(EtcdAlarmDisarmResult::from)
                .collect(),
        }
    }
}

// =============================================================================
// EtcdDefragment
// =============================================================================

/// Result from defragmentation.
#[derive(Debug, Clone)]
pub struct EtcdDefragmentResult {
    /// Node that was defragmented.
    pub node: Option<String>,
}

impl From<ProtoEtcdDefragment> for EtcdDefragmentResult {
    fn from(proto: ProtoEtcdDefragment) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
        }
    }
}

/// Response from defragmentation.
#[derive(Debug, Clone)]
pub struct EtcdDefragmentResponse {
    /// Results from each node.
    pub results: Vec<EtcdDefragmentResult>,
}

impl From<ProtoEtcdDefragmentResponse> for EtcdDefragmentResponse {
    fn from(proto: ProtoEtcdDefragmentResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(EtcdDefragmentResult::from)
                .collect(),
        }
    }
}

impl EtcdDefragmentResponse {
    /// Check if defragmentation was successful.
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.results.is_empty()
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn humanize_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etcd_member_list_request() {
        let req = EtcdMemberListRequest::new();
        assert!(!req.query_local);

        let req = EtcdMemberListRequest::local();
        assert!(req.query_local);
    }

    #[test]
    fn test_etcd_remove_member_by_id_request() {
        let req = EtcdRemoveMemberByIdRequest::new(12345);
        assert_eq!(req.member_id, 12345);

        let proto: ProtoEtcdRemoveMemberByIdRequest = req.into();
        assert_eq!(proto.member_id, 12345);
    }

    #[test]
    fn test_etcd_alarm_type() {
        assert_eq!(EtcdAlarmType::from(0), EtcdAlarmType::None);
        assert_eq!(EtcdAlarmType::from(1), EtcdAlarmType::NoSpace);
        assert_eq!(EtcdAlarmType::from(2), EtcdAlarmType::Corrupt);
        assert_eq!(EtcdAlarmType::from(99), EtcdAlarmType::None);

        assert_eq!(EtcdAlarmType::NoSpace.to_string(), "NOSPACE");
        assert_eq!(EtcdAlarmType::Corrupt.to_string(), "CORRUPT");
    }

    #[test]
    fn test_etcd_member_status_is_leader() {
        let status = EtcdMemberStatus {
            member_id: 100,
            protocol_version: "3.5.0".to_string(),
            storage_version: "3.5".to_string(),
            db_size: 10 * 1024 * 1024,
            db_size_in_use: 5 * 1024 * 1024,
            leader: 100,
            raft_index: 1000,
            raft_term: 5,
            raft_applied_index: 999,
            errors: vec![],
            is_learner: false,
        };

        assert!(status.is_leader());
        assert!(!status.has_errors());
        assert_eq!(status.db_size_human(), "10.00 MB");
    }

    #[test]
    fn test_humanize_bytes() {
        assert_eq!(humanize_bytes(500), "500 B");
        assert_eq!(humanize_bytes(1024), "1.00 KB");
        assert_eq!(humanize_bytes(1536), "1.50 KB");
        assert_eq!(humanize_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(humanize_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_etcd_leave_cluster_request() {
        let req = EtcdLeaveClusterRequest::new();
        let _proto: ProtoEtcdLeaveClusterRequest = req.into();
    }

    #[test]
    fn test_etcd_forfeit_leadership_request() {
        let req = EtcdForfeitLeadershipRequest::new();
        let _proto: ProtoEtcdForfeitLeadershipRequest = req.into();
    }
}
