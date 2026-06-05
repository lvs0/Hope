//! Hope Voice — Voice Assistant and Speech Interface
//!
//! Provides voice input/output for Hope OS.
//! Uses external backends: Whisper.cpp for STT, Piper for TTS.
//! Falls back to espeak-ng if primary backends are unavailable.

use anyhow::{bail, Context, Result};
use std::process::Command;
use tracing::{info, warn};

/// Available TTS backends
#[derive(Debug, Clone)]
enum TtsBackend {
    Piper,
    EspeakNg,
    None,
}

/// Available STT backends
#[derive(Debug, Clone)]
enum SttBackend {
    WhisperCpp,
    None,
}

/// Detect available STT backend
fn detect_stt() -> SttBackend {
    if Command::new("whisper-cli")
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        info!("STT backend: whisper-cli");
        return SttBackend::WhisperCpp;
    }
    if Command::new("whisper.cpp")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        info!("STT backend: whisper.cpp");
        return SttBackend::WhisperCpp;
    }
    warn!("No STT backend found (install whisper.cpp)");
    SttBackend::None
}

/// Detect available TTS backend
fn detect_tts() -> TtsBackend {
    if Command::new("piper")
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        info!("TTS backend: piper");
        return TtsBackend::Piper;
    }
    if Command::new("espeak-ng")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        info!("TTS backend: espeak-ng (fallback)");
        return TtsBackend::EspeakNg;
    }
    warn!("No TTS backend found (install piper or espeak-ng)");
    TtsBackend::None
}

/// Record audio from microphone and save to file
fn record_audio(output_path: &str, duration_secs: u32) -> Result<()> {
    // Try arecord (ALSA) first, then parecord (PulseAudio)
    if Command::new("arecord")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        info!("Recording {}s via arecord...", duration_secs);
        let status = Command::new("arecord")
            .args([
                "-f", "S16_LE",
                "-r", "16000",
                "-c", "1",
                "-d", &duration_secs.to_string(),
                output_path,
            ])
            .status()
            .context("Failed to run arecord")?;

        if !status.success() {
            bail!("arecord failed with exit code: {:?}", status.code());
        }
        return Ok(());
    }

    if Command::new("parecord")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        info!("Recording {}s via parecord...", duration_secs);
        let status = Command::new("parecord")
            .args([
                "--file-format=wav",
                "--rate=16000",
                "--channels=1",
                output_path,
            ])
            .status()
            .context("Failed to run parecord")?;

        if !status.success() {
            bail!("parecord failed with exit code: {:?}", status.code());
        }
        return Ok(());
    }

    bail!("No recording tool found (install alsa-utils or pulseaudio-utils)")
}

/// Transcribe audio file to text using available STT backend
fn transcribe(audio_path: &str) -> Result<String> {
    match detect_stt() {
        SttBackend::WhisperCpp => {
            let output = Command::new("whisper-cli")
                .args([
                    "--model", "base",
                    "--language", "en",
                    "--output-format", "txt",
                    audio_path,
                ])
                .output()
                .context("Failed to run whisper-cli")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("whisper-cli failed: {}", stderr);
            }

            let text = String::from_utf8(output.stdout)
                .context("Whisper output is not valid UTF-8")?;
            Ok(text.trim().to_string())
        }
        SttBackend::None => {
            bail!("No STT backend available. Install whisper.cpp: https://github.com/ggerganov/whisper.cpp")
        }
    }
}

/// Synthesize text to speech and play it
fn speak(text: &str) -> Result<()> {
    match detect_tts() {
        TtsBackend::Piper => {
            info!("Speaking via piper: {}", text);
            let status = Command::new("piper")
                .args(["--text", text, "--output_file", "/tmp/hope-voice-output.wav"])
                .status()
                .context("Failed to run piper")?;

            if !status.success() {
                bail!("piper failed");
            }

            // Play the generated audio
            play_audio("/tmp/hope-voice-output.wav")
        }
        TtsBackend::EspeakNg => {
            info!("Speaking via espeak-ng: {}", text);
            let status = Command::new("espeak-ng")
                .arg(text)
                .status()
                .context("Failed to run espeak-ng")?;

            if !status.success() {
                bail!("espeak-ng failed");
            }
            Ok(())
        }
        TtsBackend::None => {
            println!("{}", text);
            Ok(())
        }
    }
}

/// Play an audio file
fn play_audio(path: &str) -> Result<()> {
    // Try paplay, then aplay, then mpv
    for player in &["paplay", "aplay", "mpv"] {
        let mut cmd = Command::new(player);
        if *player == "mpv" {
            cmd.args(["--no-video", path]);
        } else {
            cmd.arg(path);
        }
        if cmd
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
        {
            let status = cmd.status().context(format!("Failed to run {}", player))?;
            if status.success() {
                return Ok(());
            }
        }
    }
    bail!("No audio player found (install pulseaudio-utils, alsa-utils, or mpv)")
}

/// Interactive listen loop — record, transcribe, return text
fn listen_loop() -> Result<String> {
    let stt = detect_stt();
    if matches!(stt, SttBackend::None) {
        bail!("No STT backend available. Install whisper.cpp for speech recognition.");
    }

    let tmp_path = "/tmp/hope-voice-input.wav";
    println!("Listening... (speak now, recording for 5 seconds)");
    record_audio(tmp_path, 5)?;
    println!("Transcribing...");
    let text = transcribe(tmp_path)?;
    Ok(text)
}

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
            info!("Starting voice input...");
            match listen_loop() {
                Ok(text) => {
                    println!("You said: {}", text);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            Ok(())
        }
        Some("speak") => {
            let default_text = "Hello from Hope Voice".to_string();
            let text = args.get(2).unwrap_or(&default_text);
            info!("Speaking: {}", text);
            speak(text)
        }
        Some("status") => {
            println!("Hope Voice — Audio Backend Status");
            println!("  STT: {:?}", detect_stt());
            println!("  TTS: {:?}", detect_tts());
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
    listen              Record audio and transcribe (STT)
    speak <text>        Speak text aloud (TTS)
    status              Show available audio backends
    --help              Show this help
    --version           Show version

Backends:
    STT: whisper.cpp (primary), whisper-cli
    TTS: piper (primary), espeak-ng (fallback)
    Recording: arecord (ALSA), parecord (PulseAudio)
    Playback: paplay, aplay, mpv

Install backends:
    Whisper.cpp: https://github.com/ggerganov/whisper.cpp
    Piper: https://github.com/rhasspy/piper
    espeak-ng: sudo apt install espeak-ng
"#
    );
}
