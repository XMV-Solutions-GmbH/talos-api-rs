// SPDX-License-Identifier: MIT OR Apache-2.0

use thiserror::Error;

#[allow(clippy::result_large_err)]
#[derive(Debug, Error)]
pub enum TalosError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API request failed: {0}")]
    Api(#[from] tonic::Status),

    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Circuit breaker is open: {0}")]
    CircuitOpen(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, TalosError>;
