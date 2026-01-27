// SPDX-License-Identifier: MIT OR Apache-2.0

//! Configuration management for Talos clients
//!
//! This module provides utilities for managing Talos client configuration,
//! including parsing talosctl config files.

mod talosconfig;

pub use talosconfig::{TalosConfig, TalosContext};
