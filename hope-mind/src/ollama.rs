//! Ollama API client for Hope Mind

use anyhow::{bail, Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Ollama API client
pub struct OllamaClient {
    client: Client,
    base_url: String,
}

/// Chat message
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Model info from Ollama
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: String,
}

/// Chat request body
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

/// Chat response from Ollama
#[derive(Deserialize)]
struct ChatResponse {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

/// List models response
#[derive(Deserialize)]
struct ListResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
    size: u64,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new() -> Result<Self> {
        let base_url = std::env::var("HOPE_MIND_OLLAMA")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        Ok(Self { client, base_url })
    }

    /// Check if Ollama is running (blocking)
    pub fn is_running() -> bool {
        let url = std::env::var("HOPE_MIND_OLLAMA")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let client = Client::new();
        match tokio::runtime::Runtime::new() {
            Ok(rt) => rt.block_on(async {
                client
                    .get(format!("{}/api/tags", url))
                    .timeout(std::time::Duration::from_secs(2))
                    .send()
                    .await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false)
            }),
            Err(_) => false,
        }
    }

    /// Send a chat message and get response
    pub async fn chat(&self, model: &str, message: &str) -> Result<String> {
        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            stream: false,
        };

        debug!("Sending chat to {} with model {}", self.base_url, model);

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("Ollama error {}: {}", status, body);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(chat_response.message.content)
    }

    /// Stream a chat response
    pub async fn chat_stream(
        &self,
        model: &str,
        message: &str,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("Ollama error {}: {}", status, body);
        }

        let byte_stream = response.bytes_stream();
        let stream = byte_stream.filter_map(|chunk| async move {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes).to_string();
                    for line in text.lines() {
                        if let Ok(parsed) = serde_json::from_str::<ChatResponse>(line) {
                            return Some(Ok(parsed.message.content));
                        }
                    }
                    None
                }
                Err(e) => Some(Err(anyhow::anyhow!("Stream error: {}", e))),
            }
        });

        Ok(stream)
    }

    /// List installed models
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !response.status().is_success() {
            bail!("Failed to list models");
        }

        let list: ListResponse = response.json().await?;

        Ok(list
            .models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.name,
                size: format_size(m.size),
            })
            .collect())
    }
}

/// Format bytes to human-readable size
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_test() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }
}
