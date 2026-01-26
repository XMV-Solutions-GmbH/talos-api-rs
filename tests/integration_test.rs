// SPDX-License-Identifier: MIT OR Apache-2.0

use talos_api_rs::testkit::TalosCluster;

#[tokio::test]
async fn test_cluster_lifecycle() {
    // This looks for TALOS_DEV_TESTS env var. 
    // If not present, returns None and we skip gracefully.
    let cluster = match TalosCluster::create("talos-dev-integration") {
        Some(c) => c,
        None => return, // Skip test
    };
    
    // In a real scenario, we would configure the client here:
    // let client = TalosClient::new(..., cluster.endpoint).await...
    // let version = client.version()...
    
    println!("Cluster {} is running at {}", cluster.name, cluster.endpoint);
    
    // Cluster is destroyed on drop
}
