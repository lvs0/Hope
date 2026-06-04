//! Model selection based on hardware capabilities

use crate::context::SystemContext;

/// Model tier definition
#[derive(Debug, Clone)]
pub struct ModelTier {
    /// Display name
    pub name: &'static str,
    /// Ollama model identifier
    pub model_id: &'static str,
    /// Minimum RAM in GB
    pub ram_required: &'static str,
    /// RAM threshold in GB for auto-selection
    pub ram_threshold_gb: f64,
    /// Description
    pub description: &'static str,
}

/// All available model tiers (from spec)
pub fn all_models() -> Vec<ModelTier> {
    vec![
        ModelTier {
            name: "hope-nano",
            model_id: "smollm2:135m",
            ram_required: "< 4GB",
            ram_threshold_gb: 0.0,
            description: "SmolLM2-360M — runs on anything, 128MB RAM",
        },
        ModelTier {
            name: "hope-small",
            model_id: "phi4-mini",
            ram_required: "4GB+",
            ram_threshold_gb: 4.0,
            description: "Phi-4-mini-Q4 — fast, 800MB RAM",
        },
        ModelTier {
            name: "hope-medium",
            model_id: "lfm-2.5:1.6b",
            ram_required: "8GB+",
            ram_threshold_gb: 8.0,
            description: "LFM-2.5 1.6B — balanced, 1.5GB RAM",
        },
        ModelTier {
            name: "hope-large",
            model_id: "qwen3:7b-q4",
            ram_required: "16GB+",
            ram_threshold_gb: 16.0,
            description: "Qwen3-7B-Q4 — powerful, 4GB RAM",
        },
        ModelTier {
            name: "hope-xl",
            model_id: "llama3.1:8b",
            ram_required: "32GB+",
            ram_threshold_gb: 32.0,
            description: "Llama3.1-8B — maximum quality, 6GB RAM",
        },
    ]
}

/// Select the optimal model based on available RAM
pub fn select_optimal_model() -> Result<String, anyhow::Error> {
    // Check for forced model
    if let Ok(forced) = std::env::var("HOPE_MIND_MODEL") {
        return Ok(forced);
    }

    let sys = SystemContext::gather()?;
    let models = all_models();

    // Select best model that fits in available RAM
    // Leave 1GB headroom for system
    let usable_ram = sys.ram_available_gb - 1.0;

    let selected = models
        .iter()
        .rev() // Start from largest
        .find(|m| usable_ram >= m.ram_threshold_gb)
        .unwrap_or(&models[0]); // Fallback to nano

    Ok(selected.model_id.to_string())
}

/// Check if a model is recommended for the given hardware
pub fn is_recommended(model: &ModelTier, sys: &SystemContext) -> bool {
    let usable_ram = sys.ram_available_gb - 1.0;
    usable_ram >= model.ram_threshold_gb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_tiers_exist() {
        let models = all_models();
        assert_eq!(models.len(), 5);
        assert_eq!(models[0].name, "hope-nano");
        assert_eq!(models[4].name, "hope-xl");
    }

    #[test]
    fn nano_always_recommended() {
        let sys = SystemContext {
            ram_total_gb: 2.0,
            ram_available_gb: 1.5,
            cpu_cores: 2,
            gpu_info: None,
        };
        let nano = &all_models()[0];
        assert!(is_recommended(nano, &sys));
    }
}
