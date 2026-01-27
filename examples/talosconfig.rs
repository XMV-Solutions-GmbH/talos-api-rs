// SPDX-License-Identifier: MIT OR Apache-2.0

//! Example demonstrating how to use talosctl config file parsing
//!
//! This example shows how to:
//! - Load and parse ~/.talos/config
//! - Access context information
//! - Extract endpoints and certificates
//! - Create a TalosClient from config context

use talos_api_rs::config::TalosConfig;
use talos_api_rs::{TalosClient, TalosClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Talosctl Config File Parsing Example ===\n");

    // Load config from default location (~/.talos/config)
    // This respects the TALOSCONFIG environment variable if set
    match TalosConfig::load_default() {
        Ok(config) => {
            println!("✓ Loaded talosctl config");

            // Show active context
            if let Some(context_name) = &config.context {
                println!("  Active context: {}", context_name);

                if let Some(ctx) = config.active_context() {
                    println!("  Endpoints: {:?}", ctx.endpoints);

                    if let Some(nodes) = &ctx.nodes {
                        println!("  Nodes: {:?}", nodes);
                    }

                    println!("  Has CA cert: {}", ctx.ca.is_some());
                    println!("  Has client cert: {}", ctx.crt.is_some());
                    println!("  Has client key: {}", ctx.key.is_some());

                    // Example: Create a client using the first endpoint
                    if let Some(endpoint) = ctx.first_endpoint() {
                        println!("\n=== Creating TalosClient ===");

                        // For demonstration, we'll create an insecure client
                        // In production, you would write the certs to temp files
                        // and use them for mTLS
                        let client_config = TalosClientConfig {
                            endpoint: format!("https://{}", endpoint),
                            insecure: true, // For demo only!
                            ..Default::default()
                        };

                        match TalosClient::new(client_config).await {
                            Ok(_client) => {
                                println!("✓ Successfully created TalosClient");
                                println!("  Endpoint: https://{}", endpoint);
                            }
                            Err(e) => {
                                println!("✗ Failed to create client: {}", e);
                            }
                        }
                    }
                }
            } else {
                println!("  No active context set");
            }

            // List all available contexts
            println!("\n=== Available Contexts ===");
            let context_names = config.context_names();
            for name in context_names {
                println!("  - {}", name);
                if let Some(ctx) = config.get_context(name) {
                    println!("    Endpoints: {:?}", ctx.endpoints);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to load config: {}", e);
            println!("\nNote: This example requires a valid talosctl config file.");
            println!("You can create one with: talosctl gen config <cluster-name> <endpoint>");
        }
    }

    Ok(())
}
