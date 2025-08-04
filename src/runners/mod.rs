pub mod common;
pub mod dxvk;
pub mod proton;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

impl Default for RunnerCache {
    fn default() -> Self {
        Self::new()
    }
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
        self.runners
            .iter()
            .find(|r| r.name == name && (version.is_none() || version == Some(&r.version)))
    }

    pub fn get_runners_by_type(&self, runner_type: RunnerType) -> Vec<&Runner> {
        self.runners
            .iter()
            .filter(|r| {
                std::mem::discriminant(&r.runner_type) == std::mem::discriminant(&runner_type)
            })
            .collect()
    }
}

#[async_trait::async_trait]
pub trait RunnerManager {
    async fn discover_local_runners(&self) -> Result<Vec<Runner>>;
    async fn download_runner(&self, name: &str, version: &str) -> Result<PathBuf>;
    async fn install_runner(&self, download_path: &Path, install_path: &Path) -> Result<()>;
    async fn get_available_versions(&self) -> Result<Vec<String>>;
    async fn delete_runner(&self, runner_path: &Path) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runners::dxvk::DxvkManager;
    use crate::runners::proton::ProtonManager;
    use tempfile::TempDir;

    #[test]
    fn test_runner_cache() {
        let mut cache = RunnerCache::new();

        let proton_runner = Runner {
            name: "Test Proton".to_string(),
            version: "1.0".to_string(),
            path: PathBuf::from("/test/path"),
            runner_type: RunnerType::Proton,
            installed: true,
        };

        let wine_runner = Runner {
            name: "Test Wine".to_string(),
            version: "8.0".to_string(),
            path: PathBuf::from("/test/wine"),
            runner_type: RunnerType::Wine,
            installed: true,
        };

        // Test adding runners
        cache.add_runner(proton_runner.clone());
        cache.add_runner(wine_runner.clone());
        assert_eq!(cache.runners.len(), 2);

        // Test finding runner
        let found = cache.find_runner("Test Proton", Some("1.0"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Proton");

        // Test finding runner without version
        let found_no_version = cache.find_runner("Test Proton", None);
        assert!(found_no_version.is_some());

        // Test getting runners by type
        let proton_runners = cache.get_runners_by_type(RunnerType::Proton);
        assert_eq!(proton_runners.len(), 1);
        assert_eq!(proton_runners[0].name, "Test Proton");

        let wine_runners = cache.get_runners_by_type(RunnerType::Wine);
        assert_eq!(wine_runners.len(), 1);
        assert_eq!(wine_runners[0].name, "Test Wine");

        let dxvk_runners = cache.get_runners_by_type(RunnerType::Dxvk);
        assert_eq!(dxvk_runners.len(), 0);
    }

    #[test]
    fn test_runner_creation() {
        let runner = Runner {
            name: "GE-Proton8-32".to_string(),
            version: "8-32".to_string(),
            path: PathBuf::from("/path/to/proton"),
            runner_type: RunnerType::Proton,
            installed: true,
        };

        assert_eq!(runner.name, "GE-Proton8-32");
        assert_eq!(runner.version, "8-32");
        assert!(runner.installed);
    }

    #[tokio::test]
    async fn test_proton_manager_initialization() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let proton_manager = ProtonManager::new(temp_dir.path().to_path_buf());

        // Test that we can create a ProtonManager
        assert!(proton_manager.base_runner.cellar_runners_path.exists());
    }

    #[tokio::test]
    async fn test_dxvk_manager_initialization() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dxvk_manager = DxvkManager::new(temp_dir.path().to_path_buf());

        // Test that we can create a DxvkManager
        assert!(dxvk_manager.base_runner.cellar_runners_path.exists());
    }

    #[tokio::test]
    async fn test_proton_discover_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let proton_manager = ProtonManager::new(temp_dir.path().to_path_buf());

        // Test discovering runners in empty directory
        let runners = proton_manager
            .discover_local_runners()
            .await
            .expect("Failed to discover runners");

        // Should return empty list for empty directory
        assert!(runners.is_empty());
    }

    #[tokio::test]
    async fn test_dxvk_discover_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dxvk_manager = DxvkManager::new(temp_dir.path().to_path_buf());

        // Test discovering runners in empty directory
        let runners = dxvk_manager
            .discover_local_runners()
            .await
            .expect("Failed to discover runners");

        // Should return empty list for empty directory
        assert!(runners.is_empty());
    }

    #[tokio::test]
    async fn test_runner_deletion_nonexistent_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let proton_manager = ProtonManager::new(temp_dir.path().to_path_buf());

        let nonexistent_path = temp_dir.path().join("nonexistent");
        let result = proton_manager.delete_runner(&nonexistent_path).await;

        // Should return error for nonexistent path
        assert!(result.is_err());
    }
}
