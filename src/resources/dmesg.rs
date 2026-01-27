// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for the Dmesg API.
//!
//! Provides access to the kernel message buffer (dmesg) for diagnostics.

use crate::api::generated::machine::DmesgRequest as ProtoDmesgRequest;

/// Request for kernel message buffer (dmesg).
#[derive(Debug, Clone, Default)]
pub struct DmesgRequest {
    /// If true, follow the log output (like `dmesg -w`).
    pub follow: bool,
    /// If true, only return the last messages.
    pub tail: bool,
}

impl DmesgRequest {
    /// Create a new dmesg request.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a request to follow dmesg output.
    #[must_use]
    pub fn follow() -> Self {
        Self {
            follow: true,
            tail: false,
        }
    }

    /// Create a request for tail output only.
    #[must_use]
    pub fn tail() -> Self {
        Self {
            follow: false,
            tail: true,
        }
    }

    /// Create a builder.
    #[must_use]
    pub fn builder() -> DmesgRequestBuilder {
        DmesgRequestBuilder::default()
    }
}

impl From<DmesgRequest> for ProtoDmesgRequest {
    fn from(req: DmesgRequest) -> Self {
        Self {
            follow: req.follow,
            tail: req.tail,
        }
    }
}

/// Builder for DmesgRequest.
#[derive(Debug, Clone, Default)]
pub struct DmesgRequestBuilder {
    follow: bool,
    tail: bool,
}

impl DmesgRequestBuilder {
    /// Set whether to follow output.
    #[must_use]
    pub fn follow(mut self, follow: bool) -> Self {
        self.follow = follow;
        self
    }

    /// Set whether to tail output.
    #[must_use]
    pub fn tail(mut self, tail: bool) -> Self {
        self.tail = tail;
        self
    }

    /// Build the request.
    #[must_use]
    pub fn build(self) -> DmesgRequest {
        DmesgRequest {
            follow: self.follow,
            tail: self.tail,
        }
    }
}

/// Response containing kernel message buffer content.
#[derive(Debug, Clone)]
pub struct DmesgResponse {
    /// Raw dmesg data.
    data: Vec<u8>,
    /// Node that returned this dmesg.
    pub node: Option<String>,
}

impl DmesgResponse {
    /// Create a new response from raw data.
    #[must_use]
    pub fn new(data: Vec<u8>, node: Option<String>) -> Self {
        Self { data, node }
    }

    /// Get the raw bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Try to convert to a UTF-8 string.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    /// Convert to string, replacing invalid UTF-8 with replacement character.
    #[must_use]
    pub fn as_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.data)
    }

    /// Get the length in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the response is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get individual lines from dmesg output.
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
    fn test_dmesg_request_new() {
        let req = DmesgRequest::new();
        assert!(!req.follow);
        assert!(!req.tail);
    }

    #[test]
    fn test_dmesg_request_follow() {
        let req = DmesgRequest::follow();
        assert!(req.follow);
        assert!(!req.tail);
    }

    #[test]
    fn test_dmesg_request_tail() {
        let req = DmesgRequest::tail();
        assert!(!req.follow);
        assert!(req.tail);
    }

    #[test]
    fn test_dmesg_request_builder() {
        let req = DmesgRequest::builder()
            .follow(true)
            .tail(true)
            .build();
        assert!(req.follow);
        assert!(req.tail);
    }

    #[test]
    fn test_proto_conversion() {
        let req = DmesgRequest::builder()
            .follow(true)
            .tail(false)
            .build();
        let proto: ProtoDmesgRequest = req.into();
        assert!(proto.follow);
        assert!(!proto.tail);
    }

    #[test]
    fn test_dmesg_response() {
        let data = b"[    0.000000] Linux version 5.15.0\n[    0.000001] Command line: talos.platform=metal".to_vec();
        let response = DmesgResponse::new(data.clone(), Some("node1".to_string()));

        assert_eq!(response.len(), data.len());
        assert!(!response.is_empty());
        assert!(response.as_str().is_ok());
        assert_eq!(response.lines().len(), 2);
        assert!(response.lines()[0].contains("Linux version"));
    }
}
