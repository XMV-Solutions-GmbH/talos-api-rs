// SPDX-License-Identifier: MIT OR Apache-2.0
//! Quick test for mTLS with ED25519 certificates

use talos_api_rs::{TalosClient, TalosClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TalosClientConfig::builder("https://127.0.0.1:52149")
        .ca_cert("/tmp/talos-certs/ca.crt")
        .client_cert("/tmp/talos-certs/client.crt")
        .client_key("/tmp/talos-certs/client.key")
        .build();

    println!("Creating mTLS client with ED25519 certs...");
    let client = TalosClient::new(config).await?;

    // Version is part of MachineService in Talos, not a separate VersionService
    println!("Calling MachineService.Version API...");
    let mut m = client.machine();
    let resp = m.version(()).await?;
    for msg in &resp.get_ref().messages {
        if let Some(v) = &msg.version {
            println!("✅ Version: {}", v.tag);
        }
    }

    println!("\nCalling MachineService.Hostname API...");
    let resp = m.hostname(()).await?;
    for msg in &resp.get_ref().messages {
        println!("✅ Hostname: {}", msg.hostname);
    }

    println!("\n🎉 mTLS with ED25519 certificates works!");
    Ok(())
}
