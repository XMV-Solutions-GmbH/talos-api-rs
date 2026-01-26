// SPDX-License-Identifier: MIT OR Apache-2.0

use talos_api_rs::testkit::TalosCluster;
use talos_api_rs::{TalosClient, TalosClientConfig};

#[tokio::test]
async fn test_cluster_lifecycle() {
    // 1. Setup Cluster
    let cluster = match TalosCluster::create("talos-dev-integration") {
        Some(c) => c,
        None => {
            println!("Skipping integration test (TALOS_DEV_TESTS not set)");
            return;
        }
    };

    println!("Cluster provisioned at {}", cluster.endpoint);

    // 2. Test Secure Connection
    let config = TalosClientConfig {
        endpoint: cluster.endpoint.clone(),
        ca_path: Some(cluster.ca_path.to_str().unwrap().to_string()),
        crt_path: Some(cluster.crt_path.to_str().unwrap().to_string()),
        key_path: Some(cluster.key_path.to_str().unwrap().to_string()),
        insecure: false,
    };

    let client = TalosClient::new(config).await.expect("Failed to connect securely");
    let version = client.version().version(talos_api_rs::api::version::VersionRequest { client: false }).await;
    
    assert!(version.is_ok(), "Secure version call failed: {:?}", version.err());
    let v_resp = version.unwrap().into_inner();
    println!("Server Version (Secure): {}", v_resp.tag);

    // 3. Test Insecure Connection (TLS verify skipped, No Client Auth)
    // The Version API might require auth, so this might fail with specific gRPC error, 
    // but the connection itself should succeed (no TLS error).
    let insecure_config = TalosClientConfig {
        endpoint: cluster.endpoint.clone(),
        crt_path: None,
        key_path: None,
        ca_path: None,
        insecure: true,
    };

    let client_insecure = TalosClient::new(insecure_config).await.expect("Failed to connect insecurely");
    let version_insecure = client_insecure.version().version(talos_api_rs::api::version::VersionRequest { client: false }).await;
    
    match version_insecure {
        Ok(v) => println!("Server Version (Insecure): {}", v.get_ref().tag),
        Err(status) => {
            println!("Insecure call status: {:?}", status);
        }
    }
}
