// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::api::machine::machine_service_client::MachineServiceClient;
use crate::api::machine::ApplyConfigurationRequest as ProtoApplyConfigRequest;
use crate::api::version::version_service_client::VersionServiceClient;
use crate::error::Result;
use crate::resources::{ApplyConfigurationRequest, ApplyConfigurationResponse};
use hyper_util::rt::TokioIo;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use std::sync::Arc;
use tonic::transport::{Channel, Endpoint};

#[derive(Clone, Debug)]
pub struct TalosClientConfig {
    pub endpoint: String,
    pub crt_path: Option<String>,
    pub key_path: Option<String>,
    pub ca_path: Option<String>,
    /// If true, skips TLS verification (insecure)
    pub insecure: bool,
}

impl Default for TalosClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://127.0.0.1:50000".to_string(),
            crt_path: None,
            key_path: None,
            ca_path: None,
            insecure: false,
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
        let channel = Channel::from_shared(config.endpoint.clone())
            .map_err(|e| crate::error::TalosError::Config(e.to_string()))?
            .connect()
            .await?;
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

        let channel = Endpoint::from_shared(endpoint_for_connector)
            .map_err(|e| crate::error::TalosError::Config(e.to_string()))?
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
