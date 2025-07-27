use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use schemars::_private::serde_json;
use toml::from_str;
use crate::types::HolgerConfig;

pub fn load_config_from_path(path: &Path) -> Result<HolgerConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            "json" => parse_json_config(&content),
            "toml" => parse_toml_config(&content),
            other => Err(anyhow::anyhow!("Unsupported config extension: {}", other)),
        }
    } else {
        Err(anyhow::anyhow!(
            "No extension found for config file: {}",
            path.display()
        ))
    }
}

fn parse_json_config(content: &str) -> Result<HolgerConfig> {
    let cfg = serde_json::from_str::<HolgerConfig>(content)
        .context("Failed to parse JSON Holger config")?;
    Ok(cfg)
}

fn parse_toml_config(content: &str) -> Result<HolgerConfig> {
    let cfg = from_str::<HolgerConfig>(content)
        .context("Failed to parse TOML Holger config")?;
    Ok(cfg)
}
