//! MCP JSON-RPC 2.0 server over stdio.

use crate::config::Config;
use crate::detector::IpDetector;
use crate::error::Result;
use crate::providers::{create_provider, UpdateResult};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

/// MCP Server for AI assistant integration.
pub struct McpServer {
    config: Config,
    detector: IpDetector,
    history: Arc<Mutex<Vec<UpdateResult>>>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(rename = "jsonrpc")]
    _jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    current_ip: Option<String>,
    providers: Vec<ProviderStatus>,
    last_update: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProviderStatus {
    name: String,
    domain: String,
    current_ip: Option<String>,
    healthy: bool,
}

impl McpServer {
    /// Create a new MCP server.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            detector: IpDetector::new(),
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Run the MCP server over stdio.
    pub async fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        eprintln!("rusty-dns MCP server started");

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    continue;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&error_response)?)?;
                    stdout.flush()?;
                    continue;
                }
            };

            let response = self.handle_request(request).await;
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }

        Ok(())
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize().await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request.params).await,
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    async fn handle_initialize(&self) -> std::result::Result<serde_json::Value, JsonRpcError> {
        Ok(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "rusty-dns",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    async fn handle_tools_list(&self) -> std::result::Result<serde_json::Value, JsonRpcError> {
        let tools = super::tools::get_tools();
        Ok(serde_json::json!({ "tools": tools }))
    }

    async fn handle_tools_call(
        &self,
        params: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing tool name".to_string(),
                data: None,
            })?;

        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        match name {
            "ddns_status" => self.tool_status().await,
            "ddns_update" => self.tool_update(arguments).await,
            "ddns_history" => self.tool_history(arguments).await,
            "ddns_test_provider" => self.tool_test_provider(arguments).await,
            "ddns_add_provider" => self.tool_add_provider(arguments).await,
            "ddns_remove_provider" => self.tool_remove_provider(arguments).await,
            _ => Err(JsonRpcError {
                code: -32602,
                message: format!("Unknown tool: {}", name),
                data: None,
            }),
        }
    }

    async fn tool_status(&self) -> std::result::Result<serde_json::Value, JsonRpcError> {
        let current_ip = self.detector.detect_ipv4().await.ok();

        let mut providers = Vec::new();
        for provider_config in &self.config.providers {
            let provider = create_provider(provider_config);

            let current = provider.get_current_ip().await.ok().flatten();
            let healthy = provider.validate().await.is_ok();

            providers.push(ProviderStatus {
                name: provider.name().to_string(),
                domain: provider.domain(),
                current_ip: current.map(|ip| ip.to_string()),
                healthy,
            });
        }

        let history = self.history.lock().await;
        let last_update = history.last().map(|r| r.timestamp.to_rfc3339());

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&StatusResponse {
                    current_ip: current_ip.map(|ip| ip.to_string()),
                    providers,
                    last_update,
                }).unwrap()
            }]
        }))
    }

    async fn tool_update(
        &self,
        arguments: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        let force = arguments
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let current_ip = self
            .detector
            .detect_ipv4()
            .await
            .map_err(|e| JsonRpcError {
                code: -32000,
                message: format!("Failed to detect IP: {}", e),
                data: None,
            })?;

        let mut results = Vec::new();
        for provider_config in &self.config.providers {
            let provider = create_provider(provider_config);

            // Check if update is needed
            if !force {
                if let Ok(Some(existing)) = provider.get_current_ip().await {
                    if existing == current_ip {
                        results.push(serde_json::json!({
                            "provider": provider.name(),
                            "domain": provider.domain(),
                            "skipped": true,
                            "reason": "IP unchanged"
                        }));
                        continue;
                    }
                }
            }

            let result = provider
                .update_ip(current_ip)
                .await
                .map_err(|e| JsonRpcError {
                    code: -32000,
                    message: e.to_string(),
                    data: None,
                })?;

            // Store in history
            self.history.lock().await.push(result.clone());

            results.push(serde_json::json!({
                "provider": result.provider,
                "domain": result.domain,
                "success": result.success,
                "ip": result.ip.map(|ip| ip.to_string()),
                "previous_ip": result.previous_ip.map(|ip| ip.to_string()),
                "error": result.error
            }));
        }

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&results).unwrap()
            }]
        }))
    }

    async fn tool_history(
        &self,
        arguments: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let history = self.history.lock().await;
        let recent: Vec<_> = history.iter().rev().take(limit).collect();

        let entries: Vec<_> = recent
            .iter()
            .map(|r| {
                serde_json::json!({
                    "provider": r.provider,
                    "domain": r.domain,
                    "success": r.success,
                    "ip": r.ip.map(|ip| ip.to_string()),
                    "previous_ip": r.previous_ip.map(|ip| ip.to_string()),
                    "error": r.error,
                    "timestamp": r.timestamp.to_rfc3339()
                })
            })
            .collect();

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&entries).unwrap()
            }]
        }))
    }

    async fn tool_test_provider(
        &self,
        arguments: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        let provider_name = arguments
            .get("provider")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing provider name".to_string(),
                data: None,
            })?;

        let provider_config = self
            .config
            .providers
            .iter()
            .find(|p| {
                let name = match p {
                    crate::config::ProviderConfig::Cloudflare { .. } => "cloudflare",
                    crate::config::ProviderConfig::Namecheap { .. } => "namecheap",
                    crate::config::ProviderConfig::DuckDns { .. } => "duckdns",
                    crate::config::ProviderConfig::GoDaddy { .. } => "godaddy",
                };
                name == provider_name
            })
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: format!("Provider not configured: {}", provider_name),
                data: None,
            })?;

        let provider = create_provider(provider_config);

        let validation = provider.validate().await;
        let current_ip = provider.get_current_ip().await.ok().flatten();

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&serde_json::json!({
                    "provider": provider.name(),
                    "domain": provider.domain(),
                    "valid": validation.is_ok(),
                    "error": validation.err().map(|e| e.to_string()),
                    "current_ip": current_ip.map(|ip| ip.to_string())
                })).unwrap()
            }]
        }))
    }

    async fn tool_add_provider(
        &self,
        _arguments: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        // This would require modifying the config file
        // For now, return instructions
        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": "To add a provider, edit the config file at ~/.config/rusty-dns/config.toml\n\nExample:\n\n[[providers]]\ntype = \"cloudflare\"\napi_token = \"your-token\"\nzone_id = \"your-zone-id\"\nrecord_name = \"home.example.com\"\nproxied = false"
            }]
        }))
    }

    async fn tool_remove_provider(
        &self,
        _arguments: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        // This would require modifying the config file
        // For now, return instructions
        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": "To remove a provider, edit the config file at ~/.config/rusty-dns/config.toml and remove the [[providers]] section for that provider."
            }]
        }))
    }
}
