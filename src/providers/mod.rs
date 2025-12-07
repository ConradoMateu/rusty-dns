//! DDNS provider implementations.

mod cloudflare;
mod duckdns;
mod godaddy;
mod namecheap;

pub use cloudflare::CloudflareProvider;
pub use duckdns::DuckDnsProvider;
pub use godaddy::GoDaddyProvider;
pub use namecheap::NamecheapProvider;

use crate::config::ProviderConfig;
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Result of a DNS update operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    /// Provider name.
    pub provider: String,
    /// Domain/record that was updated.
    pub domain: String,
    /// Whether the update was successful.
    pub success: bool,
    /// New IP address.
    pub ip: Option<IpAddr>,
    /// Previous IP address (if known).
    pub previous_ip: Option<IpAddr>,
    /// Error message if failed.
    pub error: Option<String>,
    /// Timestamp of the update.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Trait for DDNS providers.
#[async_trait]
pub trait DdnsProvider: Send + Sync {
    /// Get the provider name.
    fn name(&self) -> &'static str;

    /// Get the domain being managed.
    fn domain(&self) -> String;

    /// Get the current DNS record IP (if available).
    async fn get_current_ip(&self) -> Result<Option<IpAddr>>;

    /// Update the DNS record to the new IP.
    async fn update_ip(&self, ip: IpAddr) -> Result<UpdateResult>;

    /// Validate provider configuration/credentials.
    async fn validate(&self) -> Result<()>;
}

/// Create a provider from configuration.
pub fn create_provider(config: &ProviderConfig) -> Box<dyn DdnsProvider> {
    match config {
        ProviderConfig::Cloudflare {
            api_token,
            zone_id,
            record_name,
            proxied,
        } => Box::new(CloudflareProvider::new(
            resolve_env(api_token),
            zone_id.clone(),
            record_name.clone(),
            *proxied,
        )),
        ProviderConfig::Namecheap {
            domain,
            host,
            password,
        } => Box::new(NamecheapProvider::new(
            domain.clone(),
            host.clone(),
            resolve_env(password),
        )),
        ProviderConfig::DuckDns { domains, token } => {
            Box::new(DuckDnsProvider::new(domains.clone(), resolve_env(token)))
        }
        ProviderConfig::GoDaddy {
            api_key,
            api_secret,
            domain,
            name,
            ttl,
        } => Box::new(GoDaddyProvider::new(
            resolve_env(api_key),
            resolve_env(api_secret),
            domain.clone(),
            name.clone(),
            *ttl,
        )),
    }
}

/// Resolve environment variable references (values starting with $).
fn resolve_env(value: &str) -> String {
    if let Some(var_name) = value.strip_prefix('$') {
        std::env::var(var_name).unwrap_or_else(|_| {
            tracing::warn!("Environment variable {} not set", var_name);
            value.to_string()
        })
    } else {
        value.to_string()
    }
}
