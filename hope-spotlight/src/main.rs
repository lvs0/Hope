//! Hope Spotlight — Universal Search and Launcher
//!
//! Provides a Spotlight-like search interface for Hope OS.
//! Searches applications, files, settings, and commands.

use anyhow::Result;
use serde::{Deserialize, Serialize};
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

/// Search index
pub struct SearchIndex {
    applications: Vec<ApplicationEntry>,
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
        Ok(Self { applications })
    }

    /// Index installed applications
    fn index_applications() -> Result<Vec<ApplicationEntry>> {
        let mut apps = Vec::new();

        // Check common application directories
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

        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    /// Simple fuzzy matching score
    fn fuzzy_score(&self, candidate: &str, query: &str) -> f32 {
        let candidate_lower = candidate.to_lowercase();

        // Exact match
        if candidate_lower == query {
            return 1.0;
        }

        // Starts with
        if candidate_lower.starts_with(query) {
            return 0.9;
        }

        // Contains
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

    /// Evaluate a simple calculation
    fn evaluate_calculation(&self, expr: &str) -> Option<String> {
        // Simple eval for basic operations
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() == 3 {
            if let (Ok(a), Ok(b)) = (
                parts[0].parse::<f64>(),
                parts[2].parse::<f64>(),
            ) {
                let result = match parts[1] {
                    "+" => a + b,
                    "-" => a - b,
                    "*" | "x" | "×" => a * b,
                    "/" | "÷" => a / b,
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
    hope-spotlight 45 * 12       Calculate 45 * 12
    hope-spotlight wifi          Search for WiFi settings

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
        };

        let results = index.search("firefox");
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Firefox");
    }

    #[test]
    fn calculation() {
        let index = SearchIndex {
            applications: vec![],
        };

        assert!(index.is_calculation("45 * 12"));
        assert!(!index.is_calculation("hello"));
    }
}
