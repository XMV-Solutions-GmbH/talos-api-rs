// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use crate::api::version::{VersionRequest, VersionResponse};
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

#[test]
fn test_default_config() {
    let config = TalosClientConfig::default();
    assert_eq!(config.endpoint, "https://127.0.0.1:50000");
    assert!(!config.insecure);
    assert!(config.crt_path.is_none());
    assert!(config.key_path.is_none());
    assert!(config.ca_path.is_none());
}

#[tokio::test]
async fn test_new_client_invalid_cert_path() {
    let config = TalosClientConfig {
        endpoint: "https://example.com".to_string(),
        crt_path: Some("/nonexistent/path_12345.crt".to_string()),
        key_path: Some("/nonexistent/path_12345.key".to_string()),
        ..Default::default()
    };

    // Should fail immediately because it tries to read the file
    let result = TalosClient::new(config).await;
    assert!(result.is_err());
    match result {
        Err(crate::error::TalosError::Config(msg)) => {
            assert!(msg.contains("Failed to read Cert"));
        }
        _ => panic!("Expected Config error"),
    }
}

#[tokio::test]
async fn test_new_client_insecure_no_connect() {
    let config = TalosClientConfig {
        endpoint: "https://127.0.0.1:54321".to_string(), // Random port
        insecure: true,
        ..Default::default()
    };

    let result = TalosClient::new(config).await;
    // It should fail to connect to the random port
    assert!(result.is_err());
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
        insecure: false, // Testing cleartext http here, which tonic supports by default
    };

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
