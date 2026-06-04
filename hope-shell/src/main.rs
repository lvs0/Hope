//! Hope Shell — Wayland Compositor Wrapper
//!
//! Provides a unified interface for launching and configuring
//! Wayland compositors with the Hope OS Deep Space theme.

pub mod config;
pub mod compositor;
pub mod theme;

use anyhow::Result;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("start") => start_shell(),
        Some("config") => show_config(),
        Some("theme") => apply_theme(),
        Some("--help") | Some("-h") => { print_help(); Ok(()) }
        Some("--version") | Some("-v") => {
            println!("hope-shell {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            info!("Hope Shell v{}", env!("CARGO_PKG_VERSION"));
            start_shell()
        }
    }
}

/// Start the Hope Shell compositor
fn start_shell() -> Result<()> {
    info!("Starting Hope Shell...");

    // Load configuration
    let cfg = config::HopeShellConfig::load()?;
    info!("Configuration loaded: compositor={}", cfg.compositor.backend);

    // Detect available compositor
    let backend = compositor::detect_backend(&cfg.compositor)?;
    info!("Using backend: {:?}", backend);

    // Apply Deep Space theme
    theme::apply_deep_space(&cfg.theme)?;
    info!("Deep Space theme applied");

    // Launch compositor
    compositor::launch(&backend, &cfg.compositor)?;

    Ok(())
}

/// Show current configuration
fn show_config() -> Result<()> {
    let cfg = config::HopeShellConfig::load()?;
    println!("{}", toml::to_string_pretty(&cfg)?);
    Ok(())
}

/// Apply the Deep Space theme
fn apply_theme() -> Result<()> {
    let cfg = config::HopeShellConfig::load()?;
    theme::apply_deep_space(&cfg.theme)?;
    println!("Deep Space theme applied");
    Ok(())
}

fn print_help() {
    println!(
        r#"hope-shell — Hope OS Wayland Compositor

Usage: hope-shell [COMMAND]

Commands:
    start       Start the Hope Shell compositor (default)
    config      Show current configuration
    theme       Apply Deep Space theme
    --help      Show this help
    --version   Show version

Environment:
    HOPE_SHELL_CONFIG     Path to config file (default: ~/.config/hope-shell/config.toml)
    HOPE_SHELL_BACKEND    Force compositor backend (river, cage, sway, auto)
    HOPE_SHELL_THEME      Theme to apply (deep-space, light, auto)
"#
    );
}
