//! Hope Voice — Voice Assistant and Speech Interface
//!
//! Provides voice input/output for Hope OS.
//! Uses Whisper.cpp for speech-to-text, local TTS for text-to-speech.

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
        Some("listen") => {
            info!("Listening...");
            println!("Voice input not yet implemented.");
            println!("Planned: Whisper.cpp integration for speech-to-text.");
            Ok(())
        }
        Some("speak") => {
            let default_text = "Hello from Hope Voice".to_string();
            let text = args.get(2).unwrap_or(&default_text);
            info!("Speaking: {}", text);
            println!("TTS not yet implemented.");
            println!("Planned: Local TTS engine (Piper or similar).");
            Ok(())
        }
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        Some("--version") | Some("-v") => {
            println!("hope-voice {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            print_help();
            Ok(())
        }
    }
}

fn print_help() {
    println!(
        r#"hope-voice — Hope OS Voice Interface

Usage: hope-voice [COMMAND] [ARGS]

Commands:
    listen          Start voice input (speech-to-text)
    speak <text>    Speak text (text-to-speech)
    --help          Show this help
    --version       Show version

Note: Requires Whisper.cpp for STT and Piper for TTS.
      Install: sudo apt install whisper.cpp piper
"#
    );
}
