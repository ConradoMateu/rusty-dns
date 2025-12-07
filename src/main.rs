//! rusty-dns - Dynamic DNS client with MCP support.

use clap::{Parser, Subcommand};
use rusty_dns::config::Config;
use rusty_dns::detector::IpDetector;
use rusty_dns::mcp::McpServer;
use rusty_dns::providers::create_provider;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "rusty-dns")]
#[command(about = "Dynamic DNS client with MCP support for AI assistants")]
#[command(version)]
struct Cli {
    /// Path to config file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current DDNS status
    Status,

    /// Force update DNS records
    Update {
        /// Update even if IP hasn't changed
        #[arg(short, long)]
        force: bool,
    },

    /// Run as daemon (background service)
    Daemon {
        /// Check interval in seconds
        #[arg(short, long, default_value = "300")]
        interval: u64,
    },

    /// Run MCP server over stdio
    Mcp,

    /// Validate configuration
    Validate,
}

fn get_config_path(cli_path: Option<PathBuf>) -> PathBuf {
    if let Some(path) = cli_path {
        return path;
    }

    // Default locations
    let candidates = [
        dirs::config_dir().map(|p| p.join("rusty-dns/config.toml")),
        Some(PathBuf::from("/etc/rusty-dns/config.toml")),
        Some(PathBuf::from("config.toml")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return candidate;
        }
    }

    // Return default even if it doesn't exist
    dirs::config_dir()
        .map(|p| p.join("rusty-dns/config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config_path = get_config_path(cli.config);

    match cli.command {
        Commands::Status => {
            let config = Config::load_from(&config_path)?;
            cmd_status(config).await?;
        }
        Commands::Update { force } => {
            let config = Config::load_from(&config_path)?;
            cmd_update(config, force).await?;
        }
        Commands::Daemon { interval } => {
            let config = Config::load_from(&config_path)?;
            cmd_daemon(config, interval).await?;
        }
        Commands::Mcp => {
            let config = Config::load_from(&config_path)?;
            cmd_mcp(config).await?;
        }
        Commands::Validate => {
            let config = Config::load_from(&config_path)?;
            cmd_validate(config).await?;
        }
    }

    Ok(())
}

async fn cmd_status(config: Config) -> anyhow::Result<()> {
    let detector = IpDetector::new();

    println!("rusty-dns Status");
    println!("================\n");

    // Detect current IP
    match detector.detect_ipv4().await {
        Ok(ip) => println!("Current Public IP: {}", ip),
        Err(e) => println!("Failed to detect IP: {}", e),
    }

    println!("\nProviders:");
    println!("---------");

    for provider_config in &config.providers {
        let provider = create_provider(provider_config);

        print!("  {} ({}): ", provider.name(), provider.domain());

        match provider.get_current_ip().await {
            Ok(Some(ip)) => println!("{}", ip),
            Ok(None) => println!("(no record)"),
            Err(e) => println!("error: {}", e),
        }
    }

    Ok(())
}

async fn cmd_update(config: Config, force: bool) -> anyhow::Result<()> {
    let detector = IpDetector::new();
    let current_ip = detector.detect_ipv4().await?;

    println!("Current IP: {}", current_ip);
    println!();

    for provider_config in &config.providers {
        let provider = create_provider(provider_config);

        print!("Updating {} ({})... ", provider.name(), provider.domain());

        // Check if update needed
        if !force {
            if let Ok(Some(existing)) = provider.get_current_ip().await {
                if existing == current_ip {
                    println!("skipped (IP unchanged)");
                    continue;
                }
            }
        }

        match provider.update_ip(current_ip).await {
            Ok(result) => {
                if result.success {
                    if let Some(prev) = result.previous_ip {
                        println!("OK ({} -> {})", prev, current_ip);
                    } else {
                        println!("OK ({})", current_ip);
                    }
                } else {
                    println!("FAILED: {}", result.error.unwrap_or_default());
                }
            }
            Err(e) => println!("ERROR: {}", e),
        }
    }

    Ok(())
}

async fn cmd_daemon(config: Config, interval: u64) -> anyhow::Result<()> {
    let detector = IpDetector::new();
    let interval = Duration::from_secs(interval);

    println!(
        "Starting rusty-dns daemon (interval: {}s)",
        interval.as_secs()
    );

    let mut last_ip = None;

    loop {
        match detector.detect_ipv4().await {
            Ok(current_ip) => {
                let ip_changed = last_ip != Some(current_ip);

                if ip_changed {
                    println!(
                        "[{}] IP changed: {:?} -> {}",
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                        last_ip,
                        current_ip
                    );

                    for provider_config in &config.providers {
                        let provider = create_provider(provider_config);
                        match provider.update_ip(current_ip).await {
                            Ok(result) => {
                                if result.success {
                                    println!(
                                        "  {} ({}): updated",
                                        provider.name(),
                                        provider.domain()
                                    );
                                } else {
                                    eprintln!(
                                        "  {} ({}): failed - {}",
                                        provider.name(),
                                        provider.domain(),
                                        result.error.unwrap_or_default()
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "  {} ({}): error - {}",
                                    provider.name(),
                                    provider.domain(),
                                    e
                                );
                            }
                        }
                    }

                    last_ip = Some(current_ip);
                }
            }
            Err(e) => {
                eprintln!(
                    "[{}] Failed to detect IP: {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                    e
                );
            }
        }

        tokio::time::sleep(interval).await;
    }
}

async fn cmd_mcp(config: Config) -> anyhow::Result<()> {
    let server = McpServer::new(config);
    server.run().await?;
    Ok(())
}

async fn cmd_validate(config: Config) -> anyhow::Result<()> {
    println!("Validating configuration...\n");

    let mut all_valid = true;

    for provider_config in &config.providers {
        let provider = create_provider(provider_config);

        print!("  {} ({}): ", provider.name(), provider.domain());

        match provider.validate().await {
            Ok(()) => println!("OK"),
            Err(e) => {
                println!("FAILED - {}", e);
                all_valid = false;
            }
        }
    }

    println!();

    if all_valid {
        println!("All providers validated successfully.");
    } else {
        println!("Some providers failed validation.");
        std::process::exit(1);
    }

    Ok(())
}
