//! Configuration management for Hope Shell

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Root configuration for Hope Shell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopeShellConfig {
    /// Compositor backend settings
    pub compositor: CompositorConfig,
    /// Theme settings
    pub theme: ThemeConfig,
    /// Panel settings
    pub panel: PanelConfig,
    /// Keyboard shortcuts
    pub keybindings: KeybindingConfig,
}

/// Compositor backend selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositorConfig {
    /// Backend to use: "river", "cage", "sway", or "auto"
    pub backend: String,
    /// Enable XWayland compatibility
    pub xwayland: bool,
    /// Enable direct scanout for gaming
    pub direct_scanout: bool,
    /// Enable Variable Refresh Rate
    pub vrr: bool,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme name: "deep-space", "light"
    pub name: String,
    /// Background color (hex)
    pub background: String,
    /// Foreground color (hex)
    pub foreground: String,
    /// Accent colors
    pub accents: AccentColors,
    /// Blur intensity (0 = off, 1-10 = blur level)
    pub blur: u8,
    /// Window opacity (0.0 - 1.0)
    pub opacity: f32,
}

/// Accent colors for the Deep Space theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccentColors {
    /// Primary accent (indigo)
    pub primary: String,
    /// Secondary accent (violet)
    pub secondary: String,
    /// Tertiary accent (cyan)
    pub tertiary: String,
    /// Error/danger color
    pub error: String,
    /// Success color
    pub success: String,
}

/// Panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    /// Position: "bottom", "top", "left", "right"
    pub position: String,
    /// Height in pixels
    pub height: u32,
    /// Auto-hide
    pub autohide: bool,
    /// Show clock
    pub clock: bool,
    /// Show systray
    pub systray: bool,
}

/// Keybinding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    /// Launcher key (default: Super)
    pub launcher: String,
    /// Terminal key
    pub terminal: String,
    /// HopeMind key
    pub hopemind: String,
}

impl Default for HopeShellConfig {
    fn default() -> Self {
        Self {
            compositor: CompositorConfig {
                backend: "auto".to_string(),
                xwayland: true,
                direct_scanout: true,
                vrr: true,
            },
            theme: ThemeConfig {
                name: "deep-space".to_string(),
                background: "#0F0F12".to_string(),
                foreground: "#E0E0E8".to_string(),
                accents: AccentColors {
                    primary: "#6366F1".to_string(),    // Indigo
                    secondary: "#8B5CF6".to_string(),  // Violet
                    tertiary: "#06B6D4".to_string(),   // Cyan
                    error: "#EF4444".to_string(),
                    success: "#22C55E".to_string(),
                },
                blur: 5,
                opacity: 0.95,
            },
            panel: PanelConfig {
                position: "bottom".to_string(),
                height: 32,
                autohide: false,
                clock: true,
                systray: true,
            },
            keybindings: KeybindingConfig {
                launcher: "Super".to_string(),
                terminal: "Super+Return".to_string(),
                hopemind: "Super+H".to_string(),
            },
        }
    }
}

impl HopeShellConfig {
    /// Load configuration from file, falling back to defaults
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read {}", config_path.display()))?;
            let config: Self = toml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", config_path.display()))?;
            Ok(config)
        } else {
            // Create default config
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Get the configuration file path
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?;
        Ok(config_dir.join("hope-shell").join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = HopeShellConfig::default();
        assert_eq!(cfg.compositor.backend, "auto");
        assert!(cfg.compositor.xwayland);
        assert_eq!(cfg.theme.name, "deep-space");
        assert_eq!(cfg.theme.background, "#0F0F12");
    }

    #[test]
    fn serialize_roundtrip() {
        let cfg = HopeShellConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let parsed: HopeShellConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.theme.name, cfg.theme.name);
        assert_eq!(parsed.panel.height, cfg.panel.height);
    }
}
