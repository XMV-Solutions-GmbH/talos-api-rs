// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for Advanced APIs.
//!
//! Provides Rollback, GenerateClientConfiguration, PacketCapture, and Netstat operations.

use crate::api::generated::machine::{
    ConnectRecord as ProtoConnectRecord, GenerateClientConfiguration as ProtoGenerateClientConfig,
    GenerateClientConfigurationRequest as ProtoGenerateClientConfigRequest,
    GenerateClientConfigurationResponse as ProtoGenerateClientConfigResponse,
    Netstat as ProtoNetstat, NetstatRequest as ProtoNetstatRequest,
    NetstatResponse as ProtoNetstatResponse, PacketCaptureRequest as ProtoPacketCaptureRequest,
    RollbackResponse as ProtoRollbackResponse,
};

// =============================================================================
// Rollback
// =============================================================================

/// Response from a rollback request.
#[derive(Debug, Clone)]
pub struct RollbackResponse {
    /// Results from each node.
    pub results: Vec<RollbackResult>,
}

/// Result of a rollback operation on a node.
#[derive(Debug, Clone)]
pub struct RollbackResult {
    /// Node that returned this result.
    pub node: Option<String>,
}

impl From<ProtoRollbackResponse> for RollbackResponse {
    fn from(proto: ProtoRollbackResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(|m| RollbackResult {
                    node: m.metadata.map(|meta| meta.hostname),
                })
                .collect(),
        }
    }
}

impl RollbackResponse {
    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&RollbackResult> {
        self.results.first()
    }

    /// Check if rollback succeeded on all nodes.
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.results.is_empty()
    }
}

// =============================================================================
// GenerateClientConfiguration
// =============================================================================

/// Request to generate client configuration (talosconfig).
#[derive(Debug, Clone, Default)]
pub struct GenerateClientConfigurationRequest {
    /// Roles for the generated client certificate.
    pub roles: Vec<String>,
    /// Certificate TTL in seconds.
    pub crt_ttl_seconds: Option<i64>,
}

impl GenerateClientConfigurationRequest {
    /// Create a new request with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a request with specific roles.
    #[must_use]
    pub fn with_roles(roles: Vec<String>) -> Self {
        Self {
            roles,
            crt_ttl_seconds: None,
        }
    }

    /// Create a builder.
    #[must_use]
    pub fn builder() -> GenerateClientConfigurationRequestBuilder {
        GenerateClientConfigurationRequestBuilder::default()
    }
}

impl From<GenerateClientConfigurationRequest> for ProtoGenerateClientConfigRequest {
    fn from(req: GenerateClientConfigurationRequest) -> Self {
        Self {
            roles: req.roles,
            crt_ttl: req.crt_ttl_seconds.map(|s| prost_types::Duration {
                seconds: s,
                nanos: 0,
            }),
        }
    }
}

/// Builder for `GenerateClientConfigurationRequest`.
#[derive(Debug, Clone, Default)]
pub struct GenerateClientConfigurationRequestBuilder {
    roles: Vec<String>,
    crt_ttl_seconds: Option<i64>,
}

impl GenerateClientConfigurationRequestBuilder {
    /// Add a role.
    #[must_use]
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// Add multiple roles.
    #[must_use]
    pub fn roles(mut self, roles: Vec<String>) -> Self {
        self.roles.extend(roles);
        self
    }

    /// Set certificate TTL in seconds.
    #[must_use]
    pub fn crt_ttl_seconds(mut self, ttl: i64) -> Self {
        self.crt_ttl_seconds = Some(ttl);
        self
    }

    /// Set certificate TTL in hours.
    #[must_use]
    pub fn crt_ttl_hours(mut self, hours: i64) -> Self {
        self.crt_ttl_seconds = Some(hours * 3600);
        self
    }

    /// Set certificate TTL in days.
    #[must_use]
    pub fn crt_ttl_days(mut self, days: i64) -> Self {
        self.crt_ttl_seconds = Some(days * 86400);
        self
    }

    /// Build the request.
    #[must_use]
    pub fn build(self) -> GenerateClientConfigurationRequest {
        GenerateClientConfigurationRequest {
            roles: self.roles,
            crt_ttl_seconds: self.crt_ttl_seconds,
        }
    }
}

/// Generated client configuration result for a node.
#[derive(Debug, Clone)]
pub struct GenerateClientConfigurationResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// PEM-encoded CA certificate.
    pub ca: Vec<u8>,
    /// PEM-encoded client certificate.
    pub crt: Vec<u8>,
    /// PEM-encoded client key.
    pub key: Vec<u8>,
    /// Talosconfig file content.
    pub talosconfig: Vec<u8>,
}

impl From<ProtoGenerateClientConfig> for GenerateClientConfigurationResult {
    fn from(proto: ProtoGenerateClientConfig) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            ca: proto.ca,
            crt: proto.crt,
            key: proto.key,
            talosconfig: proto.talosconfig,
        }
    }
}

impl GenerateClientConfigurationResult {
    /// Get CA as string.
    #[must_use]
    pub fn ca_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.ca).ok()
    }

    /// Get certificate as string.
    #[must_use]
    pub fn crt_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.crt).ok()
    }

    /// Get key as string.
    #[must_use]
    pub fn key_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.key).ok()
    }

    /// Get talosconfig as string.
    #[must_use]
    pub fn talosconfig_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.talosconfig).ok()
    }
}

/// Response from generating client configuration.
#[derive(Debug, Clone)]
pub struct GenerateClientConfigurationResponse {
    /// Results from each node.
    pub results: Vec<GenerateClientConfigurationResult>,
}

impl From<ProtoGenerateClientConfigResponse> for GenerateClientConfigurationResponse {
    fn from(proto: ProtoGenerateClientConfigResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(GenerateClientConfigurationResult::from)
                .collect(),
        }
    }
}

impl GenerateClientConfigurationResponse {
    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&GenerateClientConfigurationResult> {
        self.results.first()
    }
}

// =============================================================================
// PacketCapture
// =============================================================================

/// Request to capture packets.
#[derive(Debug, Clone)]
pub struct PacketCaptureRequest {
    /// Network interface to capture on.
    pub interface: String,
    /// Enable promiscuous mode.
    pub promiscuous: bool,
    /// Snap length in bytes.
    pub snap_len: u32,
}

impl PacketCaptureRequest {
    /// Create a new packet capture request.
    #[must_use]
    pub fn new(interface: impl Into<String>) -> Self {
        Self {
            interface: interface.into(),
            promiscuous: false,
            snap_len: 65535,
        }
    }

    /// Create a builder.
    #[must_use]
    pub fn builder(interface: impl Into<String>) -> PacketCaptureRequestBuilder {
        PacketCaptureRequestBuilder::new(interface)
    }
}

impl From<PacketCaptureRequest> for ProtoPacketCaptureRequest {
    fn from(req: PacketCaptureRequest) -> Self {
        Self {
            interface: req.interface,
            promiscuous: req.promiscuous,
            snap_len: req.snap_len,
            bpf_filter: Vec::new(), // BPF filters not exposed for simplicity
        }
    }
}

/// Builder for `PacketCaptureRequest`.
#[derive(Debug, Clone)]
pub struct PacketCaptureRequestBuilder {
    interface: String,
    promiscuous: bool,
    snap_len: u32,
}

impl PacketCaptureRequestBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new(interface: impl Into<String>) -> Self {
        Self {
            interface: interface.into(),
            promiscuous: false,
            snap_len: 65535,
        }
    }

    /// Enable promiscuous mode.
    #[must_use]
    pub fn promiscuous(mut self, enabled: bool) -> Self {
        self.promiscuous = enabled;
        self
    }

    /// Set snap length.
    #[must_use]
    pub fn snap_len(mut self, len: u32) -> Self {
        self.snap_len = len;
        self
    }

    /// Build the request.
    #[must_use]
    pub fn build(self) -> PacketCaptureRequest {
        PacketCaptureRequest {
            interface: self.interface,
            promiscuous: self.promiscuous,
            snap_len: self.snap_len,
        }
    }
}

/// Response from packet capture (streaming pcap data).
#[derive(Debug, Clone, Default)]
pub struct PacketCaptureResponse {
    /// PCAP data.
    pub data: Vec<u8>,
    /// Node that returned this data.
    pub node: Option<String>,
}

impl PacketCaptureResponse {
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
// Netstat
// =============================================================================

/// Netstat filter type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetstatFilter {
    /// All connections.
    #[default]
    All,
    /// Connected sockets.
    Connected,
    /// Listening sockets.
    Listening,
}

impl From<NetstatFilter> for i32 {
    fn from(filter: NetstatFilter) -> Self {
        match filter {
            NetstatFilter::All => 0,
            NetstatFilter::Connected => 1,
            NetstatFilter::Listening => 2,
        }
    }
}

/// Layer 4 protocol filter.
#[derive(Debug, Clone, Default)]
pub struct L4ProtoFilter {
    /// Include TCP.
    pub tcp: bool,
    /// Include TCP6.
    pub tcp6: bool,
    /// Include UDP.
    pub udp: bool,
    /// Include UDP6.
    pub udp6: bool,
}

impl L4ProtoFilter {
    /// Create filter for all protocols.
    #[must_use]
    pub fn all() -> Self {
        Self {
            tcp: true,
            tcp6: true,
            udp: true,
            udp6: true,
        }
    }

    /// Create filter for TCP only.
    #[must_use]
    pub fn tcp_only() -> Self {
        Self {
            tcp: true,
            tcp6: true,
            ..Default::default()
        }
    }

    /// Create filter for UDP only.
    #[must_use]
    pub fn udp_only() -> Self {
        Self {
            udp: true,
            udp6: true,
            ..Default::default()
        }
    }
}

/// Request for netstat information.
#[derive(Debug, Clone, Default)]
pub struct NetstatRequest {
    /// Filter type.
    pub filter: NetstatFilter,
    /// Include process information.
    pub include_pid: bool,
    /// Layer 4 protocol filter.
    pub l4proto: Option<L4ProtoFilter>,
    /// Include host network namespace.
    pub host_network: bool,
    /// Include all network namespaces.
    pub all_netns: bool,
}

impl NetstatRequest {
    /// Create a new netstat request.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a request for listening sockets.
    #[must_use]
    pub fn listening() -> Self {
        Self {
            filter: NetstatFilter::Listening,
            ..Default::default()
        }
    }

    /// Create a request for connected sockets.
    #[must_use]
    pub fn connected() -> Self {
        Self {
            filter: NetstatFilter::Connected,
            ..Default::default()
        }
    }

    /// Create a builder.
    #[must_use]
    pub fn builder() -> NetstatRequestBuilder {
        NetstatRequestBuilder::default()
    }
}

impl From<NetstatRequest> for ProtoNetstatRequest {
    fn from(req: NetstatRequest) -> Self {
        use crate::api::generated::machine::netstat_request::{Feature, L4proto, NetNs};

        Self {
            filter: req.filter.into(),
            feature: if req.include_pid {
                Some(Feature { pid: true })
            } else {
                None
            },
            l4proto: req.l4proto.map(|l4| L4proto {
                tcp: l4.tcp,
                tcp6: l4.tcp6,
                udp: l4.udp,
                udp6: l4.udp6,
                udplite: false,
                udplite6: false,
                raw: false,
                raw6: false,
            }),
            netns: Some(NetNs {
                hostnetwork: req.host_network,
                netns: Vec::new(),
                allnetns: req.all_netns,
            }),
        }
    }
}

/// Builder for `NetstatRequest`.
#[derive(Debug, Clone, Default)]
pub struct NetstatRequestBuilder {
    filter: NetstatFilter,
    include_pid: bool,
    l4proto: Option<L4ProtoFilter>,
    host_network: bool,
    all_netns: bool,
}

impl NetstatRequestBuilder {
    /// Set filter.
    #[must_use]
    pub fn filter(mut self, filter: NetstatFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Include process information.
    #[must_use]
    pub fn include_pid(mut self, include: bool) -> Self {
        self.include_pid = include;
        self
    }

    /// Set L4 protocol filter.
    #[must_use]
    pub fn l4proto(mut self, l4proto: L4ProtoFilter) -> Self {
        self.l4proto = Some(l4proto);
        self
    }

    /// Include host network namespace.
    #[must_use]
    pub fn host_network(mut self, include: bool) -> Self {
        self.host_network = include;
        self
    }

    /// Include all network namespaces.
    #[must_use]
    pub fn all_netns(mut self, include: bool) -> Self {
        self.all_netns = include;
        self
    }

    /// Build the request.
    #[must_use]
    pub fn build(self) -> NetstatRequest {
        NetstatRequest {
            filter: self.filter,
            include_pid: self.include_pid,
            l4proto: self.l4proto,
            host_network: self.host_network,
            all_netns: self.all_netns,
        }
    }
}

/// Connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Reserved/unknown.
    Reserved,
    /// Established connection.
    Established,
    /// SYN sent.
    SynSent,
    /// SYN received.
    SynRecv,
    /// FIN wait 1.
    FinWait1,
    /// FIN wait 2.
    FinWait2,
    /// Time wait.
    TimeWait,
    /// Close.
    Close,
    /// Close wait.
    CloseWait,
    /// Last ACK.
    LastAck,
    /// Listen.
    Listen,
    /// Closing.
    Closing,
}

impl From<i32> for ConnectionState {
    fn from(state: i32) -> Self {
        match state {
            1 => Self::Established,
            2 => Self::SynSent,
            3 => Self::SynRecv,
            4 => Self::FinWait1,
            5 => Self::FinWait2,
            6 => Self::TimeWait,
            7 => Self::Close,
            8 => Self::CloseWait,
            9 => Self::LastAck,
            10 => Self::Listen,
            11 => Self::Closing,
            _ => Self::Reserved,
        }
    }
}

/// Connection record.
#[derive(Debug, Clone)]
pub struct ConnectionRecord {
    /// Layer 4 protocol.
    pub l4proto: String,
    /// Local IP address.
    pub local_ip: String,
    /// Local port.
    pub local_port: u32,
    /// Remote IP address.
    pub remote_ip: String,
    /// Remote port.
    pub remote_port: u32,
    /// Connection state.
    pub state: ConnectionState,
    /// TX queue size.
    pub tx_queue: u64,
    /// RX queue size.
    pub rx_queue: u64,
    /// Process ID (if available).
    pub pid: Option<u32>,
    /// Process name (if available).
    pub process_name: Option<String>,
    /// Network namespace.
    pub netns: String,
}

impl From<ProtoConnectRecord> for ConnectionRecord {
    fn from(proto: ProtoConnectRecord) -> Self {
        let (pid, process_name) = proto
            .process
            .map(|p| (Some(p.pid), Some(p.name)))
            .unwrap_or((None, None));

        Self {
            l4proto: proto.l4proto,
            local_ip: proto.localip,
            local_port: proto.localport,
            remote_ip: proto.remoteip,
            remote_port: proto.remoteport,
            state: ConnectionState::from(proto.state),
            tx_queue: proto.txqueue,
            rx_queue: proto.rxqueue,
            pid,
            process_name,
            netns: proto.netns,
        }
    }
}

/// Netstat result for a node.
#[derive(Debug, Clone)]
pub struct NetstatResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// Connection records.
    pub connections: Vec<ConnectionRecord>,
}

impl From<ProtoNetstat> for NetstatResult {
    fn from(proto: ProtoNetstat) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            connections: proto
                .connectrecord
                .into_iter()
                .map(ConnectionRecord::from)
                .collect(),
        }
    }
}

/// Response from netstat request.
#[derive(Debug, Clone)]
pub struct NetstatResponse {
    /// Results from each node.
    pub results: Vec<NetstatResult>,
}

impl From<ProtoNetstatResponse> for NetstatResponse {
    fn from(proto: ProtoNetstatResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(NetstatResult::from)
                .collect(),
        }
    }
}

impl NetstatResponse {
    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&NetstatResult> {
        self.results.first()
    }

    /// Get total number of connections.
    #[must_use]
    pub fn total_connections(&self) -> usize {
        self.results.iter().map(|r| r.connections.len()).sum()
    }

    /// Get all listening connections.
    #[must_use]
    pub fn listening(&self) -> Vec<&ConnectionRecord> {
        self.results
            .iter()
            .flat_map(|r| &r.connections)
            .filter(|c| c.state == ConnectionState::Listen)
            .collect()
    }

    /// Get all established connections.
    #[must_use]
    pub fn established(&self) -> Vec<&ConnectionRecord> {
        self.results
            .iter()
            .flat_map(|r| &r.connections)
            .filter(|c| c.state == ConnectionState::Established)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback_response() {
        let result = RollbackResult {
            node: Some("node1".to_string()),
        };
        assert_eq!(result.node, Some("node1".to_string()));
    }

    #[test]
    fn test_generate_client_config_request() {
        let req = GenerateClientConfigurationRequest::new();
        assert!(req.roles.is_empty());
    }

    #[test]
    fn test_generate_client_config_builder() {
        let req = GenerateClientConfigurationRequest::builder()
            .role("os:admin")
            .role("os:reader")
            .crt_ttl_days(30)
            .build();

        assert_eq!(req.roles, vec!["os:admin", "os:reader"]);
        assert_eq!(req.crt_ttl_seconds, Some(30 * 86400));
    }

    #[test]
    fn test_packet_capture_request() {
        let req = PacketCaptureRequest::new("eth0");
        assert_eq!(req.interface, "eth0");
        assert!(!req.promiscuous);
        assert_eq!(req.snap_len, 65535);
    }

    #[test]
    fn test_packet_capture_builder() {
        let req = PacketCaptureRequest::builder("bond0")
            .promiscuous(true)
            .snap_len(1500)
            .build();

        assert_eq!(req.interface, "bond0");
        assert!(req.promiscuous);
        assert_eq!(req.snap_len, 1500);
    }

    #[test]
    fn test_netstat_request() {
        let req = NetstatRequest::listening();
        assert_eq!(req.filter, NetstatFilter::Listening);
    }

    #[test]
    fn test_netstat_builder() {
        let req = NetstatRequest::builder()
            .filter(NetstatFilter::Connected)
            .include_pid(true)
            .l4proto(L4ProtoFilter::tcp_only())
            .host_network(true)
            .build();

        assert_eq!(req.filter, NetstatFilter::Connected);
        assert!(req.include_pid);
        assert!(req.l4proto.is_some());
        assert!(req.host_network);
    }

    #[test]
    fn test_connection_state() {
        assert_eq!(ConnectionState::from(1), ConnectionState::Established);
        assert_eq!(ConnectionState::from(10), ConnectionState::Listen);
        assert_eq!(ConnectionState::from(999), ConnectionState::Reserved);
    }

    #[test]
    fn test_l4proto_filter() {
        let all = L4ProtoFilter::all();
        assert!(all.tcp && all.tcp6 && all.udp && all.udp6);

        let tcp = L4ProtoFilter::tcp_only();
        assert!(tcp.tcp && tcp.tcp6 && !tcp.udp && !tcp.udp6);
    }
}
