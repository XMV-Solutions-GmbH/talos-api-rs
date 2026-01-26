// SPDX-License-Identifier: MIT OR Apache-2.0

use talos_api_rs::{TalosClient, TalosClientConfig, TalosError};

#[tokio::main]
async fn main() -> Result<(), TalosError> {
    let config = TalosClientConfig {
        endpoint: "http://127.0.0.1:50000".to_string(),
        crt_path: None,
        key_path: None,
        ca_path: None,
    };

    println!("Connecting to {}...", config.endpoint);
    // In a real example we would handle the error properly
    // This example expects a running cluster or mock
    match TalosClient::new(config).await {
        Ok(_client) => {
             // let response = client.version().version(...).await?;
             println!("Connected! (Version call not fully implemented in example)");
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
        }
    }

    Ok(())
}
