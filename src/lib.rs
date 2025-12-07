//! # rusty-dns
//!
//! A Dynamic DNS client written in Rust with MCP (Model Context Protocol) support.
//!
//! ## Features
//!
//! - Multiple DDNS providers: Cloudflare, Namecheap, DuckDNS, GoDaddy
//! - Automatic IP change detection
//! - MCP server for AI assistant integration (Claude Code, etc.)
//! - Daemon mode with configurable check interval
//! - SSH tunnel support for remote management
//!
//! ## Usage
//!
//! ```bash
//! # Check current IP
//! rusty-dns status
//!
//! # Force update all providers
//! rusty-dns update
//!
//! # Run as daemon
//! rusty-dns daemon
//!
//! # Start MCP server (for AI assistants)
//! rusty-dns mcp
//! ```

pub mod config;
pub mod detector;
pub mod error;
pub mod mcp;
pub mod providers;

pub use config::Config;
pub use detector::IpDetector;
pub use error::{DdnsError, Result};
