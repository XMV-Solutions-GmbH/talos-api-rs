// SPDX-License-Identifier: MIT OR Apache-2.0

//! Example demonstrating Machine API calls
//!
//! This example shows how to use the Machine API to:
//! - List running services
//! - Get hostname
//! - Get system statistics

use talos_api_rs::{TalosClient, TalosClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For a real Talos cluster, use insecure mode or provide proper mTLS certs
    let config = TalosClientConfig {
        endpoint: "https://127.0.0.1:50000".to_string(),
        insecure: true,
        ..Default::default()
    };

    println!("Connecting to {}...", config.endpoint);

    let client = match TalosClient::new(config).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return Ok(());
        }
    };

    // Get a handle to the Machine service
    let mut machine = client.machine();

    // Get hostname
    println!("\n--- Hostname ---");
    match machine.hostname(()).await {
        Ok(response) => {
            for msg in &response.get_ref().messages {
                let node = msg
                    .metadata
                    .as_ref()
                    .map(|m| m.hostname.as_str())
                    .unwrap_or("unknown");
                println!("Node: {} -> hostname: {}", node, msg.hostname);
            }
        }
        Err(e) => eprintln!("Failed to get hostname: {}", e),
    }

    // List services
    println!("\n--- Services ---");
    match machine.service_list(()).await {
        Ok(response) => {
            for msg in &response.get_ref().messages {
                let node = msg
                    .metadata
                    .as_ref()
                    .map(|m| m.hostname.as_str())
                    .unwrap_or("unknown");
                println!("Node: {}", node);
                for svc in &msg.services {
                    let health = svc
                        .health
                        .as_ref()
                        .map(|h| if h.healthy { "healthy" } else { "unhealthy" })
                        .unwrap_or("unknown");
                    println!("  - {} [state: {}, health: {}]", svc.id, svc.state, health);
                }
            }
        }
        Err(e) => eprintln!("Failed to list services: {}", e),
    }

    // Get system statistics
    println!("\n--- System Statistics ---");
    match machine.system_stat(()).await {
        Ok(response) => {
            for msg in &response.get_ref().messages {
                let node = msg
                    .metadata
                    .as_ref()
                    .map(|m| m.hostname.as_str())
                    .unwrap_or("unknown");
                println!("Node: {}", node);
                println!("  Boot time: {}", msg.boot_time);
                println!("  Processes running: {}", msg.process_running);
                println!("  Processes blocked: {}", msg.process_blocked);
                println!("  Context switches: {}", msg.context_switches);
                if let Some(cpu) = &msg.cpu_total {
                    println!(
                        "  CPU total - user: {:.2}%, system: {:.2}%, idle: {:.2}%",
                        cpu.user, cpu.system, cpu.idle
                    );
                }
            }
        }
        Err(e) => eprintln!("Failed to get system stats: {}", e),
    }

    Ok(())
}
