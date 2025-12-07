//! Public IP detection.

use crate::error::{DdnsError, Result};
use std::net::IpAddr;
use std::time::Duration;

/// IP detector with multiple fallback services.
pub struct IpDetector {
    client: reqwest::Client,
    services: Vec<String>,
}

impl IpDetector {
    /// Create a new IP detector with default services.
    pub fn new() -> Self {
        Self::with_services(vec![
            "https://api.ipify.org".to_string(),
            "https://icanhazip.com".to_string(),
            "https://ifconfig.me/ip".to_string(),
            "https://ipecho.net/plain".to_string(),
        ])
    }

    /// Create a new IP detector with custom services.
    pub fn with_services(services: Vec<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, services }
    }

    /// Detect public IPv4 address.
    pub async fn detect_ipv4(&self) -> Result<IpAddr> {
        for service in &self.services {
            match self.try_service(service).await {
                Ok(ip) => {
                    if ip.is_ipv4() {
                        tracing::debug!("Detected IPv4 {} from {}", ip, service);
                        return Ok(ip);
                    }
                }
                Err(e) => {
                    tracing::warn!("Service {} failed: {}", service, e);
                }
            }
        }

        Err(DdnsError::IpDetection(
            "All IP detection services failed".to_string(),
        ))
    }

    /// Detect public IPv6 address.
    pub async fn detect_ipv6(&self) -> Result<IpAddr> {
        // IPv6-specific services
        let ipv6_services = [
            "https://api6.ipify.org",
            "https://v6.ident.me",
            "https://ipv6.icanhazip.com",
        ];

        for service in ipv6_services {
            match self.try_service(service).await {
                Ok(ip) => {
                    if ip.is_ipv6() {
                        tracing::debug!("Detected IPv6 {} from {}", ip, service);
                        return Ok(ip);
                    }
                }
                Err(e) => {
                    tracing::warn!("IPv6 service {} failed: {}", service, e);
                }
            }
        }

        Err(DdnsError::IpDetection(
            "All IPv6 detection services failed".to_string(),
        ))
    }

    /// Try a single IP detection service.
    async fn try_service(&self, url: &str) -> Result<IpAddr> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(DdnsError::IpDetection(format!(
                "HTTP {} from {}",
                response.status(),
                url
            )));
        }

        let text = response.text().await?;
        let ip_str = text.trim();

        ip_str
            .parse()
            .map_err(|_| DdnsError::IpDetection(format!("Invalid IP response: {}", ip_str)))
    }
}

impl Default for IpDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_services() {
        let detector = IpDetector::new();
        assert!(!detector.services.is_empty());
    }

    #[test]
    fn test_custom_services() {
        let detector = IpDetector::with_services(vec!["https://example.com".to_string()]);
        assert_eq!(detector.services.len(), 1);
    }
}
