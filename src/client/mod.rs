// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::api::version::version_service_client::VersionServiceClient;
use crate::error::Result;
use tonic::transport::Channel;

#[derive(Clone, Debug)]
pub struct TalosClientConfig {
    pub endpoint: String,
    pub crt_path: Option<String>,
    pub key_path: Option<String>,
    pub ca_path: Option<String>,
}

#[derive(Clone)]
pub struct TalosClient {
    #[allow(dead_code)] // TODO: Remove when config is used
    config: TalosClientConfig,
    channel: Channel,
}

impl TalosClient {
    pub async fn new(config: TalosClientConfig) -> Result<Self> {
        // Placeholder connection logic
        // In real impl, load certs and build channel with TLS
        let endpoint = config.endpoint.clone();
        let channel = Channel::from_shared(endpoint)
            .map_err(|e| crate::error::TalosError::Config(e.to_string()))?
            .connect()
            .await?;

        Ok(Self { config, channel })
    }

    /// Access the Version API group
    pub fn version(&self) -> VersionServiceClient<Channel> {
        VersionServiceClient::new(self.channel.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::version::{VersionRequest, VersionResponse};
    // Note: The trait and server struct names depend on what tonic-build generates.
    // Usually it acts on the 'package.Service' name.
    // Assuming: impl VersionService for ...
    // And VersionServiceServer
    use crate::api::version::version_service_server::{VersionService, VersionServiceServer};
    use tokio::net::TcpListener;
    use tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server;

    struct MockVersion;

    #[tonic::async_trait]
    impl VersionService for MockVersion {
        async fn version(
            &self,
            _req: tonic::Request<VersionRequest>,
        ) -> std::result::Result<tonic::Response<VersionResponse>, tonic::Status> {
            Ok(tonic::Response::new(VersionResponse {
                tag: "v1.2.3".to_string(),
                sha: "abcdef".to_string(),
            }))
        }
    }

    #[tokio::test]
    async fn test_version_call() {
        // Setup mock server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_future = Server::builder()
            .add_service(VersionServiceServer::new(MockVersion))
            .serve_with_incoming(TcpListenerStream::new(listener));

        tokio::spawn(server_future);

        // Test client
        let config = TalosClientConfig {
            endpoint: format!("http://{}", addr),
            crt_path: None,
            key_path: None,
            ca_path: None,
        };

        // Allow some time for server to start? Usually not needed with spawn
        let client = TalosClient::new(config)
            .await
            .expect("Failed to create client");
        let mut v_client = client.version();

        let response = v_client
            .version(VersionRequest { client: true })
            .await
            .expect("RPC failed");
        assert_eq!(response.get_ref().tag, "v1.2.3");
    }
}
