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

    // 2. Test Insecure Connection (TLS verify skipped)
    // Note: Talos uses ED25519 client certs which tonic's default PEM parser 
    // doesn't support out of the box. So we test insecure mode first.
    let insecure_config = TalosClientConfig {
        endpoint: cluster.endpoint.clone(),
        crt_path: None,
        key_path: None,
        ca_path: None,
        insecure: true,
    };

    let client = TalosClient::new(insecure_config).await.expect("Failed to connect insecurely");
    let mut version_client = client.version();
    let version = version_client.version(talos_api_rs::api::version::VersionRequest { client: false }).await;
    
    match &version {
        Ok(v) => println!("Server Version (Insecure): {}", v.get_ref().tag),
        Err(status) => {
            // Connection worked but API might require auth - that's expected
            println!("Insecure call returned: {:?}", status);
        }
    }
    
    // The connection should have succeeded (no TLS handshake failure)
    // Even if the API returns an error, the transport layer worked
    assert!(version.is_ok() || version.as_ref().unwrap_err().code() != tonic::Code::Unavailable,
        "Transport failed unexpectedly: {:?}", version.err());
}

