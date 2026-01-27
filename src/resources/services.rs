// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for Service management APIs.
//!
//! Provides functionality to start, stop, restart, and monitor Talos services.

use crate::api::generated::machine::{
    ServiceRestart as ProtoServiceRestart, ServiceRestartRequest as ProtoServiceRestartRequest,
    ServiceRestartResponse as ProtoServiceRestartResponse, ServiceStart as ProtoServiceStart,
    ServiceStartRequest as ProtoServiceStartRequest,
    ServiceStartResponse as ProtoServiceStartResponse, ServiceStop as ProtoServiceStop,
    ServiceStopRequest as ProtoServiceStopRequest, ServiceStopResponse as ProtoServiceStopResponse,
};

// =============================================================================
// ServiceStart
// =============================================================================

/// Request to start a service.
#[derive(Debug, Clone)]
pub struct ServiceStartRequest {
    /// Service ID to start.
    pub id: String,
}

impl ServiceStartRequest {
    /// Create a new request to start a service.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl From<ServiceStartRequest> for ProtoServiceStartRequest {
    fn from(req: ServiceStartRequest) -> Self {
        Self { id: req.id }
    }
}

/// Result from starting a service.
#[derive(Debug, Clone)]
pub struct ServiceStartResult {
    /// Node that processed the request.
    pub node: Option<String>,
    /// Response message.
    pub response: String,
}

impl From<ProtoServiceStart> for ServiceStartResult {
    fn from(proto: ProtoServiceStart) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            response: proto.resp,
        }
    }
}

/// Response from starting a service.
#[derive(Debug, Clone)]
pub struct ServiceStartResponse {
    /// Results from each node.
    pub results: Vec<ServiceStartResult>,
}

impl From<ProtoServiceStartResponse> for ServiceStartResponse {
    fn from(proto: ProtoServiceStartResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(ServiceStartResult::from)
                .collect(),
        }
    }
}

impl ServiceStartResponse {
    /// Check if the operation was successful.
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.results.is_empty()
    }
}

// =============================================================================
// ServiceStop
// =============================================================================

/// Request to stop a service.
#[derive(Debug, Clone)]
pub struct ServiceStopRequest {
    /// Service ID to stop.
    pub id: String,
}

impl ServiceStopRequest {
    /// Create a new request to stop a service.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl From<ServiceStopRequest> for ProtoServiceStopRequest {
    fn from(req: ServiceStopRequest) -> Self {
        Self { id: req.id }
    }
}

/// Result from stopping a service.
#[derive(Debug, Clone)]
pub struct ServiceStopResult {
    /// Node that processed the request.
    pub node: Option<String>,
    /// Response message.
    pub response: String,
}

impl From<ProtoServiceStop> for ServiceStopResult {
    fn from(proto: ProtoServiceStop) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            response: proto.resp,
        }
    }
}

/// Response from stopping a service.
#[derive(Debug, Clone)]
pub struct ServiceStopResponse {
    /// Results from each node.
    pub results: Vec<ServiceStopResult>,
}

impl From<ProtoServiceStopResponse> for ServiceStopResponse {
    fn from(proto: ProtoServiceStopResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(ServiceStopResult::from)
                .collect(),
        }
    }
}

impl ServiceStopResponse {
    /// Check if the operation was successful.
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.results.is_empty()
    }
}

// =============================================================================
// ServiceRestart
// =============================================================================

/// Request to restart a service.
#[derive(Debug, Clone)]
pub struct ServiceRestartRequest {
    /// Service ID to restart.
    pub id: String,
}

impl ServiceRestartRequest {
    /// Create a new request to restart a service.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl From<ServiceRestartRequest> for ProtoServiceRestartRequest {
    fn from(req: ServiceRestartRequest) -> Self {
        Self { id: req.id }
    }
}

/// Result from restarting a service.
#[derive(Debug, Clone)]
pub struct ServiceRestartResult {
    /// Node that processed the request.
    pub node: Option<String>,
    /// Response message.
    pub response: String,
}

impl From<ProtoServiceRestart> for ServiceRestartResult {
    fn from(proto: ProtoServiceRestart) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            response: proto.resp,
        }
    }
}

/// Response from restarting a service.
#[derive(Debug, Clone)]
pub struct ServiceRestartResponse {
    /// Results from each node.
    pub results: Vec<ServiceRestartResult>,
}

impl From<ProtoServiceRestartResponse> for ServiceRestartResponse {
    fn from(proto: ProtoServiceRestartResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(ServiceRestartResult::from)
                .collect(),
        }
    }
}

impl ServiceRestartResponse {
    /// Check if the operation was successful.
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.results.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_start_request() {
        let req = ServiceStartRequest::new("kubelet");
        assert_eq!(req.id, "kubelet");

        let proto: ProtoServiceStartRequest = req.into();
        assert_eq!(proto.id, "kubelet");
    }

    #[test]
    fn test_service_stop_request() {
        let req = ServiceStopRequest::new("containerd");
        assert_eq!(req.id, "containerd");

        let proto: ProtoServiceStopRequest = req.into();
        assert_eq!(proto.id, "containerd");
    }

    #[test]
    fn test_service_restart_request() {
        let req = ServiceRestartRequest::new("etcd");
        assert_eq!(req.id, "etcd");

        let proto: ProtoServiceRestartRequest = req.into();
        assert_eq!(proto.id, "etcd");
    }
}
