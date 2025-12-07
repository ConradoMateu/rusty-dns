//! DuckDNS provider.

use super::{DdnsProvider, UpdateResult};
use crate::error::{DdnsError, Result};
use async_trait::async_trait;
use std::net::IpAddr;

/// DuckDNS provider.
pub struct DuckDnsProvider {
    client: reqwest::Client,
    domains: String,
    token: String,
}

impl DuckDnsProvider {
    /// Create a new DuckDNS provider.
    pub fn new(domains: String, token: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            domains,
            token,
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
            "https://www.duckdns.org/update?domains={}&token={}&ip={}",
            self.domains, self.token, ip
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
