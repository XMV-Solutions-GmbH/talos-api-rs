// SPDX-License-Identifier: MIT OR Apache-2.0

use std::env;
use std::process::Command;

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

        println!("Creating Talos cluster '{}' ...", name);

        let output = Command::new("talosctl")
            .args(["cluster", "create", "docker", "--name", name])
            .output()
            .expect("Failed to execute talosctl");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Pool overlaps") {
                eprintln!("\n\n!!! ERROR: Docker network overlap detected !!!");
                eprintln!("A local Docker network is colliding with the Talos test subnet.");
                eprintln!("Please clean up existing networks with:");
                eprintln!("  docker network prune");
                eprintln!(
                    "  # OR remove specific conflicting networks (check 'docker network ls')"
                );
                eprintln!("\nFull error: {}\n", stderr);
            } else {
                eprintln!("talosctl error: {}", stderr);
            }
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
            .args(["cluster", "destroy", "--name", &self.name])
            .status();
    }
}
