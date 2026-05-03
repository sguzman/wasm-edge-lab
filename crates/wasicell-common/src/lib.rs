use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    pub service: HashMap<String, ServiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub module: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub mounts: HashMap<String, String>,
    #[serde(default)]
    pub network: Option<NetworkConfig>,
    #[serde(default)]
    pub limits: Option<LimitsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    pub memory_mb: Option<u32>,
    pub max_runtime_seconds: Option<u32>,
}

impl AppManifest {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: AppManifest = toml::from_str(&content)?;
        Ok(manifest)
    }
}
