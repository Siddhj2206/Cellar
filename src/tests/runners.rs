#[cfg(test)]
mod tests {
    use crate::runners::{Runner, RunnerCache, RunnerType};
    use crate::runners::proton::ProtonManager;
    use crate::runners::dxvk::DxvkManager;
    use crate::runners::RunnerManager;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_runner_cache() {
        let mut cache = RunnerCache::new();

        let runner = Runner {
            name: "Test Runner".to_string(),
            version: "1.0".to_string(),
            path: PathBuf::from("/test/path"),
            runner_type: RunnerType::Proton,
            installed: true,
        };

        // Test adding runner
        cache.add_runner(runner.clone());
        assert_eq!(cache.runners.len(), 1);

        // Test finding runner
        let found = cache.find_runner("Test Runner", Some("1.0"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Runner");

        // Test finding runner without version
        let found_no_version = cache.find_runner("Test Runner", None);
        assert!(found_no_version.is_some());
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
        let runners = proton_manager.discover_local_runners().await
            .expect("Failed to discover runners");
        
        // Should return empty list for empty directory
        assert!(runners.is_empty());
    }

    #[tokio::test]
    async fn test_dxvk_discover_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dxvk_manager = DxvkManager::new(temp_dir.path().to_path_buf());
        
        // Test discovering runners in empty directory
        let runners = dxvk_manager.discover_local_runners().await
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