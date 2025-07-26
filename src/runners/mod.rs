pub mod proton;
pub mod dxvk;

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runner {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub runner_type: RunnerType,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunnerType {
    Proton,
    Wine,
    Dxvk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerCache {
    pub runners: Vec<Runner>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl RunnerCache {
    pub fn new() -> Self {
        Self {
            runners: Vec::new(),
            last_updated: chrono::Utc::now(),
        }
    }

    pub fn add_runner(&mut self, runner: Runner) {
        self.runners.push(runner);
        self.last_updated = chrono::Utc::now();
    }

    pub fn find_runner(&self, name: &str, version: Option<&str>) -> Option<&Runner> {
        self.runners.iter().find(|r| {
            r.name == name && 
            (version.is_none() || version == Some(&r.version))
        })
    }

    pub fn get_runners_by_type(&self, runner_type: RunnerType) -> Vec<&Runner> {
        self.runners.iter()
            .filter(|r| std::mem::discriminant(&r.runner_type) == std::mem::discriminant(&runner_type))
            .collect()
    }
}

#[async_trait::async_trait]
pub trait RunnerManager {
    async fn discover_local_runners(&self) -> Result<Vec<Runner>>;
    async fn download_runner(&self, name: &str, version: &str) -> Result<PathBuf>;
    async fn install_runner(&self, download_path: &Path, install_path: &Path) -> Result<()>;
    async fn get_available_versions(&self) -> Result<Vec<String>>;
}