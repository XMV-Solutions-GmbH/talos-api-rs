// SPDX-License-Identifier: MIT OR Apache-2.0

//! Configuration management for Talos clients
//!
//! This module provides utilities for managing Talos client configuration,
//! including parsing talosctl config files.
//!
//! # Environment Variables
//!
//! The following environment variables are supported:
//!
//! - `TALOSCONFIG` - Path to the talosconfig file (default: `~/.talos/config`)
//! - `TALOS_CONTEXT` - Override the active context
//! - `TALOS_ENDPOINTS` - Override endpoints (comma-separated)
//! - `TALOS_NODES` - Target specific nodes (comma-separated)
//!
//! # Example
//!
//! ```no_run
//! use talos_api_rs::config::TalosConfig;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load with environment variable overrides
//! let config = TalosConfig::load_with_env()?;
//!
//! if let Some(ctx) = config.active_context() {
//!     println!("Using endpoints: {:?}", ctx.endpoints);
//! }
//! # Ok(())
//! # }
//! ```

mod talosconfig;

pub use talosconfig::{
    TalosConfig, TalosContext, ENV_TALOSCONFIG, ENV_TALOS_CONTEXT, ENV_TALOS_ENDPOINTS,
    ENV_TALOS_NODES,
};
