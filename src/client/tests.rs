// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use crate::api::version::version_service_server::{VersionService, VersionServiceServer};
use crate::api::version::{VersionRequest, VersionResponse};
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
/// Test that machine() method returns a properly typed MachineServiceClient.
/// This test verifies:
/// 1. The machine() method compiles and returns the correct type
/// 2. The client can be created and the machine sub-client accessed
#[tokio::test]
async fn test_machine_client_type() {
    // Create a plain http server (no mock service) to test client creation
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Just spawn an empty server for connection
    let server_future = Server::builder()
        .add_service(VersionServiceServer::new(MockVersion))
        .serve_with_incoming(TcpListenerStream::new(listener));

    tokio::spawn(server_future);

    let config = TalosClientConfig {
        endpoint: format!("http://{}", addr),
        insecure: false,
        ..Default::default()
    };

    let client = TalosClient::new(config)
        .await
        .expect("Failed to create client");

    // Test that machine() returns the correct type
    let _machine_client = client.machine();

    // The type is verified at compile time:
    // MachineServiceClient<Channel>
}

/// Test Machine API request/response type construction.
/// This validates that the generated types can be correctly constructed.
#[test]
fn test_machine_request_response_types() {
    use crate::api::machine::{
        reboot_request, CpuStat, Hostname, HostnameResponse, RebootRequest, RebootResponse,
        ServiceHealth, ServiceInfo, ServiceList, ServiceListResponse, ShutdownRequest,
        ShutdownResponse, SystemStat, SystemStatResponse,
    };

    // RebootRequest
    let reboot_req = RebootRequest {
        mode: reboot_request::Mode::Default as i32,
    };
    assert_eq!(reboot_req.mode, 0);

    let reboot_req_force = RebootRequest {
        mode: reboot_request::Mode::Force as i32,
    };
    assert_eq!(reboot_req_force.mode, 2);

    // ShutdownRequest
    let shutdown_req = ShutdownRequest { force: true };
    assert!(shutdown_req.force);

    // HostnameResponse
    let hostname_resp = HostnameResponse {
        messages: vec![Hostname {
            metadata: None,
            hostname: "talos-node-1".to_string(),
        }],
    };
    assert_eq!(hostname_resp.messages.len(), 1);
    assert_eq!(hostname_resp.messages[0].hostname, "talos-node-1");

    // ServiceListResponse
    let svc_resp = ServiceListResponse {
        messages: vec![ServiceList {
            metadata: None,
            services: vec![
                ServiceInfo {
                    id: "kubelet".to_string(),
                    state: "Running".to_string(),
                    events: None,
                    health: Some(ServiceHealth {
                        unknown: false,
                        healthy: true,
                        last_message: String::new(),
                        last_change: None,
                    }),
                },
                ServiceInfo {
                    id: "containerd".to_string(),
                    state: "Running".to_string(),
                    events: None,
                    health: Some(ServiceHealth {
                        unknown: false,
                        healthy: true,
                        last_message: String::new(),
                        last_change: None,
                    }),
                },
            ],
        }],
    };
    assert_eq!(svc_resp.messages[0].services.len(), 2);
    assert_eq!(svc_resp.messages[0].services[0].id, "kubelet");
    assert!(
        svc_resp.messages[0].services[0]
            .health
            .as_ref()
            .unwrap()
            .healthy
    );

    // SystemStatResponse
    let stat_resp = SystemStatResponse {
        messages: vec![SystemStat {
            metadata: None,
            boot_time: 1700000000,
            cpu_total: Some(CpuStat {
                user: 10.5,
                nice: 0.5,
                system: 5.0,
                idle: 84.0,
                iowait: 0.0,
                irq: 0.0,
                soft_irq: 0.0,
                steal: 0.0,
                guest: 0.0,
                guest_nice: 0.0,
            }),
            cpu: vec![],
            irq_total: 0,
            irq: vec![],
            context_switches: 123456,
            process_created: 100,
            process_running: 5,
            process_blocked: 0,
            soft_irq_total: 0,
            soft_irq: None,
        }],
    };
    assert_eq!(stat_resp.messages[0].boot_time, 1700000000);
    assert_eq!(stat_resp.messages[0].process_running, 5);
    assert!((stat_resp.messages[0].cpu_total.as_ref().unwrap().user - 10.5).abs() < 0.001);

    // RebootResponse / ShutdownResponse (empty check)
    let _reboot_resp = RebootResponse { messages: vec![] };
    let _shutdown_resp = ShutdownResponse { messages: vec![] };
}
