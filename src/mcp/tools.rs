//! MCP tool definitions.

use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Get all available MCP tools.
pub fn get_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "ddns_status".to_string(),
            description: "Get current DDNS status including detected public IP, provider status, and last update time.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDefinition {
            name: "ddns_update".to_string(),
            description: "Force update DNS records for all configured providers. Use force=true to update even if IP hasn't changed.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "force": {
                        "type": "boolean",
                        "description": "Force update even if IP hasn't changed",
                        "default": false
                    }
                },
                "required": []
            }),
        },
        ToolDefinition {
            name: "ddns_history".to_string(),
            description: "Get history of recent DNS updates.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of history entries to return",
                        "default": 10
                    }
                },
                "required": []
            }),
        },
        ToolDefinition {
            name: "ddns_test_provider".to_string(),
            description: "Test connectivity and credentials for a specific DDNS provider.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "provider": {
                        "type": "string",
                        "description": "Provider name (cloudflare, namecheap, duckdns, godaddy)",
                        "enum": ["cloudflare", "namecheap", "duckdns", "godaddy"]
                    }
                },
                "required": ["provider"]
            }),
        },
        ToolDefinition {
            name: "ddns_add_provider".to_string(),
            description: "Get instructions for adding a new DDNS provider to the configuration.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "provider": {
                        "type": "string",
                        "description": "Provider type to add",
                        "enum": ["cloudflare", "namecheap", "duckdns", "godaddy"]
                    }
                },
                "required": ["provider"]
            }),
        },
        ToolDefinition {
            name: "ddns_remove_provider".to_string(),
            description: "Get instructions for removing a DDNS provider from the configuration.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "provider": {
                        "type": "string",
                        "description": "Provider name to remove"
                    }
                },
                "required": ["provider"]
            }),
        },
    ]
}
