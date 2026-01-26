// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::api::version::version_service_client::VersionServiceClient;
use crate::error::Result;
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};

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
        let endpoint_str = if config.endpoint.starts_with("http") {
            config.endpoint.clone()
        } else {
            format!("https://{}", config.endpoint)
        };

        let mut channel_builder = Channel::from_shared(endpoint_str)
            .map_err(|e| crate::error::TalosError::Config(e.to_string()))?;

        // TLS Configuration
        if config.insecure {
            // Insecure mode: trust all certs via custom connector logic
            let mut tls_config = rustls::ClientConfig::builder()
                .with_root_certificates(rustls::RootCertStore::empty())
                .with_no_client_auth();

            // Override verifier with one that accepts everything
            tls_config
                .dangerous()
                .set_certificate_verifier(Arc::new(NoVerifier));

            // Tonic requires ALPN for gRPC (h2)
            tls_config.alpn_protocols = vec![b"h2".to_vec()];
            let tls_config = Arc::new(tls_config);
            let connector = tokio_rustls::TlsConnector::from(tls_config);

            // For custom connector, we use http:// scheme because we handle TLS ourselves
            // Tonic would reject https:// without its own TLS config
            let endpoint_for_connector = if config.endpoint.starts_with("https://") {
                config.endpoint.replacen("https://", "http://", 1)
            } else if config.endpoint.starts_with("http://") {
                config.endpoint.clone()
            } else {
                format!("http://{}", config.endpoint)
            };

            let channel = Endpoint::from_shared(endpoint_for_connector)
                .map_err(|e| crate::error::TalosError::Config(e.to_string()))?
                .connect_with_connector(tower::service_fn(move |uri: tonic::transport::Uri| {
                    let connector = connector.clone();
                    async move {
                        let host = uri.host().unwrap_or("127.0.0.1");
                        let port = uri.port_u16().unwrap_or(50000);
                        let addr = format!("{}:{}", host, port);

                        let tcp = tokio::net::TcpStream::connect(addr).await?;

                        // We use a dummy server name because verification is disabled anyway
                        let server_name = rustls::pki_types::ServerName::try_from("any").unwrap();

                        let tls_stream = connector.connect(server_name, tcp).await?;
                        Ok::<_, std::io::Error>(TokioIo::new(tls_stream))
                    }
                }))
                .await?;

            return Ok(Self {
                #[allow(dead_code)]
                config,
                channel,
            });
        } else if config.endpoint.starts_with("https") {
            // Strict mode (mTLS)
            let mut tls = ClientTlsConfig::new();

            if let Some(ca_path) = &config.ca_path {
                let pem = std::fs::read_to_string(ca_path).map_err(|e| {
                    crate::error::TalosError::Config(format!("Failed to read CA: {e}"))
                })?;
                tls = tls.ca_certificate(Certificate::from_pem(pem));
            }

            if let (Some(crt), Some(key)) = (&config.crt_path, &config.key_path) {
                let cert_pem = std::fs::read_to_string(crt).map_err(|e| {
                    crate::error::TalosError::Config(format!("Failed to read Cert: {e}"))
                })?;
                let key_pem = std::fs::read_to_string(key).map_err(|e| {
                    crate::error::TalosError::Config(format!("Failed to read Key: {e}"))
                })?;
                tls = tls.identity(Identity::from_pem(cert_pem, key_pem));
            }
            channel_builder = channel_builder.tls_config(tls)?;
        }

        let channel = channel_builder.connect().await?;

        Ok(Self {
            #[allow(dead_code)]
            config,
            channel,
        })
    }

    /// Access the Version API group
    pub fn version(&self) -> VersionServiceClient<Channel> {
        VersionServiceClient::new(self.channel.clone())
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
