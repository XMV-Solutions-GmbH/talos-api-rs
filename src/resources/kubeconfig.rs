// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for kubeconfig operations.
//!
//! The Kubeconfig API retrieves the kubeconfig file from a Talos cluster.
//! This is a server-streaming RPC that returns the kubeconfig data in chunks.

/// Response containing the kubeconfig data.
///
/// The kubeconfig is retrieved via server-streaming RPC and assembled
/// from multiple data chunks.
#[derive(Debug, Clone)]
pub struct KubeconfigResponse {
    /// The complete kubeconfig YAML content.
    pub data: Vec<u8>,
    /// Node hostname (if available from metadata)
    pub node: Option<String>,
}

impl KubeconfigResponse {
    /// Create a new kubeconfig response from raw data.
    #[must_use]
    pub fn new(data: Vec<u8>, node: Option<String>) -> Self {
        Self { data, node }
    }

    /// Get the kubeconfig as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error if the kubeconfig is not valid UTF-8.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    /// Get the kubeconfig as a UTF-8 string, lossy conversion.
    ///
    /// Invalid UTF-8 sequences are replaced with the replacement character.
    #[must_use]
    pub fn as_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }

    /// Write the kubeconfig to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        std::fs::write(path, &self.data)
    }

    /// Check if the kubeconfig data is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the length of the kubeconfig data in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kubeconfig_response_new() {
        let data = b"apiVersion: v1\nkind: Config".to_vec();
        let response = KubeconfigResponse::new(data.clone(), Some("node1".to_string()));

        assert_eq!(response.data, data);
        assert_eq!(response.node, Some("node1".to_string()));
    }

    #[test]
    fn test_kubeconfig_response_as_str() {
        let data = b"apiVersion: v1\nkind: Config".to_vec();
        let response = KubeconfigResponse::new(data, None);

        assert_eq!(response.as_str().unwrap(), "apiVersion: v1\nkind: Config");
    }

    #[test]
    fn test_kubeconfig_response_as_string_lossy() {
        let data = b"apiVersion: v1\nkind: Config".to_vec();
        let response = KubeconfigResponse::new(data, None);

        assert_eq!(response.as_string_lossy(), "apiVersion: v1\nkind: Config");
    }

    #[test]
    fn test_kubeconfig_response_is_empty() {
        let empty = KubeconfigResponse::new(vec![], None);
        assert!(empty.is_empty());

        let non_empty = KubeconfigResponse::new(b"data".to_vec(), None);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_kubeconfig_response_len() {
        let response = KubeconfigResponse::new(b"12345".to_vec(), None);
        assert_eq!(response.len(), 5);
    }
}
