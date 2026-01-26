// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;
use std::env;

pub struct TalosCluster {
    pub name: String,
    pub endpoint: String,
    // Add paths to kubeconfig/talosconfig if needed
}

impl TalosCluster {
    /// Provisions a new local Talos cluster in Docker.
    /// SKIPS if `TALOS_DEV_TESTS` is not set.
    pub fn create(name: &str) -> Option<Self> {
        if env::var("TALOS_DEV_TESTS").is_err() {
            println!("Skipping integration test: TALOS_DEV_TESTS not set");
            return None;
        }
        
        // check if talosctl exists
        if Command::new("talosctl").arg("version").output().is_err() {
            eprintln!("talosctl not found");
            return None;
        }

        println!("Creating Talos cluster '{}'...", name);
        let status = Command::new("talosctl")
            .args(&["cluster", "create", "--name", name, "--provisioner", "docker"]) // use docker explicitly
            .status()
            .expect("Failed to execute talosctl");

        if !status.success() {
             panic!("Failed to create cluster");
        }
        
        // Fetch endpoint? For local docker, it's usually automatic in talosconfig
        // but for the client we need the IP.
        
        Some(Self {
            name: name.to_string(),
            endpoint: "https://127.0.0.1:50000".to_string(), // Simplified assumption for docker
        })
    }
}

impl Drop for TalosCluster {
    fn drop(&mut self) {
        if env::var("TALOS_DEV_TESTS").is_err() {
            return;
        }
        println!("Destroying Talos cluster '{}'...", self.name);
        let _ = Command::new("talosctl")
            .args(&["cluster", "destroy", "--name", &self.name])
            .status();
    }
}
