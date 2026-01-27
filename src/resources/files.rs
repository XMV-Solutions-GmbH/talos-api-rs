// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for File Operation APIs.
//!
//! Provides access to file listing, reading, copying, and disk usage.

use crate::api::generated::machine::{
    CopyRequest as ProtoCopyRequest, DiskUsageInfo as ProtoDiskUsageInfo,
    DiskUsageRequest as ProtoDiskUsageRequest, FileInfo as ProtoFileInfo,
    ListRequest as ProtoListRequest, ReadRequest as ProtoReadRequest,
};

// =============================================================================
// List (Directory Listing)
// =============================================================================

/// Type of file to filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileType {
    /// Regular file.
    #[default]
    Regular,
    /// Directory.
    Directory,
    /// Symbolic link.
    Symlink,
}

impl From<FileType> for i32 {
    fn from(ft: FileType) -> Self {
        match ft {
            FileType::Regular => 0,
            FileType::Directory => 1,
            FileType::Symlink => 2,
        }
    }
}

/// Request to list directory contents.
#[derive(Debug, Clone, Default)]
pub struct ListRequest {
    /// Root directory to list.
    pub root: String,
    /// Whether to recurse into subdirectories.
    pub recurse: bool,
    /// Maximum recursion depth (0 = unlimited).
    pub recursion_depth: i32,
    /// File types to include.
    pub types: Vec<FileType>,
    /// Whether to report extended attributes.
    pub report_xattrs: bool,
}

impl ListRequest {
    /// Create a new list request for a path.
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            ..Default::default()
        }
    }

    /// Create a builder for more complex requests.
    #[must_use]
    pub fn builder(root: impl Into<String>) -> ListRequestBuilder {
        ListRequestBuilder::new(root)
    }
}

impl From<ListRequest> for ProtoListRequest {
    fn from(req: ListRequest) -> Self {
        Self {
            root: req.root,
            recurse: req.recurse,
            recursion_depth: req.recursion_depth,
            types: req.types.into_iter().map(i32::from).collect(),
            report_xattrs: req.report_xattrs,
        }
    }
}

/// Builder for `ListRequest`.
#[derive(Debug, Clone)]
pub struct ListRequestBuilder {
    root: String,
    recurse: bool,
    recursion_depth: i32,
    types: Vec<FileType>,
    report_xattrs: bool,
}

impl ListRequestBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            recurse: false,
            recursion_depth: 0,
            types: Vec::new(),
            report_xattrs: false,
        }
    }

    /// Enable recursive listing.
    #[must_use]
    pub fn recurse(mut self, recurse: bool) -> Self {
        self.recurse = recurse;
        self
    }

    /// Set maximum recursion depth.
    #[must_use]
    pub fn recursion_depth(mut self, depth: i32) -> Self {
        self.recursion_depth = depth;
        self
    }

    /// Filter by file types.
    #[must_use]
    pub fn types(mut self, types: Vec<FileType>) -> Self {
        self.types = types;
        self
    }

    /// Report extended attributes.
    #[must_use]
    pub fn report_xattrs(mut self, report: bool) -> Self {
        self.report_xattrs = report;
        self
    }

    /// Build the request.
    #[must_use]
    pub fn build(self) -> ListRequest {
        ListRequest {
            root: self.root,
            recurse: self.recurse,
            recursion_depth: self.recursion_depth,
            types: self.types,
            report_xattrs: self.report_xattrs,
        }
    }
}

/// Information about a file or directory.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Node that returned this info.
    pub node: Option<String>,
    /// File name (including path).
    pub name: String,
    /// File size in bytes.
    pub size: i64,
    /// UNIX mode/permission flags.
    pub mode: u32,
    /// UNIX timestamp of last modification.
    pub modified: i64,
    /// Whether this is a directory.
    pub is_dir: bool,
    /// Error message if any.
    pub error: Option<String>,
    /// Symlink target if this is a symlink.
    pub link: Option<String>,
    /// Relative name from root path.
    pub relative_name: String,
    /// Owner UID.
    pub uid: u32,
    /// Owner GID.
    pub gid: u32,
}

impl From<ProtoFileInfo> for FileInfo {
    fn from(proto: ProtoFileInfo) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            name: proto.name,
            size: proto.size,
            mode: proto.mode,
            modified: proto.modified,
            is_dir: proto.is_dir,
            error: if proto.error.is_empty() {
                None
            } else {
                Some(proto.error)
            },
            link: if proto.link.is_empty() {
                None
            } else {
                Some(proto.link)
            },
            relative_name: proto.relative_name,
            uid: proto.uid,
            gid: proto.gid,
        }
    }
}

impl FileInfo {
    /// Check if this entry has an error.
    #[must_use]
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Check if this is a regular file.
    #[must_use]
    pub fn is_file(&self) -> bool {
        !self.is_dir && self.link.is_none()
    }

    /// Check if this is a symlink.
    #[must_use]
    pub fn is_symlink(&self) -> bool {
        self.link.is_some()
    }
}

/// Response from a list request (streaming).
#[derive(Debug, Clone, Default)]
pub struct ListResponse {
    /// File entries.
    pub entries: Vec<FileInfo>,
}

impl ListResponse {
    /// Create a new response.
    #[must_use]
    pub fn new(entries: Vec<FileInfo>) -> Self {
        Self { entries }
    }

    /// Get the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get only directories.
    #[must_use]
    pub fn directories(&self) -> Vec<&FileInfo> {
        self.entries.iter().filter(|e| e.is_dir).collect()
    }

    /// Get only files.
    #[must_use]
    pub fn files(&self) -> Vec<&FileInfo> {
        self.entries.iter().filter(|e| e.is_file()).collect()
    }
}

// =============================================================================
// Read (File Reading)
// =============================================================================

/// Request to read a file.
#[derive(Debug, Clone)]
pub struct ReadRequest {
    /// Path to the file to read.
    pub path: String,
}

impl ReadRequest {
    /// Create a new read request.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl From<ReadRequest> for ProtoReadRequest {
    fn from(req: ReadRequest) -> Self {
        Self { path: req.path }
    }
}

/// Response from a read request (streaming).
#[derive(Debug, Clone, Default)]
pub struct ReadResponse {
    /// File content.
    pub data: Vec<u8>,
    /// Node that returned this data.
    pub node: Option<String>,
}

impl ReadResponse {
    /// Create a new response.
    #[must_use]
    pub fn new(data: Vec<u8>, node: Option<String>) -> Self {
        Self { data, node }
    }

    /// Get data as UTF-8 string.
    ///
    /// Returns `None` if the data is not valid UTF-8.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.data).ok()
    }

    /// Get data as lossy UTF-8 string.
    #[must_use]
    pub fn as_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }

    /// Get data length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// =============================================================================
// Copy (File/Directory Copy)
// =============================================================================

/// Request to copy a file or directory.
#[derive(Debug, Clone)]
pub struct CopyRequest {
    /// Root path to copy from.
    pub root_path: String,
}

impl CopyRequest {
    /// Create a new copy request.
    #[must_use]
    pub fn new(root_path: impl Into<String>) -> Self {
        Self {
            root_path: root_path.into(),
        }
    }
}

impl From<CopyRequest> for ProtoCopyRequest {
    fn from(req: CopyRequest) -> Self {
        Self {
            root_path: req.root_path,
        }
    }
}

/// Response from a copy request (streaming tar data).
#[derive(Debug, Clone, Default)]
pub struct CopyResponse {
    /// Tar archive data.
    pub data: Vec<u8>,
    /// Node that returned this data.
    pub node: Option<String>,
}

impl CopyResponse {
    /// Create a new response.
    #[must_use]
    pub fn new(data: Vec<u8>, node: Option<String>) -> Self {
        Self { data, node }
    }

    /// Get data length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// =============================================================================
// DiskUsage
// =============================================================================

/// Request to get disk usage.
#[derive(Debug, Clone, Default)]
pub struct DiskUsageRequest {
    /// Paths to calculate disk usage for.
    pub paths: Vec<String>,
    /// Maximum recursion depth (0 = unlimited).
    pub recursion_depth: i32,
    /// Include all files, not just directories.
    pub all: bool,
    /// Size threshold (positive = exclude smaller, negative = exclude larger).
    pub threshold: i64,
}

impl DiskUsageRequest {
    /// Create a new disk usage request for a path.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            paths: vec![path.into()],
            ..Default::default()
        }
    }

    /// Create a request for multiple paths.
    #[must_use]
    pub fn for_paths(paths: Vec<String>) -> Self {
        Self {
            paths,
            ..Default::default()
        }
    }

    /// Create a builder for more complex requests.
    #[must_use]
    pub fn builder() -> DiskUsageRequestBuilder {
        DiskUsageRequestBuilder::default()
    }
}

impl From<DiskUsageRequest> for ProtoDiskUsageRequest {
    fn from(req: DiskUsageRequest) -> Self {
        Self {
            paths: req.paths,
            recursion_depth: req.recursion_depth,
            all: req.all,
            threshold: req.threshold,
        }
    }
}

/// Builder for `DiskUsageRequest`.
#[derive(Debug, Clone, Default)]
pub struct DiskUsageRequestBuilder {
    paths: Vec<String>,
    recursion_depth: i32,
    all: bool,
    threshold: i64,
}

impl DiskUsageRequestBuilder {
    /// Add a path.
    #[must_use]
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.paths.push(path.into());
        self
    }

    /// Add multiple paths.
    #[must_use]
    pub fn paths(mut self, paths: Vec<String>) -> Self {
        self.paths.extend(paths);
        self
    }

    /// Set recursion depth.
    #[must_use]
    pub fn recursion_depth(mut self, depth: i32) -> Self {
        self.recursion_depth = depth;
        self
    }

    /// Include all files.
    #[must_use]
    pub fn all(mut self, all: bool) -> Self {
        self.all = all;
        self
    }

    /// Set size threshold.
    #[must_use]
    pub fn threshold(mut self, threshold: i64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Build the request.
    #[must_use]
    pub fn build(self) -> DiskUsageRequest {
        DiskUsageRequest {
            paths: self.paths,
            recursion_depth: self.recursion_depth,
            all: self.all,
            threshold: self.threshold,
        }
    }
}

/// Disk usage information for a file or directory.
#[derive(Debug, Clone)]
pub struct DiskUsageInfo {
    /// Node that returned this info.
    pub node: Option<String>,
    /// File/directory name.
    pub name: String,
    /// Size in bytes.
    pub size: i64,
    /// Error message if any.
    pub error: Option<String>,
    /// Relative name from root.
    pub relative_name: String,
}

impl From<ProtoDiskUsageInfo> for DiskUsageInfo {
    fn from(proto: ProtoDiskUsageInfo) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            name: proto.name,
            size: proto.size,
            error: if proto.error.is_empty() {
                None
            } else {
                Some(proto.error)
            },
            relative_name: proto.relative_name,
        }
    }
}

impl DiskUsageInfo {
    /// Check if this entry has an error.
    #[must_use]
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Get size in human-readable format.
    #[must_use]
    pub fn size_human(&self) -> String {
        humanize_bytes(self.size as u64)
    }
}

/// Response from a disk usage request (streaming).
#[derive(Debug, Clone, Default)]
pub struct DiskUsageResponse {
    /// Disk usage entries.
    pub entries: Vec<DiskUsageInfo>,
}

impl DiskUsageResponse {
    /// Create a new response.
    #[must_use]
    pub fn new(entries: Vec<DiskUsageInfo>) -> Self {
        Self { entries }
    }

    /// Get total size across all entries.
    #[must_use]
    pub fn total_size(&self) -> i64 {
        self.entries.iter().map(|e| e.size).sum()
    }

    /// Get the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Convert bytes to human-readable format.
fn humanize_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_request_new() {
        let req = ListRequest::new("/var/log");
        assert_eq!(req.root, "/var/log");
        assert!(!req.recurse);
    }

    #[test]
    fn test_list_request_builder() {
        let req = ListRequest::builder("/etc")
            .recurse(true)
            .recursion_depth(3)
            .types(vec![FileType::Regular, FileType::Directory])
            .report_xattrs(true)
            .build();

        assert_eq!(req.root, "/etc");
        assert!(req.recurse);
        assert_eq!(req.recursion_depth, 3);
        assert_eq!(req.types.len(), 2);
        assert!(req.report_xattrs);
    }

    #[test]
    fn test_file_info() {
        let info = FileInfo {
            node: Some("node1".to_string()),
            name: "/var/log/syslog".to_string(),
            size: 1024,
            mode: 0o644,
            modified: 1234567890,
            is_dir: false,
            error: None,
            link: None,
            relative_name: "syslog".to_string(),
            uid: 0,
            gid: 0,
        };

        assert!(info.is_file());
        assert!(!info.is_dir);
        assert!(!info.is_symlink());
        assert!(!info.has_error());
    }

    #[test]
    fn test_read_request() {
        let req = ReadRequest::new("/etc/hosts");
        assert_eq!(req.path, "/etc/hosts");
    }

    #[test]
    fn test_read_response() {
        let resp = ReadResponse::new(b"hello world".to_vec(), Some("node1".to_string()));
        assert_eq!(resp.as_str(), Some("hello world"));
        assert_eq!(resp.len(), 11);
    }

    #[test]
    fn test_copy_request() {
        let req = CopyRequest::new("/var/log");
        assert_eq!(req.root_path, "/var/log");
    }

    #[test]
    fn test_disk_usage_request() {
        let req = DiskUsageRequest::new("/var");
        assert_eq!(req.paths, vec!["/var"]);
    }

    #[test]
    fn test_disk_usage_request_builder() {
        let req = DiskUsageRequest::builder()
            .path("/var")
            .path("/tmp")
            .recursion_depth(2)
            .all(true)
            .threshold(1024)
            .build();

        assert_eq!(req.paths, vec!["/var", "/tmp"]);
        assert_eq!(req.recursion_depth, 2);
        assert!(req.all);
        assert_eq!(req.threshold, 1024);
    }

    #[test]
    fn test_humanize_bytes() {
        assert_eq!(humanize_bytes(512), "512 B");
        assert_eq!(humanize_bytes(1024), "1.00 KB");
        assert_eq!(humanize_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(humanize_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
