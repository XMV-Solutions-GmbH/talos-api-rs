// SPDX-License-Identifier: MIT OR Apache-2.0

//! Runtime utilities for resilience and observability.
//!
//! This module provides production-ready features like retry policies,
//! timeouts, and logging interceptors for the Talos API client.

mod retry;

pub use retry::{
    BackoffStrategy, CustomRetryPolicy, DefaultRetryPolicy, ExponentialBackoff, FixedBackoff,
    LinearBackoff, NoBackoff, NoRetryPolicy, RetryConfig, RetryConfigBuilder, RetryPolicy,
};
