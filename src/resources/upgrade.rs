// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for the Upgrade API.
//!
//! Provides functionality to upgrade Talos nodes to a new version.

use crate::api::generated::machine::{
    Upgrade as ProtoUpgrade, UpgradeRequest as ProtoUpgradeRequest,
    UpgradeResponse as ProtoUpgradeResponse,
};

/// Reboot mode for upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpgradeRebootMode {
    /// Default reboot mode.
    #[default]
    Default,
    /// Power cycle instead of reboot.
    PowerCycle,
}

impl From<UpgradeRebootMode> for i32 {
    fn from(mode: UpgradeRebootMode) -> Self {
        match mode {
            UpgradeRebootMode::Default => 0,
            UpgradeRebootMode::PowerCycle => 1,
        }
    }
}

impl std::fmt::Display for UpgradeRebootMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpgradeRebootMode::Default => write!(f, "default"),
            UpgradeRebootMode::PowerCycle => write!(f, "powercycle"),
        }
    }
}

/// Request to upgrade a Talos node.
///
/// # Example
///
/// ```no_run
/// use talos_api::resources::UpgradeRequest;
///
/// // Upgrade to a specific version
/// let request = UpgradeRequest::new("ghcr.io/siderolabs/installer:v1.6.0");
///
/// // Staged upgrade (downloads image but doesn't apply until reboot)
/// let request = UpgradeRequest::builder("ghcr.io/siderolabs/installer:v1.6.0")
///     .stage(true)
///     .preserve(true)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct UpgradeRequest {
    /// The upgrade image reference.
    pub image: String,
    /// Preserve data across the upgrade.
    pub preserve: bool,
    /// Stage the upgrade (don't apply immediately).
    pub stage: bool,
    /// Force upgrade even if already on same version.
    pub force: bool,
    /// Reboot mode.
    pub reboot_mode: UpgradeRebootMode,
}

impl UpgradeRequest {
    /// Create a new upgrade request with the given image.
    #[must_use]
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            preserve: false,
            stage: false,
            force: false,
            reboot_mode: UpgradeRebootMode::Default,
        }
    }

    /// Create a builder for customizing the upgrade request.
    #[must_use]
    pub fn builder(image: impl Into<String>) -> UpgradeRequestBuilder {
        UpgradeRequestBuilder::new(image)
    }
}

impl From<UpgradeRequest> for ProtoUpgradeRequest {
    fn from(req: UpgradeRequest) -> Self {
        Self {
            image: req.image,
            preserve: req.preserve,
            stage: req.stage,
            force: req.force,
            reboot_mode: req.reboot_mode.into(),
        }
    }
}

/// Builder for UpgradeRequest.
#[derive(Debug, Clone)]
pub struct UpgradeRequestBuilder {
    image: String,
    preserve: bool,
    stage: bool,
    force: bool,
    reboot_mode: UpgradeRebootMode,
}

impl UpgradeRequestBuilder {
    /// Create a new builder with the given image.
    #[must_use]
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            preserve: false,
            stage: false,
            force: false,
            reboot_mode: UpgradeRebootMode::Default,
        }
    }

    /// Preserve data across the upgrade.
    #[must_use]
    pub fn preserve(mut self, preserve: bool) -> Self {
        self.preserve = preserve;
        self
    }

    /// Stage the upgrade (downloads image but doesn't apply until reboot).
    #[must_use]
    pub fn stage(mut self, stage: bool) -> Self {
        self.stage = stage;
        self
    }

    /// Force upgrade even if already on same version.
    #[must_use]
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Set the reboot mode.
    #[must_use]
    pub fn reboot_mode(mut self, mode: UpgradeRebootMode) -> Self {
        self.reboot_mode = mode;
        self
    }

    /// Build the request.
    #[must_use]
    pub fn build(self) -> UpgradeRequest {
        UpgradeRequest {
            image: self.image,
            preserve: self.preserve,
            stage: self.stage,
            force: self.force,
            reboot_mode: self.reboot_mode,
        }
    }
}

/// Result from an upgrade operation.
#[derive(Debug, Clone)]
pub struct UpgradeResult {
    /// Node that processed the upgrade.
    pub node: Option<String>,
    /// Acknowledgement message.
    pub ack: String,
    /// Actor ID that triggered the upgrade.
    pub actor_id: String,
}

impl From<ProtoUpgrade> for UpgradeResult {
    fn from(proto: ProtoUpgrade) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            ack: proto.ack,
            actor_id: proto.actor_id,
        }
    }
}

/// Response from an upgrade operation.
#[derive(Debug, Clone)]
pub struct UpgradeResponse {
    /// Results from each node.
    pub results: Vec<UpgradeResult>,
}

impl From<ProtoUpgradeResponse> for UpgradeResponse {
    fn from(proto: ProtoUpgradeResponse) -> Self {
        Self {
            results: proto.messages.into_iter().map(UpgradeResult::from).collect(),
        }
    }
}

impl UpgradeResponse {
    /// Check if the upgrade was initiated successfully.
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.results.is_empty()
    }

    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&UpgradeResult> {
        self.results.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_request_new() {
        let req = UpgradeRequest::new("ghcr.io/siderolabs/installer:v1.6.0");
        assert_eq!(req.image, "ghcr.io/siderolabs/installer:v1.6.0");
        assert!(!req.preserve);
        assert!(!req.stage);
        assert!(!req.force);
        assert_eq!(req.reboot_mode, UpgradeRebootMode::Default);
    }

    #[test]
    fn test_upgrade_request_builder() {
        let req = UpgradeRequest::builder("ghcr.io/siderolabs/installer:v1.6.0")
            .preserve(true)
            .stage(true)
            .force(true)
            .reboot_mode(UpgradeRebootMode::PowerCycle)
            .build();

        assert_eq!(req.image, "ghcr.io/siderolabs/installer:v1.6.0");
        assert!(req.preserve);
        assert!(req.stage);
        assert!(req.force);
        assert_eq!(req.reboot_mode, UpgradeRebootMode::PowerCycle);
    }

    #[test]
    fn test_upgrade_reboot_mode() {
        assert_eq!(i32::from(UpgradeRebootMode::Default), 0);
        assert_eq!(i32::from(UpgradeRebootMode::PowerCycle), 1);
        assert_eq!(UpgradeRebootMode::PowerCycle.to_string(), "powercycle");
    }

    #[test]
    fn test_proto_conversion() {
        let req = UpgradeRequest::builder("test:v1.0")
            .stage(true)
            .force(true)
            .build();

        let proto: ProtoUpgradeRequest = req.into();
        assert_eq!(proto.image, "test:v1.0");
        assert!(proto.stage);
        assert!(proto.force);
    }
}
