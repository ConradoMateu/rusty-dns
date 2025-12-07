//! Namecheap DDNS provider.

use super::{DdnsProvider, UpdateResult};
use crate::error::{DdnsError, Result};
use async_trait::async_trait;
use std::net::IpAddr;

/// Namecheap DDNS provider.
pub struct NamecheapProvider {
    client: reqwest::Client,
    domain: String,
    host: String,
    password: String,
}

impl NamecheapProvider {
    /// Create a new Namecheap provider.
    pub fn new(domain: String, host: String, password: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            domain,
            host,
            password,
        }
    }

    fn full_domain(&self) -> String {
        if self.host == "@" {
            self.domain.clone()
        } else {
            format!("{}.{}", self.host, self.domain)
        }
    }
}

#[async_trait]
impl DdnsProvider for NamecheapProvider {
    fn name(&self) -> &'static str {
        "namecheap"
    }

    fn domain(&self) -> String {
        self.full_domain()
    }

    async fn get_current_ip(&self) -> Result<Option<IpAddr>> {
        // Namecheap doesn't provide a way to query current IP
        Ok(None)
    }

    async fn update_ip(&self, ip: IpAddr) -> Result<UpdateResult> {
        let url = format!(
            "https://dynamicdns.park-your-domain.com/update?host={}&domain={}&password={}&ip={}",
            self.host, self.domain, self.password, ip
        );

        let response = self.client.get(&url).send().await?;
        let text = response.text().await?;

        // Namecheap returns XML with <ErrCount>0</ErrCount> on success
        let success = text.contains("<ErrCount>0</ErrCount>");

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
            // Try to extract error message
            let error = if text.contains("<Err1>") {
                text.split("<Err1>")
                    .nth(1)
                    .and_then(|s| s.split("</Err1>").next())
                    .map(|s| s.to_string())
            } else {
                Some("Unknown error".to_string())
            };

            Ok(UpdateResult {
                provider: self.name().to_string(),
                domain: self.full_domain(),
                success: false,
                ip: None,
                previous_ip: None,
                error,
                timestamp: chrono::Utc::now(),
            })
        }
    }

    async fn validate(&self) -> Result<()> {
        // Namecheap doesn't have a validation endpoint
        // We just check that credentials are not empty
        if self.password.is_empty() {
            return Err(DdnsError::Provider {
                provider: "namecheap".to_string(),
                message: "Password is empty".to_string(),
            });
        }
        Ok(())
    }
}
