//! Hope Spotlight — Universal Search and Launcher
//!
//! Provides a Spotlight-like search interface for Hope OS.
//! Searches applications, files, settings, and commands.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub subtitle: String,
    pub category: SearchCategory,
    pub action: String,
    pub score: f32,
}

/// Category of search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchCategory {
    Application,
    File,
    Setting,
    Command,
    Calculation,
}

impl std::fmt::Display for SearchCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchCategory::Application => write!(f, "App"),
            SearchCategory::File => write!(f, "File"),
            SearchCategory::Setting => write!(f, "Setting"),
            SearchCategory::Command => write!(f, "Command"),
            SearchCategory::Calculation => write!(f, "Calc"),
        }
    }
}

/// A searchable system setting
#[derive(Debug, Clone)]
struct SettingEntry {
    name: String,
    description: String,
    category: String,
    command: String,
}

/// Search index
pub struct SearchIndex {
    applications: Vec<ApplicationEntry>,
    settings: Vec<SettingEntry>,
}

#[derive(Debug, Clone)]
struct ApplicationEntry {
    name: String,
    executable: String,
    description: String,
}

impl SearchIndex {
    /// Build search index from system
    pub fn build() -> Result<Self> {
        let applications = Self::index_applications()?;
        let settings = Self::index_settings()?;
        Ok(Self {
            applications,
            settings,
        })
    }

    /// Index installed applications
    fn index_applications() -> Result<Vec<ApplicationEntry>> {
        let mut apps = Vec::new();

        let desktop_dirs = [
            "/usr/share/applications",
            "/usr/local/share/applications",
            "/var/lib/flatpak/exports/share/applications",
        ];

        for dir in &desktop_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "desktop") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Some(app) = Self::parse_desktop_file(&content) {
                                apps.push(app);
                            }
                        }
                    }
                }
            }
        }

        Ok(apps)
    }

    /// Index system settings (GNOME, KDE, common configs)
    fn index_settings() -> Result<Vec<SettingEntry>> {
        let mut settings = Vec::new();

        // Network settings
        settings.push(SettingEntry {
            name: "WiFi".to_string(),
            description: "Configure wireless networks".to_string(),
            category: "Network".to_string(),
            command: "nmcli device wifi list".to_string(),
        });
        settings.push(SettingEntry {
            name: "Bluetooth".to_string(),
            description: "Manage Bluetooth devices".to_string(),
            category: "Network".to_string(),
            command: "bluetoothctl show".to_string(),
        });
        settings.push(SettingEntry {
            name: "VPN".to_string(),
            description: "VPN connections".to_string(),
            category: "Network".to_string(),
            command: "nmcli connection show".to_string(),
        });

        // Display settings
        settings.push(SettingEntry {
            name: "Display".to_string(),
            description: "Screen resolution and brightness".to_string(),
            category: "Display".to_string(),
            command: "wlr-randr".to_string(),
        });
        settings.push(SettingEntry {
            name: "Wallpaper".to_string(),
            description: "Change desktop wallpaper".to_string(),
            category: "Display".to_string(),
            command: "swaymsg output * bg ~/Pictures/wallpaper.png fill".to_string(),
        });

        // Sound settings
        settings.push(SettingEntry {
            name: "Audio".to_string(),
            description: "Sound settings and volume".to_string(),
            category: "Sound".to_string(),
            command: "pavucontrol".to_string(),
        });
        settings.push(SettingEntry {
            name: "Microphone".to_string(),
            description: "Input device settings".to_string(),
            category: "Sound".to_string(),
            command: "pactl list sources".to_string(),
        });

        // System settings
        settings.push(SettingEntry {
            name: "Power".to_string(),
            description: "Power management and battery".to_string(),
            category: "System".to_string(),
            command: "powerprofilesctl get".to_string(),
        });
        settings.push(SettingEntry {
            name: "Keyboard".to_string(),
            description: "Keyboard layout and shortcuts".to_string(),
            category: "System".to_string(),
            command: "localectl status".to_string(),
        });
        settings.push(SettingEntry {
            name: "Theme".to_string(),
            description: "Change system theme".to_string(),
            category: "Appearance".to_string(),
            command: "gsettings get org.gnome.desktop.interface gtk-theme".to_string(),
        });
        settings.push(SettingEntry {
            name: "Font".to_string(),
            description: "System font settings".to_string(),
            category: "Appearance".to_string(),
            command: "gsettings get org.gnome.desktop.interface font-name".to_string(),
        });

        // User settings
        settings.push(SettingEntry {
            name: "Users".to_string(),
            description: "Manage user accounts".to_string(),
            category: "System".to_string(),
            command: "cat /etc/passwd".to_string(),
        });
        settings.push(SettingEntry {
            name: "Firewall".to_string(),
            description: "Network security rules".to_string(),
            category: "Security".to_string(),
            command: "ufw status".to_string(),
        });

        // Hope OS specific
        settings.push(SettingEntry {
            name: "HAL".to_string(),
            description: "Hardware Adaptation Layer status".to_string(),
            category: "Hope OS".to_string(),
            command: "systemctl status hope-hal".to_string(),
        });
        settings.push(SettingEntry {
            name: "Shell".to_string(),
            description: "Hope Shell compositor settings".to_string(),
            category: "Hope OS".to_string(),
            command: "hope-shell config".to_string(),
        });

        Ok(settings)
    }

    /// Parse a .desktop file
    fn parse_desktop_file(content: &str) -> Option<ApplicationEntry> {
        let mut name = None;
        let mut exec = None;
        let mut comment = None;

        for line in content.lines() {
            if let Some(value) = line.strip_prefix("Name=") {
                name = Some(value.to_string());
            }
            if let Some(value) = line.strip_prefix("Exec=") {
                exec = Some(value.to_string());
            }
            if let Some(value) = line.strip_prefix("Comment=") {
                comment = Some(value.to_string());
            }
        }

        match (name, exec) {
            (Some(n), Some(e)) => Some(ApplicationEntry {
                name: n,
                executable: e,
                description: comment.unwrap_or_default(),
            }),
            _ => None,
        }
    }

    /// Search the index
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchResult> = Vec::new();

        // Search applications
        for app in &self.applications {
            let score = self.fuzzy_score(&app.name, &query_lower);
            if score > 0.3 {
                results.push(SearchResult {
                    title: app.name.clone(),
                    subtitle: app.description.clone(),
                    category: SearchCategory::Application,
                    action: app.executable.clone(),
                    score,
                });
            }
        }

        // Search settings
        for setting in &self.settings {
            let name_score = self.fuzzy_score(&setting.name, &query_lower);
            let cat_score = self.fuzzy_score(&setting.category, &query_lower);
            let score = name_score.max(cat_score);
            if score > 0.3 {
                results.push(SearchResult {
                    title: setting.name.clone(),
                    subtitle: format!("[{}] {}", setting.category, setting.description),
                    category: SearchCategory::Setting,
                    action: setting.command.clone(),
                    score,
                });
            }
        }

        // Search files in home directory (shallow)
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(&home);
            self.search_files(&home_path, &query_lower, &mut results, 0, 3);
        }

        // Check for calculations
        if self.is_calculation(query) {
            if let Some(result) = self.evaluate_calculation(query) {
                results.push(SearchResult {
                    title: result.clone(),
                    subtitle: format!("= {}", query),
                    category: SearchCategory::Calculation,
                    action: format!("echo {}", result),
                    score: 1.0,
                });
            }
        }

        // Check for shell commands
        if self.is_command(query) {
            results.push(SearchResult {
                title: query.to_string(),
                subtitle: "Run as shell command".to_string(),
                category: SearchCategory::Command,
                action: query.to_string(),
                score: 0.5,
            });
        }

        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    /// Recursively search files up to max depth
    fn search_files(
        &self,
        dir: &Path,
        query: &str,
        results: &mut Vec<SearchResult>,
        current_depth: usize,
        max_depth: usize,
    ) {
        if current_depth >= max_depth {
            return;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_lowercase();

            // Skip hidden files and system dirs
            if name.starts_with('.') || name == "node_modules" || name == "__pycache__" {
                continue;
            }

            if let Some(stem) = path.file_stem() {
                let stem_str = stem.to_string_lossy().to_lowercase();
                let score = self.fuzzy_score(&stem_str, query);
                if score > 0.4 {
                    let is_dir = path.is_dir();
                    results.push(SearchResult {
                        title: entry.file_name().to_string_lossy().to_string(),
                        subtitle: path.to_string_lossy().to_string(),
                        category: SearchCategory::File,
                        action: if is_dir {
                            format!("cd {}", path.display())
                        } else {
                            path.to_string_lossy().to_string()
                        },
                        score: score * 0.8, // Files score slightly lower than apps
                    });
                }
            }

            // Recurse into directories
            if path.is_dir() {
                self.search_files(&path, query, results, current_depth + 1, max_depth);
            }
        }
    }

    /// Simple fuzzy matching score
    fn fuzzy_score(&self, candidate: &str, query: &str) -> f32 {
        let candidate_lower = candidate.to_lowercase();

        if candidate_lower == query {
            return 1.0;
        }
        if candidate_lower.starts_with(query) {
            return 0.9;
        }
        if candidate_lower.contains(query) {
            return 0.7;
        }

        // Character matching
        let mut score = 0.0;
        let mut last_idx = 0;
        for ch in query.chars() {
            if let Some(pos) = candidate_lower[last_idx..].find(ch) {
                score += 1.0;
                last_idx += pos + 1;
            }
        }
        score / query.len() as f32
    }

    /// Check if input is a mathematical expression
    fn is_calculation(&self, input: &str) -> bool {
        input.chars().any(|c| c.is_ascii_digit())
            && input.chars().any(|c| "+-*/".contains(c))
    }

    /// Check if input looks like a shell command
    fn is_command(&self, input: &str) -> bool {
        let cmd = input.split_whitespace().next().unwrap_or("");
        // Check common command prefixes
        matches!(
            cmd,
            "ls" | "cat" | "echo" | "grep" | "find" | "cd" | "mkdir" | "rm"
            | "cp" | "mv" | "sudo" | "apt" | "dnf" | "pacman" | "systemctl"
            | "git" | "docker" | "curl" | "wget" | "ssh" | "ping"
        )
    }

    /// Evaluate a simple calculation
    fn evaluate_calculation(&self, expr: &str) -> Option<String> {
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() == 3 {
            if let (Ok(a), Ok(b)) = (
                parts[0].parse::<f64>(),
                parts[2].parse::<f64>(),
            ) {
                let result = match parts[1] {
                    "+" => a + b,
                    "-" => a - b,
                    "*" | "x" | "\u{00d7}" => a * b,
                    "/" | "\u{00f7}" => a / b,
                    _ => return None,
                };
                return Some(format!("{}", result));
            }
        }
        None
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    let query = args[1..].join(" ");

    if query.is_empty() || query == "--help" || query == "-h" {
        print_help();
        return Ok(());
    }

    if query == "--version" || query == "-v" {
        println!("hope-spotlight {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    info!("Searching: {}", query);

    let index = SearchIndex::build()?;
    let results = index.search(&query);

    if results.is_empty() {
        println!("No results for '{}'", query);
    } else {
        for result in &results {
            println!("[{}] {} — {}", result.category, result.title, result.subtitle);
        }
    }

    Ok(())
}

fn print_help() {
    println!(
        r#"hope-spotlight — Hope OS Universal Search

Usage: hope-spotlight <query>

Examples:
    hope-spotlight firefox       Search for Firefox
    hope-spotlight wifi          Search for WiFi settings
    hope-spotlight wallpaper     Search for wallpaper settings
    hope-spotlight 45 * 12       Calculate 45 * 12
    hope-spotlight ls            Run ls command

Search categories:
    Applications    Installed .desktop apps
    Settings        System and Hope OS settings
    Files           Home directory files (shallow)
    Commands        Common shell commands
    Calculations    Mathematical expressions

Keyboard shortcuts (when running as overlay):
    Super+Espace     Open Spotlight
    Escape           Close
    Enter            Launch selected
    Up/Down          Navigate results
"#
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_matching() {
        let index = SearchIndex {
            applications: vec![ApplicationEntry {
                name: "Firefox".to_string(),
                executable: "firefox".to_string(),
                description: "Web browser".to_string(),
            }],
            settings: vec![],
        };

        let results = index.search("firefox");
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Firefox");
    }

    #[test]
    fn calculation() {
        let index = SearchIndex {
            applications: vec![],
            settings: vec![],
        };
        assert!(index.is_calculation("45 * 12"));
        assert!(!index.is_calculation("hello"));
    }

    #[test]
    fn setting_search() {
        let index = SearchIndex {
            applications: vec![],
            settings: vec![SettingEntry {
                name: "WiFi".to_string(),
                description: "Configure wireless networks".to_string(),
                category: "Network".to_string(),
                command: "nmcli device wifi list".to_string(),
            }],
        };
        let results = index.search("wifi");
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "WiFi");
        assert!(matches!(results[0].category, SearchCategory::Setting));
    }

    #[test]
    fn command_detection() {
        let index = SearchIndex {
            applications: vec![],
            settings: vec![],
        };
        assert!(index.is_command("ls -la"));
        assert!(index.is_command("git status"));
        assert!(!index.is_command("firefox"));
    }
}
