# CLAUDE.md

AI assistant guide for rusty-dns development.

## Project Overview

**rusty-dns** is a Dynamic DNS (DDNS) client written in Rust with MCP (Model Context Protocol) support for AI assistant integration.

**Key Features:**
- Multi-provider DDNS support (Cloudflare, Namecheap, DuckDNS, GoDaddy)
- MCP server for AI-assisted remote configuration
- Daemon mode with configurable check intervals
- Environment variable resolution for secrets

---

## Architecture

```
rusty-dns/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry point (clap)
│   ├── lib.rs            # Library exports
│   ├── config.rs         # TOML configuration
│   ├── detector.rs       # IP detection service
│   ├── error.rs          # Error types
│   ├── mcp/
│   │   ├── mod.rs        # MCP module
│   │   ├── server.rs     # JSON-RPC 2.0 server (stdio)
│   │   └── tools.rs      # MCP tool definitions
│   └── providers/
│       ├── mod.rs        # DdnsProvider trait
│       ├── cloudflare.rs # Cloudflare API
│       ├── namecheap.rs  # Namecheap Dynamic DNS
│       ├── duckdns.rs    # DuckDNS API
│       └── godaddy.rs    # GoDaddy Domains API
├── systemd/              # Linux services
│   └── rusty-dns.service
└── launchd/              # macOS services
    ├── com.rusty-dns.daemon.plist
    └── com.rusty-dns.agent.plist
```

---

## Key Components

### DdnsProvider Trait
```rust
#[async_trait]
pub trait DdnsProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn domain(&self) -> String;
    async fn get_current_ip(&self) -> Result<Option<IpAddr>>;
    async fn update_ip(&self, ip: IpAddr) -> Result<UpdateResult>;
    async fn validate(&self) -> Result<()>;
}
```

### MCP Tools
| Tool | Description |
|------|-------------|
| `ddns_status` | Current IP, provider status, last update |
| `ddns_update` | Force DNS update (force=true to update even if unchanged) |
| `ddns_history` | Recent update history |
| `ddns_test_provider` | Test provider connectivity |
| `ddns_add_provider` | Instructions for adding provider |
| `ddns_remove_provider` | Instructions for removing provider |

---

## Configuration

Config file: `~/.config/rusty-dns/config.toml`

```toml
check_interval_secs = 300

[[providers]]
type = "cloudflare"
api_token = "$CF_API_TOKEN"  # Env var reference
zone_id = "your-zone-id"
record_name = "vpn.example.com"
proxied = false

[[providers]]
type = "duckdns"
domains = "mysubdomain"
token = "$DUCKDNS_TOKEN"
```

### Environment Variables
Secrets can use `$VAR_NAME` syntax for environment variable resolution.

---

## CLI Commands

```bash
# Show status
rusty-dns status

# Force update
rusty-dns update --force

# Run as daemon (5 minute checks)
rusty-dns daemon --interval 300

# Run MCP server
rusty-dns mcp

# Validate config
rusty-dns validate
```

---

## MCP Remote Configuration

### Via SSH Tunnel
```bash
# From Mac/PC with YubiKey
ssh -t pi@raspberrypi "rusty-dns mcp"
```

### Claude Code Configuration
```json
// ~/.claude/mcp_servers.json
{
  "rusty-dns": {
    "command": "ssh",
    "args": ["-t", "pi@raspberrypi", "rusty-dns", "mcp"]
  }
}
```

---

## Development

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
```

### Run
```bash
cargo run -- status
cargo run -- mcp
```

---

## Service Installation

### Linux (systemd)
```bash
sudo cp systemd/rusty-dns.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now rusty-dns
```

### macOS (launchd)
```bash
# System daemon (root)
sudo cp launchd/com.rusty-dns.daemon.plist /Library/LaunchDaemons/
sudo launchctl load -w /Library/LaunchDaemons/com.rusty-dns.daemon.plist

# User agent (current user)
cp launchd/com.rusty-dns.agent.plist ~/Library/LaunchAgents/
launchctl load -w ~/Library/LaunchAgents/com.rusty-dns.agent.plist
```

---

## Adding a New Provider

1. Create `src/providers/newprovider.rs`
2. Implement `DdnsProvider` trait
3. Add to `ProviderConfig` enum in `config.rs`
4. Add to `create_provider()` factory in `providers/mod.rs`
5. Export in `providers/mod.rs`

---

## Error Handling

Uses `thiserror` for error types:
- `DdnsError::Config` - Configuration errors
- `DdnsError::Network` - HTTP/network errors
- `DdnsError::Provider` - Provider-specific errors
- `DdnsError::IpDetection` - IP detection failures

---

## Git Commit Rules

**Never add AI attribution to commits.** Write clean, technical commit messages:

```
Short descriptive title

- Bullet point details
- More details if needed
```
