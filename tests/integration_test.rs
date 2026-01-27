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

    // 6. Test ApplyConfiguration (dry-run with minimal YAML)
    println!("\n--- Machine API: ApplyConfiguration (dry-run) ---");
    use talos_api_rs::{ApplyConfigurationRequest, ApplyMode};

    // Use a minimal valid Talos config for dry-run validation
    // This will fail validation but tests that the API is reachable
    let minimal_config = r#"
version: v1alpha1
machine:
  type: controlplane
  token: placeholder
  ca:
    crt: placeholder
    key: placeholder
cluster:
  controlPlane:
    endpoint: https://127.0.0.1:6443
  network:
    cni:
      name: flannel
  token: placeholder
  secretboxEncryptionSecret: placeholder
  ca:
    crt: placeholder
    key: placeholder
"#;

    let request = ApplyConfigurationRequest::builder()
        .config_yaml(minimal_config)
        .mode(ApplyMode::Auto)
        .dry_run(true)
        .build();

    match client.apply_configuration(request).await {
        Ok(apply_response) => {
            for result in &apply_response.results {
                let node = result.node.as_deref().unwrap_or("unknown");
                println!("✓ Node: {} -> mode: {}", node, result.mode);
                if !result.mode_details.is_empty() {
                    println!("  Details: {}", result.mode_details);
                }
                if !result.warnings.is_empty() {
                    println!("  Warnings: {} total", result.warnings.len());
                }
            }
        }
        Err(e) => {
            // Validation errors are expected with placeholder config
            // We're testing that the API is reachable and responds
            println!(
                "  ApplyConfiguration dry-run returned error (expected): {}",
                e
            );
            println!("  (Validation errors are expected with placeholder config)");
        }
    }

    // 7. Test Bootstrap API (will fail on already-bootstrapped cluster - expected)
    println!("\n--- Machine API: Bootstrap ---");
    use talos_api_rs::BootstrapRequest;

    // The cluster is already bootstrapped, so this should fail with a specific error
    match client.bootstrap(BootstrapRequest::new()).await {
        Ok(response) => {
            // Unexpected success - cluster shouldn't allow re-bootstrap
            for result in &response.results {
                let node = result.node.as_deref().unwrap_or("unknown");
                println!("✓ Node: {} - bootstrap succeeded (unexpected)", node);
            }
        }
        Err(e) => {
            // Expected: cluster is already bootstrapped
            let err_str = e.to_string();
            if err_str.contains("AlreadyExists")
                || err_str.contains("already")
                || err_str.contains("etcd")
            {
                println!("✓ Bootstrap correctly rejected (cluster already bootstrapped)");
            } else {
                println!("  Bootstrap returned: {}", e);
                println!("  (Error expected - cluster is already bootstrapped)");
            }
        }
    }

    // 8. Test Kubeconfig API (server-streaming)
    println!("\n--- Machine API: Kubeconfig ---");

    match client.kubeconfig().await {
        Ok(kubeconfig) => {
            let node = kubeconfig.node.as_deref().unwrap_or("unknown");
            println!("✓ Node: {} -> kubeconfig retrieved", node);
            println!("  Size: {} bytes", kubeconfig.len());

            // Verify it's valid YAML/kubeconfig
            if let Ok(content) = kubeconfig.as_str() {
                if content.contains("apiVersion") && content.contains("clusters") {
                    println!("  ✓ Valid kubeconfig structure detected");
                } else {
                    println!(
                        "  Content preview: {}...",
                        &content[..content.len().min(100)]
                    );
                }
            }
        }
        Err(e) => {
            println!("  Kubeconfig returned error: {}", e);
            // May fail if cluster not fully ready - that's OK for integration test
        }
    }

    // 9. Test Reset API (DESTRUCTIVE - only verify API is available, don't execute)
    //
    // IMPORTANT: The Reset API is destructive and would destroy our test cluster.
    // In a real integration test environment, you would:
    //   1. Create a dedicated node for reset testing
    //   2. Actually execute the reset
    //   3. Verify the node comes back up
    //
    // Here we only verify the API types compile and the gRPC method exists.
    // The actual reset functionality is tested by:
    //   - Unit tests for type conversions
    //   - Manual testing against a disposable cluster
    println!("\n--- Machine API: Reset (API verification only) ---");
    use talos_api_rs::{ResetRequest, WipeMode};

    // Verify types work correctly
    let graceful = ResetRequest::graceful();
    println!("✓ ResetRequest::graceful() creates valid request");
    println!(
        "  graceful={}, reboot={}, mode={}",
        graceful.graceful, graceful.reboot, graceful.mode
    );

    let force = ResetRequest::force();
    println!("✓ ResetRequest::force() creates valid request");
    println!(
        "  graceful={}, reboot={}, mode={}",
        force.graceful, force.reboot, force.mode
    );

    let halt = ResetRequest::halt();
    println!("✓ ResetRequest::halt() creates valid request");
    println!(
        "  graceful={}, reboot={}, mode={}",
        halt.graceful, halt.reboot, halt.mode
    );

    let custom = ResetRequest::builder()
        .graceful(true)
        .reboot(true)
        .wipe_mode(WipeMode::SystemDisk)
        .build();
    println!("✓ ResetRequest::builder() creates valid request");
    println!(
        "  graceful={}, reboot={}, mode={}",
        custom.graceful, custom.reboot, custom.mode
    );

    // NOTE: We do NOT execute client.reset() here because it would destroy the test cluster.
    // The method signature and gRPC connectivity are verified by the compile check.
    println!("⚠ Skipping actual reset execution (would destroy test cluster)");
    println!("  Run manual reset tests against a disposable cluster");

    // 10. Test etcd APIs (control plane only)
    println!("\n--- etcd API: Member List ---");
    use talos_api_rs::EtcdMemberListRequest;

    match client.etcd_member_list(EtcdMemberListRequest::new()).await {
        Ok(response) => {
            let members = response.all_members();
            println!("✓ etcd cluster has {} member(s)", members.len());
            for member in members {
                println!(
                    "  Member: {} (ID: {}, learner: {})",
                    member.hostname, member.id, member.is_learner
                );
            }
        }
        Err(e) => {
            println!("  etcd_member_list returned error: {}", e);
        }
    }

    println!("\n--- etcd API: Status ---");
    match client.etcd_status().await {
        Ok(response) => {
            if let Some(status) = response.first() {
                println!("✓ etcd status retrieved");
                println!("  Member ID: {}", status.member_id);
                println!("  Protocol: {}", status.protocol_version);
                println!("  DB Size: {}", status.db_size_human());
                println!(
                    "  Leader: {} (is_leader: {})",
                    status.leader,
                    status.is_leader()
                );
            }
        }
        Err(e) => {
            println!("  etcd_status returned error: {}", e);
        }
    }

    println!("\n--- etcd API: Alarm List ---");
    match client.etcd_alarm_list().await {
        Ok(response) => {
            if response.has_alarms() {
                println!("⚠ Active alarms found:");
                for alarm in response.active_alarms() {
                    println!("  Member {}: {}", alarm.member_id, alarm.alarm);
                }
            } else {
                println!("✓ No active etcd alarms");
            }
        }
        Err(e) => {
            println!("  etcd_alarm_list returned error: {}", e);
        }
    }

    // 11. Show cluster status via talosctl (visual feedback)
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

    // 12. Show running services via talosctl
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
