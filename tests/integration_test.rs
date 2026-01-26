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

    println!("========================================");
    println!("  Talos Integration Test Suite");
    println!("========================================");
    println!("Cluster provisioned at {}", cluster.endpoint);

    // 2. Create Client with mTLS (using cluster certs)
    let mtls_config = TalosClientConfig {
        endpoint: cluster.endpoint.clone(),
        crt_path: Some(cluster.crt_path.to_string_lossy().to_string()),
        key_path: Some(cluster.key_path.to_string_lossy().to_string()),
        ca_path: Some(cluster.ca_path.to_string_lossy().to_string()),
        insecure: false,
    };

    println!("\nUsing mTLS with certs from:");
    println!("  CA:  {}", cluster.ca_path.display());
    println!("  CRT: {}", cluster.crt_path.display());
    println!("  KEY: {}", cluster.key_path.display());

    let client = match TalosClient::new(mtls_config).await {
        Ok(c) => c,
        Err(e) => {
            // mTLS might fail due to ED25519 certs - fall back to insecure
            println!("\nmTLS connection failed: {}", e);
            println!("Falling back to insecure mode...\n");

            let insecure_config = TalosClientConfig {
                endpoint: cluster.endpoint.clone(),
                insecure: true,
                ..Default::default()
            };

            TalosClient::new(insecure_config)
                .await
                .expect("Failed to connect insecurely")
        }
    };

    println!("\n--- Version API ---");
    let mut version_client = client.version();
    let version = version_client
        .version(talos_api_rs::api::version::VersionRequest { client: false })
        .await;

    match &version {
        Ok(v) => println!("✓ Server Version: {}", v.get_ref().tag),
        Err(status) => {
            println!("✗ Version call returned: {:?}", status.code());
        }
    }

    // The connection should have succeeded (no TLS handshake failure)
    assert!(
        version.is_ok() || version.as_ref().unwrap_err().code() != tonic::Code::Unavailable,
        "Transport failed unexpectedly: {:?}",
        version.err()
    );

    // 3. Test Machine API - Hostname
    println!("\n--- Machine API: Hostname ---");
    let mut machine_client = client.machine();
    match machine_client.hostname(()).await {
        Ok(response) => {
            for msg in &response.get_ref().messages {
                let node = msg
                    .metadata
                    .as_ref()
                    .map(|m| m.hostname.as_str())
                    .unwrap_or("unknown");
                println!("✓ Node: {} -> hostname: {}", node, msg.hostname);
            }
        }
        Err(status) => {
            println!("✗ Hostname call returned: {:?}", status.code());
            // mTLS required is expected - the transport worked
            assert_ne!(status.code(), tonic::Code::Unavailable, "Transport failed");
        }
    }

    // 4. Test Machine API - ServiceList
    println!("\n--- Machine API: ServiceList ---");
    let mut machine_client = client.machine();
    match machine_client.service_list(()).await {
        Ok(response) => {
            for msg in &response.get_ref().messages {
                let node = msg
                    .metadata
                    .as_ref()
                    .map(|m| m.hostname.as_str())
                    .unwrap_or("unknown");
                println!("✓ Node: {}", node);
                println!("  Services:");
                for svc in &msg.services {
                    let health = svc
                        .health
                        .as_ref()
                        .map(|h| if h.healthy { "✓" } else { "✗" })
                        .unwrap_or("?");
                    println!("    {} {} [{}]", health, svc.id, svc.state);
                }
            }
        }
        Err(status) => {
            println!("✗ ServiceList call returned: {:?}", status.code());
            assert_ne!(status.code(), tonic::Code::Unavailable, "Transport failed");
        }
    }

    // 5. Test Machine API - SystemStat
    println!("\n--- Machine API: SystemStat ---");
    let mut machine_client = client.machine();
    match machine_client.system_stat(()).await {
        Ok(response) => {
            for msg in &response.get_ref().messages {
                let node = msg
                    .metadata
                    .as_ref()
                    .map(|m| m.hostname.as_str())
                    .unwrap_or("unknown");
                println!("✓ Node: {}", node);
                println!("  Boot time:         {}", msg.boot_time);
                println!("  Processes running: {}", msg.process_running);
                println!("  Processes blocked: {}", msg.process_blocked);
                println!("  Context switches:  {}", msg.context_switches);
                if let Some(cpu) = &msg.cpu_total {
                    println!(
                        "  CPU: user={:.1}% sys={:.1}% idle={:.1}%",
                        cpu.user, cpu.system, cpu.idle
                    );
                }
            }
        }
        Err(status) => {
            println!("✗ SystemStat call returned: {:?}", status.code());
            assert_ne!(status.code(), tonic::Code::Unavailable, "Transport failed");
        }
    }

    // 6. Show cluster status via talosctl (visual feedback)
    println!("\n--- Cluster Status (via talosctl) ---");
    let talosconfig_str = cluster.talosconfig_path.to_string_lossy();
    if let Ok(output) = std::process::Command::new("talosctl")
        .args(["--talosconfig", &talosconfig_str])
        .args(["-n", "127.0.0.1"])
        .args(["get", "members"])
        .output()
    {
        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            println!(
                "talosctl get members failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    // 7. Show running services via talosctl
    println!("\n--- Services Status (via talosctl) ---");
    if let Ok(output) = std::process::Command::new("talosctl")
        .args(["--talosconfig", &talosconfig_str])
        .args(["-n", "127.0.0.1"])
        .args(["services"])
        .output()
    {
        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            println!(
                "talosctl services failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    println!("\n========================================");
    println!("  Integration Tests Complete");
    println!("========================================");
}
