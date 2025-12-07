use anyhow::Result;
use serde::Deserialize;
use std::fs::read_to_string;

#[derive(Deserialize, Debug)]
struct DidConfig { pub controller_did: String }

pub fn load_did(config_path: Option<&str>) -> Result<String> {
    let cfg_path = config_path.unwrap_or("config/did.json");
    let content = read_to_string(cfg_path)?;
    let cfg: DidConfig = serde_json::from_str(&content)?;
    Ok(cfg.controller_did)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_did_default() {
        // if config missing, function should return an error; test not covering file I/O
    }
}
