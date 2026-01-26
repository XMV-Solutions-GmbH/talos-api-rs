// SPDX-License-Identifier: MIT OR Apache-2.0

//! Strongly typed domain wrappers for Talos resources.
//!
//! This module provides ergonomic, type-safe wrappers around the raw
//! protobuf types generated from the Talos API.

mod bootstrap;
mod configuration;
mod kubeconfig;
mod reset;

pub use bootstrap::{
    BootstrapRequest, BootstrapRequestBuilder, BootstrapResponse, BootstrapResult,
};
pub use configuration::{
    ApplyConfigurationRequest, ApplyConfigurationRequestBuilder, ApplyConfigurationResponse,
    ApplyConfigurationResult, ApplyMode,
};
pub use kubeconfig::KubeconfigResponse;
pub use reset::{
    ResetPartitionSpec, ResetRequest, ResetRequestBuilder, ResetResponse, ResetResult, WipeMode,
};
