//! MCP (Model Context Protocol) server for AI assistant integration.

pub mod server;
pub mod tools;

pub use server::McpServer;
pub use tools::get_tools;
