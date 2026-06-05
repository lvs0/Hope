//! Notification system for Hope OS
//!
//! Simple notification system: 1 question max, 3 buttons max, no jargon.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// A notification action (button)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Label shown on the button
    pub label: String,
    /// Unique identifier for the action
    pub id: String,
}

/// A user notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Notification title (keep short)
    pub title: String,
    /// Notification body text (no jargon)
    pub body: String,
    /// Action buttons (max 3)
    pub actions: Vec<Action>,
}

impl Notification {
    /// Validate notification rules: 1 question, 3 buttons max, no jargon
    pub fn validate(&self) -> Result<(), NotificationError> {
        // Max 3 buttons
        if self.actions.len() > 3 {
            return Err(NotificationError::TooManyButtons(self.actions.len()));
        }

        // Check for jargon
        let jargon_words = ["kernel", "module", "driver", "udev", "irq", "dma", "ioctl"];
        let body_lower = self.body.to_lowercase();
        for word in jargon_words {
            if body_lower.contains(word) {
                return Err(NotificationError::Jargon(word.to_string()));
            }
        }

        Ok(())
    }

    /// Create a simple notification with one OK button
    pub fn simple(title: &str, body: &str) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            actions: vec![Action {
                label: "OK".into(),
                id: "ok".into(),
            }],
        }
    }

    /// Create a notification with a question and choices
    pub fn question(title: &str, body: &str, choices: Vec<(&str, &str)>) -> Self {
        assert!(choices.len() <= 3, "Max 3 choices allowed");

        Self {
            title: title.into(),
            body: body.into(),
            actions: choices
                .into_iter()
                .map(|(label, id)| Action {
                    label: label.into(),
                    id: id.into(),
                })
                .collect(),
        }
    }
}

/// Notification system errors
#[derive(Debug, Clone)]
pub enum NotificationError {
    TooManyButtons(usize),
    Jargon(String),
}

impl std::fmt::Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyButtons(n) => write!(f, "Too many buttons: {} (max 3)", n),
            Self::Jargon(word) => write!(f, "Jargon detected: {}", word),
        }
    }
}

impl std::error::Error for NotificationError {}

/// Notification sender channel type
pub type NotificationSender = Arc<tokio::sync::mpsc::Sender<Notification>>;

/// Spawn the notification system and return the sender
pub fn spawn_notification_system() -> NotificationSender {
    let (tx, mut rx) = mpsc::channel::<Notification>(32);

    tokio::spawn(async move {
        info!("Notification system started");

        while let Some(notification) = rx.recv().await {
            if let Err(e) = notification.validate() {
                error!("Invalid notification: {}", e);
                continue;
            }

            show_notification(&notification).await;
        }
    });

    Arc::new(tx)
}

/// Show a notification using the best available backend
async fn show_notification(notification: &Notification) {
    debug!("Showing notification: {}", notification.title);

    // Try notify-send first (most portable)
    if let Err(e) = show_via_notify_send(notification) {
        debug!("notify-send failed: {}", e);
        // Fall back to terminal echo
        show_via_terminal(notification);
    }
}

/// Show notification via notify-send (libnotify)
fn show_via_notify_send(notification: &Notification) -> Result<(), std::io::Error> {
    let mut args = vec!["-a".into(), "Hope HAL".into(), "-u".into(), "normal".into()];

    args.push(notification.title.clone());
    args.push(notification.body.clone());

    if !notification.actions.is_empty() {
        let actions_str: Vec<String> = notification
            .actions
            .iter()
            .flat_map(|a| vec![a.label.clone(), a.id.clone()])
            .collect();
        args.push("-A".into());
        args.push(actions_str.join(","));
    }

    std::process::Command::new("notify-send")
        .args(&args)
        .output()?;

    Ok(())
}

/// Fallback: show notification in terminal
fn show_via_terminal(notification: &Notification) {
    println!("┌──────────────────────────────────────┐");
    println!("│ {} │", centered(&notification.title, 38));
    println!("├──────────────────────────────────────┤");
    for line in notification.body.lines() {
        println!("│ {} │", padded(line, 38));
    }
    if !notification.actions.is_empty() {
        println!("├──────────────────────────────────────┤");
        let labels: Vec<&str> = notification.actions.iter().map(|a| a.label.as_str()).collect();
        println!("│ {} │", centered(&labels.join(" | "), 38));
    }
    println!("└──────────────────────────────────────┘");
}

fn centered(s: &str, width: usize) -> String {
    let s_len = s.chars().count();
    if s_len >= width {
        return s.chars().take(width).collect();
    }
    let pad = (width - s_len) / 2;
    format!("{}{}", " ".repeat(pad), s)
}

fn padded(s: &str, width: usize) -> String {
    let s_len = s.chars().count();
    if s_len >= width {
        return s.chars().take(width).collect();
    }
    format!("{}{}", s, " ".repeat(width - s_len))
}
