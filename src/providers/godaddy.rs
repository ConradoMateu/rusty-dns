//! GoDaddy DDNS provider.

use super::{DdnsProvider, UpdateResult};
use crate::error::{DdnsError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// GoDaddy DDNS provider.
pub struct GoDaddyProvider {
    client: reqwest::Client,
    api_key: String,
    api_secret: String,
    domain: String,
    name: String,
    ttl: u32,
}

#[derive(Debug, Deserialize)]
struct DnsRecord {
    data: String,
}

#[derive(Debug, Serialize)]
struct UpdateRecord {
    data: String,
    ttl: u32,
}

#[derive(Debug, Deserialize)]
struct GoDaddyError {
    message: String,
}

impl GoDaddyProvider {
    /// Create a new GoDaddy provider.
    pub fn new(
        api_key: String,
        api_secret: String,
        domain: String,
        name: String,
        ttl: u32,
    ) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            api_key,
            api_secret,
            domain,
            name,
            ttl,
        }
    }

    fn full_domain(&self) -> String {
        if self.name == "@" {
            self.domain.clone()
        } else {
            format!("{}.{}", self.name, self.domain)
        }
    }

    fn auth_header(&self) -> String {
        format!("sso-key {}:{}", self.api_key, self.api_secret)
    }
}

#[async_trait]
impl DdnsProvider for GoDaddyProvider {
    fn name(&self) -> &'static str {
        "godaddy"
    }

    fn domain(&self) -> String {
        self.full_domain()
    }

    async fn get_current_ip(&self) -> Result<Option<IpAddr>> {
        let url = format!(
            "https://api.godaddy.com/v1/domains/{}/records/A/{}",
            self.domain, self.name
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let records: Vec<DnsRecord> = response.json().await?;
        Ok(records.first().and_then(|r| r.data.parse().ok()))
    }

    async fn update_ip(&self, ip: IpAddr) -> Result<UpdateResult> {
        let previous_ip = self.get_current_ip().await.ok().flatten();

        let record_type = if ip.is_ipv4() { "A" } else { "AAAA" };
        let url = format!(
            "https://api.godaddy.com/v1/domains/{}/records/{}/{}",
            self.domain, record_type, self.name
        );

        let records = vec![UpdateRecord {
            data: ip.to_string(),
            ttl: self.ttl,
        }];

        let response = self
            .client
            .put(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&records)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(UpdateResult {
                provider: self.name().to_string(),
                domain: self.full_domain(),
                success: true,
                ip: Some(ip),
                previous_ip,
                error: None,
                timestamp: chrono::Utc::now(),
            })
        } else {
            let error: std::result::Result<GoDaddyError, _> = response.json().await;
            let msg = error
                .map(|e| e.message)
                .unwrap_or_else(|_| "Unknown error".to_string());

            Ok(UpdateResult {
                provider: self.name().to_string(),
                domain: self.full_domain(),
                success: false,
                ip: None,
                previous_ip,
                error: Some(msg),
                timestamp: chrono::Utc::now(),
            })
        }
    }

    async fn validate(&self) -> Result<()> {
        let url = format!(
            "https://api.godaddy.com/v1/domains/{}/records/A/{}",
            self.domain, self.name
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            let error: std::result::Result<GoDaddyError, _> = response.json().await;
            let msg = error
                .map(|e| e.message)
                .unwrap_or_else(|_| "Authentication failed".to_string());

            return Err(DdnsError::Provider {
                provider: "godaddy".to_string(),
                message: msg,
            });
        }

        Ok(())
    }
}
