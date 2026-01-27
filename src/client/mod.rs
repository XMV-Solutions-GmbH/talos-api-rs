// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::api::machine::machine_service_client::MachineServiceClient;
use crate::api::machine::ApplyConfigurationRequest as ProtoApplyConfigRequest;
use crate::api::machine::BootstrapRequest as ProtoBootstrapRequest;
use crate::api::machine::CopyRequest as ProtoCopyRequest;
use crate::api::machine::DiskUsageRequest as ProtoDiskUsageRequest;
use crate::api::machine::DmesgRequest as ProtoDmesgRequest;
use crate::api::machine::EtcdForfeitLeadershipRequest as ProtoEtcdForfeitLeadershipRequest;
use crate::api::machine::EtcdLeaveClusterRequest as ProtoEtcdLeaveClusterRequest;
use crate::api::machine::EtcdMemberListRequest as ProtoEtcdMemberListRequest;
use crate::api::machine::EtcdRemoveMemberByIdRequest as ProtoEtcdRemoveMemberByIdRequest;
use crate::api::machine::GenerateClientConfigurationRequest as ProtoGenerateClientConfigRequest;
use crate::api::machine::ListRequest as ProtoListRequest;
use crate::api::machine::LogsRequest as ProtoLogsRequest;
use crate::api::machine::NetstatRequest as ProtoNetstatRequest;
use crate::api::machine::PacketCaptureRequest as ProtoPacketCaptureRequest;
use crate::api::machine::ReadRequest as ProtoReadRequest;
use crate::api::machine::ResetRequest as ProtoResetRequest;
use crate::api::machine::RollbackRequest as ProtoRollbackRequest;
use crate::api::machine::ServiceRestartRequest as ProtoServiceRestartRequest;
use crate::api::machine::ServiceStartRequest as ProtoServiceStartRequest;
use crate::api::machine::ServiceStopRequest as ProtoServiceStopRequest;
use crate::api::machine::UpgradeRequest as ProtoUpgradeRequest;
use crate::api::version::version_service_client::VersionServiceClient;
use crate::error::Result;
use crate::resources::{
    ApplyConfigurationRequest, ApplyConfigurationResponse, BootstrapRequest, BootstrapResponse,
    CopyRequest, CopyResponse, CpuInfoResponse, DiskStatsResponse, DiskUsageInfo, DiskUsageRequest,
    DiskUsageResponse, DmesgRequest, DmesgResponse, EtcdAlarmDisarmResponse, EtcdAlarmListResponse,
    EtcdDefragmentResponse, EtcdForfeitLeadershipRequest, EtcdForfeitLeadershipResponse,
    EtcdLeaveClusterRequest, EtcdLeaveClusterResponse, EtcdMemberListRequest,
    EtcdMemberListResponse, EtcdRemoveMemberByIdRequest, EtcdRemoveMemberByIdResponse,
    EtcdStatusResponse, FileInfo, GenerateClientConfigurationRequest,
    GenerateClientConfigurationResponse, KubeconfigResponse, ListRequest, ListResponse,
    LoadAvgResponse, LogsRequest, LogsResponse, MemoryResponse, MountsResponse, NetstatRequest,
    NetstatResponse, NetworkDeviceStatsResponse, PacketCaptureRequest, PacketCaptureResponse,
    ProcessesResponse, ReadRequest, ReadResponse, ResetRequest, ResetResponse, RollbackResponse,
    ServiceRestartRequest, ServiceRestartResponse, ServiceStartRequest, ServiceStartResponse,
    ServiceStopRequest, ServiceStopResponse, UpgradeRequest, UpgradeResponse,
};
use hyper_util::rt::TokioIo;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};

/// Configuration for the Talos API client.
#[derive(Clone, Debug)]
pub struct TalosClientConfig {
    /// The gRPC endpoint URL.
    pub endpoint: String,
    /// Path to client certificate.
    pub crt_path: Option<String>,
    /// Path to client private key.
    pub key_path: Option<String>,
    /// Path to CA certificate.
    pub ca_path: Option<String>,
    /// If true, skips TLS verification (insecure).
    pub insecure: bool,
    /// Connection timeout for establishing the gRPC channel.
    pub connect_timeout: Option<Duration>,
    /// Request timeout for individual RPC calls.
    pub request_timeout: Option<Duration>,
    /// Keepalive interval for long-running connections.
    pub keepalive_interval: Option<Duration>,
    /// Keepalive timeout.
    pub keepalive_timeout: Option<Duration>,
}

impl Default for TalosClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://127.0.0.1:50000".to_string(),
            crt_path: None,
            key_path: None,
            ca_path: None,
            insecure: false,
            connect_timeout: Some(Duration::from_secs(10)),
            request_timeout: Some(Duration::from_secs(30)),
            keepalive_interval: Some(Duration::from_secs(30)),
            keepalive_timeout: Some(Duration::from_secs(10)),
        }
    }
}

impl TalosClientConfig {
    /// Create a new configuration with an endpoint.
    #[must_use]
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            ..Default::default()
        }
    }

    /// Create a builder for more complex configuration.
    #[must_use]
    pub fn builder(endpoint: impl Into<String>) -> TalosClientConfigBuilder {
        TalosClientConfigBuilder::new(endpoint)
    }

    /// Set client certificate path.
    #[must_use]
    pub fn with_client_cert(mut self, crt_path: impl Into<String>) -> Self {
        self.crt_path = Some(crt_path.into());
        self
    }

    /// Set client key path.
    #[must_use]
    pub fn with_client_key(mut self, key_path: impl Into<String>) -> Self {
        self.key_path = Some(key_path.into());
        self
    }

    /// Set CA certificate path.
    #[must_use]
    pub fn with_ca(mut self, ca_path: impl Into<String>) -> Self {
        self.ca_path = Some(ca_path.into());
        self
    }

    /// Enable insecure mode (skip TLS verification).
    #[must_use]
    pub fn insecure(mut self) -> Self {
        self.insecure = true;
        self
    }

    /// Set connect timeout.
    #[must_use]
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Set request timeout.
    #[must_use]
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Disable all timeouts.
    #[must_use]
    pub fn no_timeout(mut self) -> Self {
        self.connect_timeout = None;
        self.request_timeout = None;
        self
    }
}

/// Builder for `TalosClientConfig`.
#[derive(Debug, Clone)]
pub struct TalosClientConfigBuilder {
    endpoint: String,
    crt_path: Option<String>,
    key_path: Option<String>,
    ca_path: Option<String>,
    insecure: bool,
    connect_timeout: Option<Duration>,
    request_timeout: Option<Duration>,
    keepalive_interval: Option<Duration>,
    keepalive_timeout: Option<Duration>,
}

impl TalosClientConfigBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            crt_path: None,
            key_path: None,
            ca_path: None,
            insecure: false,
            connect_timeout: Some(Duration::from_secs(10)),
            request_timeout: Some(Duration::from_secs(30)),
            keepalive_interval: Some(Duration::from_secs(30)),
            keepalive_timeout: Some(Duration::from_secs(10)),
        }
    }

    /// Set client certificate path.
    #[must_use]
    pub fn client_cert(mut self, path: impl Into<String>) -> Self {
        self.crt_path = Some(path.into());
        self
    }

    /// Set client key path.
    #[must_use]
    pub fn client_key(mut self, path: impl Into<String>) -> Self {
        self.key_path = Some(path.into());
        self
    }

    /// Set CA certificate path.
    #[must_use]
    pub fn ca_cert(mut self, path: impl Into<String>) -> Self {
        self.ca_path = Some(path.into());
        self
    }

    /// Enable insecure mode.
    #[must_use]
    pub fn insecure(mut self) -> Self {
        self.insecure = true;
        self
    }

    /// Set connect timeout.
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Set request timeout.
    #[must_use]
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Set keepalive settings.
    #[must_use]
    pub fn keepalive(mut self, interval: Duration, timeout: Duration) -> Self {
        self.keepalive_interval = Some(interval);
        self.keepalive_timeout = Some(timeout);
        self
    }

    /// Disable timeouts.
    #[must_use]
    pub fn no_timeout(mut self) -> Self {
        self.connect_timeout = None;
        self.request_timeout = None;
        self
    }

    /// Build the configuration.
    #[must_use]
    pub fn build(self) -> TalosClientConfig {
        TalosClientConfig {
            endpoint: self.endpoint,
            crt_path: self.crt_path,
            key_path: self.key_path,
            ca_path: self.ca_path,
            insecure: self.insecure,
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
            keepalive_interval: self.keepalive_interval,
            keepalive_timeout: self.keepalive_timeout,
        }
    }
}

#[derive(Clone)]
pub struct TalosClient {
    #[allow(dead_code)] // TODO: Remove when config is used
    config: TalosClientConfig,
    channel: Channel,
}

impl TalosClient {
    pub async fn new(config: TalosClientConfig) -> Result<Self> {
        // Install ring as default crypto provider (supports ED25519)
        let _ = rustls::crypto::ring::default_provider().install_default();

        // Check if using plain HTTP (no TLS)
        let is_http = config.endpoint.starts_with("http://");

        let channel = if is_http {
            // Plain HTTP - no TLS at all
            Self::create_http_channel(&config).await?
        } else if config.insecure {
            Self::create_insecure_channel(&config).await?
        } else {
            Self::create_mtls_channel(&config).await?
        };

        Ok(Self { config, channel })
    }

    /// Create a plain HTTP channel (no TLS)
    async fn create_http_channel(config: &TalosClientConfig) -> Result<Channel> {
        let mut endpoint = Channel::from_shared(config.endpoint.clone())
            .map_err(|e| crate::error::TalosError::Config(e.to_string()))?;

        // Apply timeout configuration
        if let Some(timeout) = config.connect_timeout {
            endpoint = endpoint.connect_timeout(timeout);
        }
        if let Some(timeout) = config.request_timeout {
            endpoint = endpoint.timeout(timeout);
        }
        if let Some(interval) = config.keepalive_interval {
            if let Some(ka_timeout) = config.keepalive_timeout {
                endpoint = endpoint
                    .http2_keep_alive_interval(interval)
                    .keep_alive_timeout(ka_timeout);
            }
        }

        let channel = endpoint.connect().await?;
        Ok(channel)
    }

    /// Create an insecure channel (TLS without certificate verification)
    async fn create_insecure_channel(config: &TalosClientConfig) -> Result<Channel> {
        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(rustls::RootCertStore::empty())
            .with_no_client_auth();

        Self::connect_with_custom_tls(config, tls_config, true).await
    }

    /// Create an mTLS channel with full certificate verification
    async fn create_mtls_channel(config: &TalosClientConfig) -> Result<Channel> {
        // Load CA certificate
        let root_store = if let Some(ca_path) = &config.ca_path {
            let ca_pem = std::fs::read(ca_path).map_err(|e| {
                crate::error::TalosError::Config(format!("Failed to read CA cert: {e}"))
            })?;
            let mut root_store = rustls::RootCertStore::empty();
            let certs = Self::load_pem_certs(&ca_pem)?;
            for cert in certs {
                root_store.add(cert).map_err(|e| {
                    crate::error::TalosError::Config(format!("Failed to add CA cert: {e}"))
                })?;
            }
            root_store
        } else {
            // Use system roots if no CA provided
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            root_store
        };

        // Build TLS config with or without client auth
        let tls_config =
            if let (Some(crt_path), Some(key_path)) = (&config.crt_path, &config.key_path) {
                // mTLS with client certificate
                let cert_pem = std::fs::read(crt_path).map_err(|e| {
                    crate::error::TalosError::Config(format!("Failed to read client cert: {e}"))
                })?;
                let key_pem = std::fs::read(key_path).map_err(|e| {
                    crate::error::TalosError::Config(format!("Failed to read client key: {e}"))
                })?;

                let client_certs = Self::load_pem_certs(&cert_pem)?;
                let client_key = Self::load_pem_key(&key_pem)?;

                rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_client_auth_cert(client_certs, client_key)
                    .map_err(|e| {
                        crate::error::TalosError::Config(format!(
                            "Failed to configure client auth: {e}"
                        ))
                    })?
            } else {
                // TLS without client auth
                rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth()
            };

        Self::connect_with_custom_tls(config, tls_config, false).await
    }

    /// Connect using a custom rustls TLS configuration
    async fn connect_with_custom_tls(
        config: &TalosClientConfig,
        mut tls_config: rustls::ClientConfig,
        skip_verification: bool,
    ) -> Result<Channel> {
        // Override verifier for insecure mode
        if skip_verification {
            tls_config
                .dangerous()
                .set_certificate_verifier(Arc::new(NoVerifier));
        }

        // gRPC requires ALPN h2
        tls_config.alpn_protocols = vec![b"h2".to_vec()];
        let tls_config = Arc::new(tls_config);
        let connector = tokio_rustls::TlsConnector::from(tls_config);

        // Extract host for SNI
        let endpoint_url = if config.endpoint.starts_with("http") {
            config.endpoint.clone()
        } else {
            format!("https://{}", config.endpoint)
        };
        let parsed_url = url::Url::parse(&endpoint_url)
            .map_err(|e| crate::error::TalosError::Config(format!("Invalid endpoint URL: {e}")))?;
        let host = parsed_url
            .host_str()
            .ok_or_else(|| crate::error::TalosError::Config("No host in endpoint".to_string()))?
            .to_string();
        let port = parsed_url.port().unwrap_or(50000);

        // For custom connector, use http:// scheme (we handle TLS ourselves)
        let endpoint_for_connector = format!("http://{}:{}", host, port);

        // Build endpoint with timeout configuration
        let mut endpoint = Endpoint::from_shared(endpoint_for_connector)
            .map_err(|e| crate::error::TalosError::Config(e.to_string()))?;

        // Apply timeout configuration
        if let Some(timeout) = config.connect_timeout {
            endpoint = endpoint.connect_timeout(timeout);
        }
        if let Some(timeout) = config.request_timeout {
            endpoint = endpoint.timeout(timeout);
        }
        if let Some(interval) = config.keepalive_interval {
            if let Some(ka_timeout) = config.keepalive_timeout {
                endpoint = endpoint
                    .http2_keep_alive_interval(interval)
                    .keep_alive_timeout(ka_timeout);
            }
        }

        let channel = endpoint
            .connect_with_connector(tower::service_fn(move |uri: tonic::transport::Uri| {
                let connector = connector.clone();
                let host = host.clone();
                async move {
                    let uri_host = uri.host().unwrap_or("127.0.0.1");
                    let uri_port = uri.port_u16().unwrap_or(50000);
                    let addr = format!("{}:{}", uri_host, uri_port);

                    let tcp = tokio::net::TcpStream::connect(addr).await?;

                    // Use actual hostname for SNI (important for cert verification)
                    let server_name = ServerName::try_from(host.clone())
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

                    let tls_stream = connector.connect(server_name, tcp).await?;
                    Ok::<_, std::io::Error>(TokioIo::new(tls_stream))
                }
            }))
            .await?;

        Ok(channel)
    }

    /// Load PEM-encoded certificates
    #[allow(clippy::result_large_err)]
    fn load_pem_certs(pem_data: &[u8]) -> Result<Vec<CertificateDer<'static>>> {
        let mut reader = std::io::BufReader::new(pem_data);
        let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                crate::error::TalosError::Config(format!("Failed to parse PEM certificates: {e}"))
            })?;
        if certs.is_empty() {
            return Err(crate::error::TalosError::Config(
                "No certificates found in PEM data".to_string(),
            ));
        }
        Ok(certs)
    }

    /// Load PEM-encoded private key (supports RSA, EC, PKCS8, and ED25519)
    #[allow(clippy::result_large_err)]
    fn load_pem_key(pem_data: &[u8]) -> Result<PrivateKeyDer<'static>> {
        // First, try standard PEM formats via rustls_pemfile
        let mut reader = std::io::BufReader::new(pem_data);

        loop {
            match rustls_pemfile::read_one(&mut reader) {
                Ok(Some(rustls_pemfile::Item::Pkcs1Key(key))) => {
                    return Ok(PrivateKeyDer::Pkcs1(key));
                }
                Ok(Some(rustls_pemfile::Item::Pkcs8Key(key))) => {
                    return Ok(PrivateKeyDer::Pkcs8(key));
                }
                Ok(Some(rustls_pemfile::Item::Sec1Key(key))) => {
                    return Ok(PrivateKeyDer::Sec1(key));
                }
                Ok(Some(_)) => {
                    // Skip other PEM items (certificates, etc.)
                    continue;
                }
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    return Err(crate::error::TalosError::Config(format!(
                        "Failed to parse PEM key: {e}"
                    )));
                }
            }
        }

        // Fallback: Handle non-standard "ED25519 PRIVATE KEY" PEM label
        // Talos uses this format, which is PKCS#8-encoded but with a custom label
        let pem_str = std::str::from_utf8(pem_data)
            .map_err(|e| crate::error::TalosError::Config(format!("Invalid UTF-8 in key: {e}")))?;

        if pem_str.contains("-----BEGIN ED25519 PRIVATE KEY-----") {
            // Extract the base64 content between the headers
            let start_marker = "-----BEGIN ED25519 PRIVATE KEY-----";
            let end_marker = "-----END ED25519 PRIVATE KEY-----";

            if let Some(start) = pem_str.find(start_marker) {
                if let Some(end) = pem_str.find(end_marker) {
                    let base64_content = &pem_str[start + start_marker.len()..end];
                    let base64_clean: String = base64_content
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect();

                    let der_bytes = base64::Engine::decode(
                        &base64::engine::general_purpose::STANDARD,
                        &base64_clean,
                    )
                    .map_err(|e| {
                        crate::error::TalosError::Config(format!(
                            "Failed to decode ED25519 key: {e}"
                        ))
                    })?;

                    // ED25519 PRIVATE KEY is actually PKCS#8 encoded
                    return Ok(PrivateKeyDer::Pkcs8(
                        rustls::pki_types::PrivatePkcs8KeyDer::from(der_bytes),
                    ));
                }
            }
        }

        Err(crate::error::TalosError::Config(
            "No private key found in PEM data".to_string(),
        ))
    }

    /// Access the Version API group
    pub fn version(&self) -> VersionServiceClient<Channel> {
        VersionServiceClient::new(self.channel.clone())
    }

    /// Access the Machine API group
    pub fn machine(&self) -> MachineServiceClient<Channel> {
        MachineServiceClient::new(self.channel.clone())
    }

    // ========================================================================
    // High-level convenience methods
    // ========================================================================

    /// Apply a configuration to the node.
    ///
    /// This is a high-level wrapper around the `MachineService::ApplyConfiguration` RPC.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use talos_api::{TalosClient, TalosClientConfig, ApplyConfigurationRequest, ApplyMode};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = TalosClient::new(TalosClientConfig {
    ///     endpoint: "https://192.168.1.100:50000".to_string(),
    ///     insecure: true,
    ///     ..Default::default()
    /// }).await?;
    ///
    /// // Apply configuration in dry-run mode
    /// let request = ApplyConfigurationRequest::builder()
    ///     .config_yaml("machine:\n  type: worker")
    ///     .mode(ApplyMode::NoReboot)
    ///     .dry_run(true)
    ///     .build();
    ///
    /// let response = client.apply_configuration(request).await?;
    /// println!("Warnings: {:?}", response.all_warnings());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the configuration is invalid.
    pub async fn apply_configuration(
        &self,
        request: ApplyConfigurationRequest,
    ) -> Result<ApplyConfigurationResponse> {
        let proto_request: ProtoApplyConfigRequest = request.into();
        let response = self
            .machine()
            .apply_configuration(proto_request)
            .await?
            .into_inner();
        Ok(response.into())
    }

    /// Apply a YAML configuration string to the node.
    ///
    /// Convenience method for simple configuration application.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use talos_api::{TalosClient, TalosClientConfig, ApplyMode};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = TalosClient::new(TalosClientConfig::default()).await?;
    /// let config_yaml = std::fs::read_to_string("machine.yaml")?;
    /// let response = client.apply_configuration_yaml(&config_yaml, ApplyMode::Auto, false).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn apply_configuration_yaml(
        &self,
        yaml: &str,
        mode: crate::ApplyMode,
        dry_run: bool,
    ) -> Result<ApplyConfigurationResponse> {
        let request = ApplyConfigurationRequest::builder()
            .config_yaml(yaml)
            .mode(mode)
            .dry_run(dry_run)
            .build();
        self.apply_configuration(request).await
    }

    /// Bootstrap the etcd cluster on this node.
    ///
    /// This initializes a new etcd cluster. **This should only be called ONCE**
    /// on the first control-plane node when creating a new Talos cluster.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use talos_api::{TalosClient, TalosClientConfig, BootstrapRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = TalosClient::new(TalosClientConfig::default()).await?;
    ///
    /// // Bootstrap a new cluster
    /// let response = client.bootstrap(BootstrapRequest::new()).await?;
    /// println!("Bootstrap complete: {:?}", response.first());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Recovery
    ///
    /// To recover from an etcd snapshot (uploaded via `EtcdRecover` RPC):
    ///
    /// ```no_run
    /// use talos_api::{TalosClient, TalosClientConfig, BootstrapRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = TalosClient::new(TalosClientConfig::default()).await?;
    /// let response = client.bootstrap(BootstrapRequest::recovery()).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The node is not a control-plane node
    /// - etcd is already bootstrapped
    /// - Network/connection issues
    pub async fn bootstrap(&self, request: BootstrapRequest) -> Result<BootstrapResponse> {
        let proto_request: ProtoBootstrapRequest = request.into();
        let response = self.machine().bootstrap(proto_request).await?.into_inner();
        Ok(response.into())
    }

    /// Bootstrap a new etcd cluster (convenience method).
    ///
    /// Equivalent to `bootstrap(BootstrapRequest::new())`.
    pub async fn bootstrap_cluster(&self) -> Result<BootstrapResponse> {
        self.bootstrap(BootstrapRequest::new()).await
    }

    /// Retrieve the kubeconfig from the cluster.
    ///
    /// This is a server-streaming RPC that retrieves the kubeconfig file
    /// from a control-plane node. The kubeconfig can be used to access
    /// the Kubernetes API of the cluster.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use talos_api::{TalosClient, TalosClientConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = TalosClient::new(TalosClientConfig::default()).await?;
    ///
    /// // Get kubeconfig
    /// let kubeconfig = client.kubeconfig().await?;
    /// println!("Kubeconfig from node: {:?}", kubeconfig.node);
    ///
    /// // Write to file
    /// kubeconfig.write_to_file("kubeconfig.yaml")?;
    ///
    /// // Or get as string
    /// let yaml = kubeconfig.as_str()?;
    /// println!("{}", yaml);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The node is not a control-plane node
    /// - The cluster is not yet bootstrapped
    /// - Network/connection issues
    pub async fn kubeconfig(&self) -> Result<KubeconfigResponse> {
        use tonic::codegen::tokio_stream::StreamExt;

        let mut stream = self.machine().kubeconfig(()).await?.into_inner();

        let mut data = Vec::new();
        let mut node = None;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            // Capture node from first chunk with metadata
            if node.is_none() {
                if let Some(metadata) = &chunk.metadata {
                    node = Some(metadata.hostname.clone());
                }
            }
            data.extend(chunk.bytes);
        }

        Ok(KubeconfigResponse::new(data, node))
    }

    /// Reset a Talos node, optionally wiping disks.
    ///
    /// # Warning
    ///
    /// This is a **destructive** operation. The node will be reset and may
    /// lose all data depending on the wipe mode configured.
    ///
    /// # Arguments
    ///
    /// * `request` - The reset request configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use talos_api::{TalosClient, TalosClientConfig, ResetRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TalosClientConfig::new("https://192.168.1.100:50000".parse()?);
    /// let client = TalosClient::new(config).await?;
    ///
    /// // Graceful reset (leaves etcd cluster first)
    /// let response = client.reset(ResetRequest::graceful()).await?;
    ///
    /// // Force reset with full disk wipe
    /// let response = client.reset(ResetRequest::force()).await?;
    ///
    /// // Custom reset
    /// let response = client.reset(
    ///     ResetRequest::builder()
    ///         .graceful(true)
    ///         .reboot(true)
    ///         .build()
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reset(&self, request: ResetRequest) -> Result<ResetResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoResetRequest = request.into();
        let response = client.reset(proto_request).await?;
        let inner = response.into_inner();

        Ok(ResetResponse::from(inner))
    }

    /// Gracefully reset a Talos node.
    ///
    /// This is a convenience method that performs a graceful reset, which:
    /// - Leaves the etcd cluster gracefully (for control plane nodes)
    /// - Reboots after reset
    /// - Does not wipe disks
    ///
    /// For more control, use [`reset`](Self::reset) with a custom [`ResetRequest`].
    pub async fn reset_graceful(&self) -> Result<ResetResponse> {
        self.reset(ResetRequest::graceful()).await
    }

    // =========================================================================
    // etcd Operations
    // =========================================================================

    /// List etcd cluster members.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use talos_api::{TalosClient, TalosClientConfig, EtcdMemberListRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TalosClientConfig::new("https://192.168.1.100:50000".parse()?);
    /// let client = TalosClient::new(config).await?;
    ///
    /// let response = client.etcd_member_list(EtcdMemberListRequest::new()).await?;
    /// for member in response.all_members() {
    ///     println!("{}: {}", member.id, member.hostname);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn etcd_member_list(
        &self,
        request: EtcdMemberListRequest,
    ) -> Result<EtcdMemberListResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoEtcdMemberListRequest = request.into();
        let response = client.etcd_member_list(proto_request).await?;
        let inner = response.into_inner();

        Ok(EtcdMemberListResponse::from(inner))
    }

    /// Remove an etcd member by ID.
    ///
    /// Use this to remove members that no longer have an associated Talos node.
    /// For nodes that are still running, use [`etcd_leave_cluster`](Self::etcd_leave_cluster).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use talos_api::{TalosClient, TalosClientConfig, EtcdRemoveMemberByIdRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TalosClientConfig::new("https://192.168.1.100:50000".parse()?);
    /// let client = TalosClient::new(config).await?;
    ///
    /// // First, find the member ID
    /// let members = client.etcd_member_list(Default::default()).await?;
    /// if let Some(member) = members.find_by_hostname("old-node") {
    ///     client.etcd_remove_member_by_id(
    ///         EtcdRemoveMemberByIdRequest::new(member.id)
    ///     ).await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn etcd_remove_member_by_id(
        &self,
        request: EtcdRemoveMemberByIdRequest,
    ) -> Result<EtcdRemoveMemberByIdResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoEtcdRemoveMemberByIdRequest = request.into();
        let response = client.etcd_remove_member_by_id(proto_request).await?;
        let inner = response.into_inner();

        Ok(EtcdRemoveMemberByIdResponse::from(inner))
    }

    /// Make a node leave the etcd cluster gracefully.
    ///
    /// This should be called on the node that is being removed.
    pub async fn etcd_leave_cluster(
        &self,
        request: EtcdLeaveClusterRequest,
    ) -> Result<EtcdLeaveClusterResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoEtcdLeaveClusterRequest = request.into();
        let response = client.etcd_leave_cluster(proto_request).await?;
        let inner = response.into_inner();

        Ok(EtcdLeaveClusterResponse::from(inner))
    }

    /// Forfeit etcd leadership.
    ///
    /// Causes the current leader to step down and trigger a new election.
    pub async fn etcd_forfeit_leadership(
        &self,
        request: EtcdForfeitLeadershipRequest,
    ) -> Result<EtcdForfeitLeadershipResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoEtcdForfeitLeadershipRequest = request.into();
        let response = client.etcd_forfeit_leadership(proto_request).await?;
        let inner = response.into_inner();

        Ok(EtcdForfeitLeadershipResponse::from(inner))
    }

    /// Get etcd status for the current member.
    pub async fn etcd_status(&self) -> Result<EtcdStatusResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.etcd_status(()).await?;
        let inner = response.into_inner();

        Ok(EtcdStatusResponse::from(inner))
    }

    /// List etcd alarms.
    pub async fn etcd_alarm_list(&self) -> Result<EtcdAlarmListResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.etcd_alarm_list(()).await?;
        let inner = response.into_inner();

        Ok(EtcdAlarmListResponse::from(inner))
    }

    /// Disarm etcd alarms.
    pub async fn etcd_alarm_disarm(&self) -> Result<EtcdAlarmDisarmResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.etcd_alarm_disarm(()).await?;
        let inner = response.into_inner();

        Ok(EtcdAlarmDisarmResponse::from(inner))
    }

    /// Defragment etcd storage.
    ///
    /// **Warning**: This is a resource-heavy operation.
    pub async fn etcd_defragment(&self) -> Result<EtcdDefragmentResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.etcd_defragment(()).await?;
        let inner = response.into_inner();

        Ok(EtcdDefragmentResponse::from(inner))
    }

    // =========================================================================
    // Diagnostics
    // =========================================================================

    /// Get kernel message buffer (dmesg).
    ///
    /// This is a server-streaming RPC that returns kernel messages.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use talos_api::{TalosClient, TalosClientConfig, DmesgRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TalosClientConfig::new("https://192.168.1.100:50000".parse()?);
    /// let client = TalosClient::new(config).await?;
    ///
    /// let dmesg = client.dmesg(DmesgRequest::new()).await?;
    /// println!("{}", dmesg.as_string_lossy());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dmesg(&self, request: DmesgRequest) -> Result<DmesgResponse> {
        use tonic::codegen::tokio_stream::StreamExt;

        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoDmesgRequest = request.into();
        let response = client.dmesg(proto_request).await?;
        let mut stream = response.into_inner();

        let mut data = Vec::new();
        let mut node = None;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if node.is_none() {
                if let Some(metadata) = &chunk.metadata {
                    node = Some(metadata.hostname.clone());
                }
            }
            data.extend(chunk.bytes);
        }

        Ok(DmesgResponse::new(data, node))
    }

    // =========================================================================
    // Upgrade
    // =========================================================================

    /// Upgrade a Talos node to a new version.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use talos_api::{TalosClient, TalosClientConfig, UpgradeRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TalosClientConfig::new("https://192.168.1.100:50000".parse()?);
    /// let client = TalosClient::new(config).await?;
    ///
    /// // Upgrade to a specific version
    /// let response = client.upgrade(
    ///     UpgradeRequest::new("ghcr.io/siderolabs/installer:v1.6.0")
    /// ).await?;
    ///
    /// // Staged upgrade (downloads but doesn't apply until reboot)
    /// let response = client.upgrade(
    ///     UpgradeRequest::builder("ghcr.io/siderolabs/installer:v1.6.0")
    ///         .stage(true)
    ///         .preserve(true)
    ///         .build()
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upgrade(&self, request: UpgradeRequest) -> Result<UpgradeResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoUpgradeRequest = request.into();
        let response = client.upgrade(proto_request).await?;
        let inner = response.into_inner();

        Ok(UpgradeResponse::from(inner))
    }

    // =========================================================================
    // Service Management
    // =========================================================================

    /// Start a service.
    pub async fn service_start(
        &self,
        request: ServiceStartRequest,
    ) -> Result<ServiceStartResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoServiceStartRequest = request.into();
        let response = client.service_start(proto_request).await?;
        let inner = response.into_inner();

        Ok(ServiceStartResponse::from(inner))
    }

    /// Stop a service.
    pub async fn service_stop(&self, request: ServiceStopRequest) -> Result<ServiceStopResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoServiceStopRequest = request.into();
        let response = client.service_stop(proto_request).await?;
        let inner = response.into_inner();

        Ok(ServiceStopResponse::from(inner))
    }

    /// Restart a service.
    pub async fn service_restart(
        &self,
        request: ServiceRestartRequest,
    ) -> Result<ServiceRestartResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoServiceRestartRequest = request.into();
        let response = client.service_restart(proto_request).await?;
        let inner = response.into_inner();

        Ok(ServiceRestartResponse::from(inner))
    }

    /// Get service/container logs (server-streaming).
    pub async fn logs(&self, request: LogsRequest) -> Result<LogsResponse> {
        use tonic::codegen::tokio_stream::StreamExt;

        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoLogsRequest = request.into();
        let response = client.logs(proto_request).await?;
        let mut stream = response.into_inner();

        let mut data = Vec::new();
        let mut node = None;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if node.is_none() {
                if let Some(metadata) = &chunk.metadata {
                    node = Some(metadata.hostname.clone());
                }
            }
            data.extend(chunk.bytes);
        }

        Ok(LogsResponse::new(data, node))
    }

    // =========================================================================
    // System Information
    // =========================================================================

    /// Get system load averages.
    pub async fn load_avg(&self) -> Result<LoadAvgResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.load_avg(()).await?;
        let inner = response.into_inner();

        Ok(LoadAvgResponse::from(inner))
    }

    /// Get memory information.
    pub async fn memory(&self) -> Result<MemoryResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.memory(()).await?;
        let inner = response.into_inner();

        Ok(MemoryResponse::from(inner))
    }

    /// Get CPU information.
    pub async fn cpu_info(&self) -> Result<CpuInfoResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.cpu_info(()).await?;
        let inner = response.into_inner();

        Ok(CpuInfoResponse::from(inner))
    }

    /// Get disk statistics.
    pub async fn disk_stats(&self) -> Result<DiskStatsResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.disk_stats(()).await?;
        let inner = response.into_inner();

        Ok(DiskStatsResponse::from(inner))
    }

    /// Get network device statistics.
    pub async fn network_device_stats(&self) -> Result<NetworkDeviceStatsResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.network_device_stats(()).await?;
        let inner = response.into_inner();

        Ok(NetworkDeviceStatsResponse::from(inner))
    }

    /// Get mount points.
    pub async fn mounts(&self) -> Result<MountsResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.mounts(()).await?;
        let inner = response.into_inner();

        Ok(MountsResponse::from(inner))
    }

    /// Get process list.
    pub async fn processes(&self) -> Result<ProcessesResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.processes(()).await?;
        let inner = response.into_inner();

        Ok(ProcessesResponse::from(inner))
    }

    // =========================================================================
    // File Operations
    // =========================================================================

    /// List directory contents (server-streaming).
    pub async fn list(&self, request: ListRequest) -> Result<ListResponse> {
        use tonic::codegen::tokio_stream::StreamExt;

        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoListRequest = request.into();
        let response = client.list(proto_request).await?;
        let mut stream = response.into_inner();

        let mut entries = Vec::new();
        while let Some(info) = stream.next().await {
            let info = info?;
            entries.push(FileInfo::from(info));
        }

        Ok(ListResponse::new(entries))
    }

    /// Read a file (server-streaming).
    pub async fn read(&self, request: ReadRequest) -> Result<ReadResponse> {
        use tonic::codegen::tokio_stream::StreamExt;

        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoReadRequest = request.into();
        let response = client.read(proto_request).await?;
        let mut stream = response.into_inner();

        let mut data = Vec::new();
        let mut node = None;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if node.is_none() {
                if let Some(metadata) = &chunk.metadata {
                    node = Some(metadata.hostname.clone());
                }
            }
            data.extend(chunk.bytes);
        }

        Ok(ReadResponse::new(data, node))
    }

    /// Copy a file or directory as tar archive (server-streaming).
    pub async fn copy(&self, request: CopyRequest) -> Result<CopyResponse> {
        use tonic::codegen::tokio_stream::StreamExt;

        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoCopyRequest = request.into();
        let response = client.copy(proto_request).await?;
        let mut stream = response.into_inner();

        let mut data = Vec::new();
        let mut node = None;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if node.is_none() {
                if let Some(metadata) = &chunk.metadata {
                    node = Some(metadata.hostname.clone());
                }
            }
            data.extend(chunk.bytes);
        }

        Ok(CopyResponse::new(data, node))
    }

    /// Get disk usage (server-streaming).
    pub async fn disk_usage(&self, request: DiskUsageRequest) -> Result<DiskUsageResponse> {
        use tonic::codegen::tokio_stream::StreamExt;

        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoDiskUsageRequest = request.into();
        let response = client.disk_usage(proto_request).await?;
        let mut stream = response.into_inner();

        let mut entries = Vec::new();
        while let Some(info) = stream.next().await {
            let info = info?;
            entries.push(DiskUsageInfo::from(info));
        }

        Ok(DiskUsageResponse::new(entries))
    }

    // =========================================================================
    // Advanced APIs
    // =========================================================================

    /// Rollback a Talos node to the previous installed version.
    pub async fn rollback(&self) -> Result<RollbackResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let response = client.rollback(ProtoRollbackRequest {}).await?;
        let inner = response.into_inner();

        Ok(RollbackResponse::from(inner))
    }

    /// Generate client configuration (talosconfig).
    pub async fn generate_client_configuration(
        &self,
        request: GenerateClientConfigurationRequest,
    ) -> Result<GenerateClientConfigurationResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoGenerateClientConfigRequest = request.into();
        let response = client.generate_client_configuration(proto_request).await?;
        let inner = response.into_inner();

        Ok(GenerateClientConfigurationResponse::from(inner))
    }

    /// Capture packets on a network interface (server-streaming).
    pub async fn packet_capture(
        &self,
        request: PacketCaptureRequest,
    ) -> Result<PacketCaptureResponse> {
        use tonic::codegen::tokio_stream::StreamExt;

        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoPacketCaptureRequest = request.into();
        let response = client.packet_capture(proto_request).await?;
        let mut stream = response.into_inner();

        let mut data = Vec::new();
        let mut node = None;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if node.is_none() {
                if let Some(metadata) = &chunk.metadata {
                    node = Some(metadata.hostname.clone());
                }
            }
            data.extend(chunk.bytes);
        }

        Ok(PacketCaptureResponse::new(data, node))
    }

    /// Get network connection information (netstat).
    pub async fn netstat(&self, request: NetstatRequest) -> Result<NetstatResponse> {
        let mut client = MachineServiceClient::new(self.channel.clone());

        let proto_request: ProtoNetstatRequest = request.into();
        let response = client.netstat(proto_request).await?;
        let inner = response.into_inner();

        Ok(NetstatResponse::from(inner))
    }
}

// Helper for insecure mode
#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}

#[cfg(test)]
mod tests;
