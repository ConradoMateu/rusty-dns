//! DuckDNS provider.

use super::{DdnsProvider, UpdateResult};
use crate::error::{DdnsError, Result};
use async_trait::async_trait;
use std::net::IpAddr;

const DEFAULT_BASE_URL: &str = "https://www.duckdns.org";

/// DuckDNS provider.
pub struct DuckDnsProvider {
    client: reqwest::Client,
    domains: String,
    token: String,
    base_url: String,
}

impl DuckDnsProvider {
    /// Create a new DuckDNS provider.
    pub fn new(domains: String, token: String) -> Self {
        Self::with_base_url(domains, token, DEFAULT_BASE_URL.to_string())
    }

    /// Create with custom base URL (for testing).
    pub fn with_base_url(domains: String, token: String, base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            domains,
            token,
            base_url,
        }
    }

    fn full_domain(&self) -> String {
        format!(
            "{}.duckdns.org",
            self.domains.split(',').next().unwrap_or(&self.domains)
        )
    }
}

#[async_trait]
impl DdnsProvider for DuckDnsProvider {
    fn name(&self) -> &'static str {
        "duckdns"
    }

    fn domain(&self) -> String {
        self.full_domain()
    }

    async fn get_current_ip(&self) -> Result<Option<IpAddr>> {
        // DuckDNS doesn't provide a way to query current IP
        Ok(None)
    }

    async fn update_ip(&self, ip: IpAddr) -> Result<UpdateResult> {
        let url = format!(
            "{}/update?domains={}&token={}&ip={}",
            self.base_url, self.domains, self.token, ip
        );

        let response = self.client.get(&url).send().await?;
        let text = response.text().await?;

        let success = text.trim() == "OK";

        if success {
            Ok(UpdateResult {
                provider: self.name().to_string(),
                domain: self.full_domain(),
                success: true,
                ip: Some(ip),
                previous_ip: None,
                error: None,
                timestamp: chrono::Utc::now(),
            })
        } else {
            Ok(UpdateResult {
                provider: self.name().to_string(),
                domain: self.full_domain(),
                success: false,
                ip: None,
                previous_ip: None,
                error: Some(format!("DuckDNS returned: {}", text.trim())),
                timestamp: chrono::Utc::now(),
            })
        }
    }

    async fn validate(&self) -> Result<()> {
        if self.token.is_empty() {
            return Err(DdnsError::Provider {
                provider: "duckdns".to_string(),
                message: "Token is empty".to_string(),
            });
        }
        if self.domains.is_empty() {
            return Err(DdnsError::Provider {
                provider: "duckdns".to_string(),
                message: "Domains is empty".to_string(),
            });
        }
        Ok(())
    }
}
