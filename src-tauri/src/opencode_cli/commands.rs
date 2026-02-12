//! Tauri commands for OpenCode CLI management

use crate::platform::silent_command;
use serde::{Deserialize, Serialize};

/// Status of the OpenCode CLI installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeCliStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

/// Check if OpenCode CLI is installed and get its status
#[tauri::command]
pub async fn check_opencode_installed() -> Result<OpenCodeCliStatus, String> {
    log::trace!("Checking OpenCode CLI installation...");

    // Use the `which` crate to find the binary (cross-platform)
    let path = match which::which("opencode") {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => {
            log::trace!("OpenCode CLI not found in PATH");
            return Ok(OpenCodeCliStatus {
                installed: false,
                version: None,
                path: None,
            });
        }
    };

    // Get version
    let version_output = silent_command("opencode")
        .arg("version")
        .output()
        .map_err(|e| format!("Failed to run 'opencode version': {e}"))?;

    let version = if version_output.status.success() {
        let version_str = String::from_utf8_lossy(&version_output.stdout)
            .trim()
            .to_string();
        if version_str.is_empty() {
            None
        } else {
            Some(version_str)
        }
    } else {
        // Try --version flag as fallback
        let alt_output = silent_command("opencode").arg("--version").output().ok();
        alt_output.and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            } else {
                None
            }
        })
    };

    log::trace!("OpenCode CLI found: path={path}, version={version:?}");

    Ok(OpenCodeCliStatus {
        installed: true,
        version,
        path: Some(path),
    })
}

/// Available OpenCode model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeModel {
    pub id: String,
    pub name: String,
    pub provider: String,
}

/// List available OpenCode models by running `opencode models`
/// and enriching with friendly names from `~/.config/opencode/opencode.json`
#[tauri::command]
pub async fn list_opencode_models() -> Result<Vec<OpenCodeModel>, String> {
    log::trace!("Listing OpenCode models...");

    // 1. Load friendly names from opencode.json config
    let config_names = load_opencode_config_names();

    // 2. Run `opencode models` (no --format json, that flag doesn't exist)
    let output = silent_command("opencode")
        .arg("models")
        .output()
        .map_err(|e| format!("Failed to run 'opencode models': {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("opencode models failed: {stderr}");
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 3. Parse line-by-line text output (format: "provider/model-id")
    let models: Vec<OpenCodeModel> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let id = line.trim().to_string();
            // Skip lines that look like help text (e.g. "opencode models [provider]")
            if id.starts_with("opencode ") || id.contains("  ") || !id.contains('/') {
                return None;
            }
            let parts: Vec<&str> = id.splitn(2, '/').collect();
            let (provider, model_key) = if parts.len() == 2 {
                (parts[0].trim().to_string(), parts[1].trim().to_string())
            } else {
                return None;
            };

            // Look up friendly name from config, fall back to humanized model key
            let name = config_names
                .get(&id)
                .cloned()
                .unwrap_or_else(|| humanize_model_name(&model_key));

            Some(OpenCodeModel { id, name, provider })
        })
        .collect();

    log::trace!("Found {} OpenCode models", models.len());
    Ok(models)
}

/// Load model display names from `~/.config/opencode/opencode.json`
fn load_opencode_config_names() -> std::collections::HashMap<String, String> {
    let mut names = std::collections::HashMap::new();

    let config_path = dirs::config_dir().map(|d| d.join("opencode").join("opencode.json"));

    let Some(path) = config_path else {
        return names;
    };

    let Ok(content) = std::fs::read_to_string(&path) else {
        log::trace!("Could not read opencode config at {path:?}");
        return names;
    };

    // Parse the JSON config to extract provider.*.models.*.name
    let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) else {
        return names;
    };

    if let Some(providers) = config.get("provider").and_then(|p| p.as_object()) {
        for (provider_id, provider_val) in providers {
            if let Some(models) = provider_val.get("models").and_then(|m| m.as_object()) {
                for (model_key, model_val) in models {
                    if let Some(display_name) = model_val.get("name").and_then(|n| n.as_str()) {
                        let full_id = format!("{provider_id}/{model_key}");
                        names.insert(full_id, display_name.to_string());
                    }
                }
            }
        }
    }

    log::trace!("Loaded {} model names from opencode config", names.len());
    names
}

/// Convert a model key like "antigravity-claude-opus-4-6-thinking" into
/// a readable name like "Antigravity Claude Opus 4 6 Thinking"
fn humanize_model_name(key: &str) -> String {
    key.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + chars.as_str()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opencode_cli_status_serialization() {
        let status = OpenCodeCliStatus {
            installed: true,
            version: Some("0.2.5".to_string()),
            path: Some("/usr/local/bin/opencode".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"installed\":true"));
        assert!(json.contains("\"version\":\"0.2.5\""));
    }

    #[test]
    fn test_opencode_model_serialization() {
        let model = OpenCodeModel {
            id: "anthropic/claude-sonnet-4".to_string(),
            name: "Claude Sonnet 4".to_string(),
            provider: "anthropic".to_string(),
        };

        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("\"id\":\"anthropic/claude-sonnet-4\""));
    }
}
