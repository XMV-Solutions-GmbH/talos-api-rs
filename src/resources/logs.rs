// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for the Logs API.
//!
//! Provides streaming access to service and container logs.

use crate::api::generated::machine::LogsRequest as ProtoLogsRequest;

/// Container driver type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContainerDriver {
    /// Default containerd driver.
    #[default]
    Containerd,
    /// CRI driver.
    Cri,
}

impl From<ContainerDriver> for i32 {
    fn from(driver: ContainerDriver) -> Self {
        match driver {
            ContainerDriver::Containerd => 0,
            ContainerDriver::Cri => 1,
        }
    }
}

impl std::fmt::Display for ContainerDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerDriver::Containerd => write!(f, "containerd"),
            ContainerDriver::Cri => write!(f, "cri"),
        }
    }
}

/// Request for service/container logs.
#[derive(Debug, Clone, Default)]
pub struct LogsRequest {
    /// Namespace (e.g., "system" for system services).
    pub namespace: String,
    /// Service or container ID.
    pub id: String,
    /// Container driver.
    pub driver: ContainerDriver,
    /// Follow the log output.
    pub follow: bool,
    /// Number of lines to tail (0 = all).
    pub tail_lines: i32,
}

impl LogsRequest {
    /// Create a new logs request for a service.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            namespace: String::new(),
            id: id.into(),
            driver: ContainerDriver::Containerd,
            follow: false,
            tail_lines: 0,
        }
    }

    /// Create a new logs request with namespace.
    #[must_use]
    pub fn with_namespace(namespace: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            id: id.into(),
            driver: ContainerDriver::Containerd,
            follow: false,
            tail_lines: 0,
        }
    }

    /// Create a builder.
    #[must_use]
    pub fn builder(id: impl Into<String>) -> LogsRequestBuilder {
        LogsRequestBuilder::new(id)
    }
}

impl From<LogsRequest> for ProtoLogsRequest {
    fn from(req: LogsRequest) -> Self {
        Self {
            namespace: req.namespace,
            id: req.id,
            driver: req.driver.into(),
            follow: req.follow,
            tail_lines: req.tail_lines,
        }
    }
}

/// Builder for LogsRequest.
#[derive(Debug, Clone)]
pub struct LogsRequestBuilder {
    namespace: String,
    id: String,
    driver: ContainerDriver,
    follow: bool,
    tail_lines: i32,
}

impl LogsRequestBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            namespace: String::new(),
            id: id.into(),
            driver: ContainerDriver::Containerd,
            follow: false,
            tail_lines: 0,
        }
    }

    /// Set the namespace.
    #[must_use]
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    /// Set the container driver.
    #[must_use]
    pub fn driver(mut self, driver: ContainerDriver) -> Self {
        self.driver = driver;
        self
    }

    /// Follow log output.
    #[must_use]
    pub fn follow(mut self, follow: bool) -> Self {
        self.follow = follow;
        self
    }

    /// Tail the last N lines.
    #[must_use]
    pub fn tail(mut self, lines: i32) -> Self {
        self.tail_lines = lines;
        self
    }

    /// Build the request.
    #[must_use]
    pub fn build(self) -> LogsRequest {
        LogsRequest {
            namespace: self.namespace,
            id: self.id,
            driver: self.driver,
            follow: self.follow,
            tail_lines: self.tail_lines,
        }
    }
}

/// Response containing log data.
#[derive(Debug, Clone)]
pub struct LogsResponse {
    /// Raw log data.
    data: Vec<u8>,
    /// Node that returned the logs.
    pub node: Option<String>,
}

impl LogsResponse {
    /// Create a new response.
    #[must_use]
    pub fn new(data: Vec<u8>, node: Option<String>) -> Self {
        Self { data, node }
    }

    /// Get raw bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Try to convert to UTF-8 string.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    /// Convert to string, replacing invalid UTF-8.
    #[must_use]
    pub fn as_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.data)
    }

    /// Get the length in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get individual lines.
    #[must_use]
    pub fn lines(&self) -> Vec<&str> {
        self.as_str()
            .map(|s| s.lines().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logs_request_new() {
        let req = LogsRequest::new("kubelet");
        assert_eq!(req.id, "kubelet");
        assert!(req.namespace.is_empty());
        assert!(!req.follow);
    }

    #[test]
    fn test_logs_request_with_namespace() {
        let req = LogsRequest::with_namespace("system", "kubelet");
        assert_eq!(req.namespace, "system");
        assert_eq!(req.id, "kubelet");
    }

    #[test]
    fn test_logs_request_builder() {
        let req = LogsRequest::builder("etcd")
            .namespace("system")
            .driver(ContainerDriver::Cri)
            .follow(true)
            .tail(100)
            .build();

        assert_eq!(req.id, "etcd");
        assert_eq!(req.namespace, "system");
        assert_eq!(req.driver, ContainerDriver::Cri);
        assert!(req.follow);
        assert_eq!(req.tail_lines, 100);
    }

    #[test]
    fn test_container_driver() {
        assert_eq!(i32::from(ContainerDriver::Containerd), 0);
        assert_eq!(i32::from(ContainerDriver::Cri), 1);
        assert_eq!(ContainerDriver::Cri.to_string(), "cri");
    }

    #[test]
    fn test_proto_conversion() {
        let req = LogsRequest::builder("kubelet")
            .follow(true)
            .tail(50)
            .build();

        let proto: ProtoLogsRequest = req.into();
        assert_eq!(proto.id, "kubelet");
        assert!(proto.follow);
        assert_eq!(proto.tail_lines, 50);
    }
}
