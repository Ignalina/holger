use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use crate::types::HolgerConfig;

use serde::Deserialize;
pub fn factory(config:HolgerConfig)  {

}

pub fn load_config_from_path<P: AsRef<Path>>(path: P) -> Result<HolgerConfig> {
    let data = fs::read_to_string(path)?;
    let config: HolgerConfig = toml::from_str(&data)?;
    Ok(config)
}



