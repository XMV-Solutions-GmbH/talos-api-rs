// SPDX-License-Identifier: MIT OR Apache-2.0

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde::Deserialize;
use base64::prelude::*;

#[derive(Deserialize, Debug)]
struct TalosConfig {
    contexts: std::collections::HashMap<String, ContextConfig>,
}

#[derive(Deserialize, Debug)]
struct ContextConfig {
    target: String,
    ca: String,
    crt: String,
    key: String,
}

pub struct TalosCluster {
    pub name: String,
    pub endpoint: String,
    pub talosconfig_path: PathBuf,
    // Temp dir to hold certs
    _temp_dir: tempfile::TempDir,
    pub ca_path: PathBuf,
    pub crt_path: PathBuf,
    pub key_path: PathBuf,
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

        // Create temp dir for config and certs
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let talosconfig_path = temp_dir.path().join("talosconfig");

        println!("Creating Talos cluster '{}' with config at {:?} ...", name, talosconfig_path);

        // We use --talosconfig to direct the output to our temp file
        // We use 'docker' provisioner
        // Note: 'talosctl cluster create' generally updates the merged config unless --talosconfig is specified?
        // Actually, if --talosconfig file does not exist, it creates it.
        let output = Command::new("talosctl")
            .args([
                "cluster", "create", 
                "--provisioner", "docker", 
                "--name", name,
                "--talosconfig", talosconfig_path.to_str().unwrap()
            ])
            .output()
            .expect("Failed to execute talosctl");

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Pool overlaps") {
                eprintln!("\n\n!!! ERROR: Docker network overlap detected !!!");
                eprintln!("A local Docker network is colliding with the Talos test subnet.");
                eprintln!("Please clean up existing networks with:");
                eprintln!("  docker network prune");
                eprintln!("\nFull error: {}\n", stderr);
            } else {
                eprintln!("talosctl error: {}", stderr);
            }
            panic!("Failed to create cluster");
        }

        // Parse talosconfig
        let config_str = fs::read_to_string(&talosconfig_path).expect("Failed to read talosconfig");
        let config: TalosConfig = serde_yaml::from_str(&config_str).expect("Failed to parse talosconfig");

        let (_, ctx) = config.contexts.iter().next().expect("No context in talosconfig");
        
        // Helper to decode and write
        let decode_and_write = |fname: &str, content: &str| -> PathBuf {
            let bytes = BASE64_STANDARD.decode(content).or_else(|_| BASE64_STANDARD.decode(content.replace('\n', "")))
                .expect("Failed to decode cert");
            let path = temp_dir.path().join(fname);
            fs::write(&path, bytes).expect("Failed to write cert file");
            path
        };

        let ca_path = decode_and_write("ca.crt", &ctx.ca);
        let crt_path = decode_and_write("client.crt", &ctx.crt);
        let key_path = decode_and_write("client.key", &ctx.key);
        
        // Format endpoint
        // Start simple: use what is in target. If it is just IP, add protocol and port.
        let endpoint = if ctx.target.contains("://") {
             ctx.target.clone()
        } else {
             // Basic heuristic
             format!("https://{}:50000", ctx.target)
        };

        Some(Self {
            name: name.to_string(),
            endpoint,
            talosconfig_path,
            _temp_dir: temp_dir,
            ca_path,
            crt_path,
            key_path,
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
