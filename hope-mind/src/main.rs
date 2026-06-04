//! Hope Mind — Local AI Integration via Ollama
//!
//! Provides a unified interface for local AI inference using Ollama.
//! Auto-selects the optimal model based on available hardware.

pub mod ollama;
pub mod models;
pub mod context;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("chat") => chat_mode().await,
        Some("status") => show_status().await,
        Some("models") => list_models().await,
        Some("select") => select_model().await,
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        Some("--version") | Some("-v") => {
            println!("hope-mind {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            info!("Hope Mind v{}", env!("CARGO_PKG_VERSION"));
            chat_mode().await
        }
    }
}

/// Interactive chat mode
async fn chat_mode() -> Result<()> {
    let client = ollama::OllamaClient::new()?;
    let model = models::select_optimal_model()?;

    info!("Starting chat with model: {}", model);

    println!("Hope Mind — Chat Mode");
    println!("Model: {}", model);
    println!("Type 'exit' to quit, 'clear' to reset context\n");

    loop {
        print!("You: ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" || input == "quit" {
            println!("Goodbye!");
            break;
        }

        if input == "clear" {
            println!("Context cleared.\n");
            continue;
        }

        if input.is_empty() {
            continue;
        }

        match client.chat(&model, input).await {
            Ok(response) => {
                println!("Hope: {}\n", response);
            }
            Err(e) => {
                eprintln!("Error: {}\n", e);
            }
        }
    }

    Ok(())
}

/// Show system status and model info
async fn show_status() -> Result<()> {
    let sys = context::SystemContext::gather()?;
    let model = models::select_optimal_model()?;

    println!("Hope Mind Status");
    println!("================");
    println!("Model: {}", model);
    println!("RAM: {}GB total, {}GB available", sys.ram_total_gb, sys.ram_available_gb);
    println!("CPU: {} cores", sys.cpu_cores);
    if let Some(gpu) = &sys.gpu_info {
        println!("GPU: {}", gpu);
    }
    println!("Ollama: {}", if ollama::OllamaClient::is_running() { "running" } else { "not running" });

    Ok(())
}

/// List available models
async fn list_models() -> Result<()> {
    let client = ollama::OllamaClient::new()?;

    println!("Available models:");
    println!("=================");

    match client.list_models().await {
        Ok(models) => {
            for model in &models {
                println!("  {} ({})", model.name, model.size);
            }
            if models.is_empty() {
                println!("  No models installed. Run: ollama pull smollm2:135m");
            }
        }
        Err(e) => {
            eprintln!("Error listing models: {}", e);
            println!("Make sure Ollama is running: ollama serve");
        }
    }

    Ok(())
}

/// Manually select a model
async fn select_model() -> Result<()> {
    let sys = context::SystemContext::gather()?;
    let all_models = models::all_models();

    println!("System: {}GB RAM, {} cores", sys.ram_total_gb, sys.cpu_cores);
    println!("\nAvailable models:");
    println!("=================");

    for (i, m) in all_models.iter().enumerate() {
        let marker = if models::is_recommended(m, &sys) { " *" } else { "" };
        println!("  {}. {} (RAM: {}){}", i + 1, m.name, m.ram_required, marker);
    }

    println!("\n* = recommended for your hardware");

    Ok(())
}

fn print_help() {
    println!(
        r#"hope-mind — Hope OS Local AI

Usage: hope-mind [COMMAND]

Commands:
    chat        Interactive chat mode (default)
    status      Show system status and current model
    models      List available models
    select      Show model selection for your hardware
    --help      Show this help
    --version   Show version

Environment:
    HOPE_MIND_MODEL     Force specific model
    HOPE_MIND_OLLAMA    Ollama server URL (default: http://localhost:11434)
    HOPE_MIND_CTX_SIZE  Context window size (default: 2048)
"#
    );
}
