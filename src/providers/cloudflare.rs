//! Cloudflare DDNS provider.

use super::{DdnsProvider, UpdateResult};
use crate::error::{DdnsError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

const DEFAULT_BASE_URL: &str = "https://api.cloudflare.com";

/// Cloudflare DDNS provider.
pub struct CloudflareProvider {
    client: reqwest::Client,
    api_token: String,
    zone_id: String,
    record_name: String,
    proxied: bool,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    result: Option<T>,
    errors: Vec<CloudflareError>,
}

#[derive(Debug, Deserialize)]
struct CloudflareError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct DnsRecord {
    id: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct UpdateRequest {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    proxied: bool,
}

impl CloudflareProvider {
    /// Create a new Cloudflare provider.
    pub fn new(api_token: String, zone_id: String, record_name: String, proxied: bool) -> Self {
        Self::with_base_url(api_token, zone_id, record_name, proxied, DEFAULT_BASE_URL.to_string())
    }

    /// Create with custom base URL (for testing).
    pub fn with_base_url(
        api_token: String,
        zone_id: String,
        record_name: String,
        proxied: bool,
        base_url: String,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_token,
            zone_id,
            record_name,
            proxied,
            base_url,
        }
    }

    /// Get the DNS record ID.
    async fn get_record_id(&self) -> Result<(String, String)> {
        let url = format!(
            "{}/client/v4/zones/{}/dns_records?name={}",
            self.base_url, self.zone_id, self.record_name
        );

        let response: CloudflareResponse<Vec<DnsRecord>> = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?
            .json()
            .await?;

        if !response.success {
            let msg = response
                .errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(DdnsError::Provider {
                provider: "cloudflare".to_string(),
                message: msg,
            });
        }

        response
            .result
            .and_then(|records| records.into_iter().next())
            .map(|r| (r.id, r.content))
            .ok_or_else(|| DdnsError::Provider {
                provider: "cloudflare".to_string(),
                message: format!("DNS record {} not found", self.record_name),
            })
    }
}

#[async_trait]
impl DdnsProvider for CloudflareProvider {
    fn name(&self) -> &'static str {
        "cloudflare"
    }

    fn domain(&self) -> String {
        self.record_name.clone()
    }

    async fn get_current_ip(&self) -> Result<Option<IpAddr>> {
        let (_, content) = self.get_record_id().await?;
        Ok(content.parse().ok())
    }

    async fn update_ip(&self, ip: IpAddr) -> Result<UpdateResult> {
        let previous_ip = self.get_current_ip().await.ok().flatten();

        let (record_id, _) = self.get_record_id().await?;

        let url = format!(
            "{}/client/v4/zones/{}/dns_records/{}",
            self.base_url, self.zone_id, record_id
        );

        let record_type = if ip.is_ipv4() { "A" } else { "AAAA" };

        let request = UpdateRequest {
            record_type: record_type.to_string(),
            name: self.record_name.clone(),
            content: ip.to_string(),
            proxied: self.proxied,
        };

        let response: CloudflareResponse<DnsRecord> = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        if response.success {
            Ok(UpdateResult {
                provider: self.name().to_string(),
                domain: self.record_name.clone(),
                success: true,
                ip: Some(ip),
                previous_ip,
                error: None,
                timestamp: chrono::Utc::now(),
            })
        } else {
            let msg = response
                .errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown error".to_string());

            Ok(UpdateResult {
                provider: self.name().to_string(),
                domain: self.record_name.clone(),
                success: false,
                ip: None,
                previous_ip,
                error: Some(msg),
                timestamp: chrono::Utc::now(),
            })
        }
    }

    async fn validate(&self) -> Result<()> {
        // Try to get the record to validate credentials
        self.get_record_id().await?;
        Ok(())
    }
}
