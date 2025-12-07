//! Error types for rusty-dns.

use thiserror::Error;

/// Result type alias for rusty-dns.
pub type Result<T> = std::result::Result<T, DdnsError>;

/// DDNS error types.
#[derive(Error, Debug)]
pub enum DdnsError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network/HTTP error.
    #[error("Network error: {0}")]
    Network(String),

    /// Provider-specific error.
    #[error("Provider error ({provider}): {message}")]
    Provider { provider: String, message: String },

    /// IP detection error.
    #[error("IP detection failed: {0}")]
    IpDetection(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<reqwest::Error> for DdnsError {
    fn from(e: reqwest::Error) -> Self {
        DdnsError::Network(e.to_string())
    }
}

impl From<toml::de::Error> for DdnsError {
    fn from(e: toml::de::Error) -> Self {
        DdnsError::Config(e.to_string())
    }
}

impl From<toml::ser::Error> for DdnsError {
    fn from(e: toml::ser::Error) -> Self {
        DdnsError::Serialization(e.to_string())
    }
}

impl From<serde_json::Error> for DdnsError {
    fn from(e: serde_json::Error) -> Self {
        DdnsError::Serialization(e.to_string())
    }
}
