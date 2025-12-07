//! Configuration management for rusty-dns.

use crate::error::{DdnsError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Check interval in seconds (default: 300 = 5 minutes).
    #[serde(default = "default_interval")]
    pub check_interval_secs: u64,

    /// IP detection services to use.
    #[serde(default = "default_ip_services")]
    pub ip_services: Vec<String>,

    /// Configured DDNS providers.
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,

    /// History settings.
    #[serde(default)]
    pub history: HistoryConfig,
}

fn default_interval() -> u64 {
    300
}

fn default_ip_services() -> Vec<String> {
    vec![
        "https://api.ipify.org".to_string(),
        "https://icanhazip.com".to_string(),
        "https://ifconfig.me/ip".to_string(),
        "https://ipecho.net/plain".to_string(),
    ]
}

/// Provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    #[serde(rename = "cloudflare")]
    Cloudflare {
        /// API token (or environment variable name if prefixed with $).
        api_token: String,
        /// Zone ID.
        zone_id: String,
        /// DNS record name (e.g., "vpn.example.com").
        record_name: String,
        /// Whether to proxy through Cloudflare (default: false).
        #[serde(default)]
        proxied: bool,
    },

    #[serde(rename = "namecheap")]
    Namecheap {
        /// Domain name.
        domain: String,
        /// Host (subdomain, @ for root).
        host: String,
        /// Dynamic DNS password.
        password: String,
    },

    #[serde(rename = "duckdns")]
    DuckDns {
        /// DuckDNS subdomain(s), comma-separated.
        domains: String,
        /// DuckDNS token.
        token: String,
    },

    #[serde(rename = "godaddy")]
    GoDaddy {
        /// API key.
        api_key: String,
        /// API secret.
        api_secret: String,
        /// Domain name.
        domain: String,
        /// Record name (subdomain).
        name: String,
        /// TTL in seconds (default: 600).
        #[serde(default = "default_ttl")]
        ttl: u32,
    },
}

fn default_ttl() -> u32 {
    600
}

/// History configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Whether to keep update history.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum number of history entries to keep.
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
}

fn default_true() -> bool {
    true
}

fn default_max_entries() -> usize {
    100
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 100,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            check_interval_secs: 300,
            ip_services: default_ip_services(),
            providers: Vec::new(),
            history: HistoryConfig::default(),
        }
    }
}

impl Config {
    /// Get the default config file path.
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| DdnsError::Config("Could not find config directory".to_string()))?;

        Ok(config_dir.join("rusty-dns").join("config.toml"))
    }

    /// Load configuration from file.
    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;
        Self::load_from(&path)
    }

    /// Load configuration from a specific path.
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file.
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path()?;
        self.save_to(&path)
    }

    /// Save configuration to a specific path.
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Generate example configuration.
    pub fn example() -> Self {
        Self {
            check_interval_secs: 300,
            ip_services: default_ip_services(),
            providers: vec![
                ProviderConfig::Cloudflare {
                    api_token: "$CF_API_TOKEN".to_string(),
                    zone_id: "your-zone-id".to_string(),
                    record_name: "vpn.example.com".to_string(),
                    proxied: false,
                },
                ProviderConfig::DuckDns {
                    domains: "mysubdomain".to_string(),
                    token: "$DUCKDNS_TOKEN".to_string(),
                },
            ],
            history: HistoryConfig::default(),
        }
    }
}

impl ProviderConfig {
    /// Get the provider name.
    pub fn name(&self) -> &'static str {
        match self {
            ProviderConfig::Cloudflare { .. } => "cloudflare",
            ProviderConfig::Namecheap { .. } => "namecheap",
            ProviderConfig::DuckDns { .. } => "duckdns",
            ProviderConfig::GoDaddy { .. } => "godaddy",
        }
    }

    /// Get the display name (domain/subdomain).
    pub fn display_name(&self) -> String {
        match self {
            ProviderConfig::Cloudflare { record_name, .. } => record_name.clone(),
            ProviderConfig::Namecheap { domain, host, .. } => {
                if host == "@" {
                    domain.clone()
                } else {
                    format!("{}.{}", host, domain)
                }
            }
            ProviderConfig::DuckDns { domains, .. } => format!("{}.duckdns.org", domains),
            ProviderConfig::GoDaddy { domain, name, .. } => {
                if name == "@" {
                    domain.clone()
                } else {
                    format!("{}.{}", name, domain)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.check_interval_secs, 300);
        assert!(!config.ip_services.is_empty());
    }

    #[test]
    fn test_example_config() {
        let config = Config::example();
        assert_eq!(config.providers.len(), 2);
    }

    #[test]
    fn test_provider_names() {
        let cf = ProviderConfig::Cloudflare {
            api_token: "test".to_string(),
            zone_id: "test".to_string(),
            record_name: "vpn.example.com".to_string(),
            proxied: false,
        };
        assert_eq!(cf.name(), "cloudflare");
        assert_eq!(cf.display_name(), "vpn.example.com");
    }
}
