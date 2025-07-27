#[cfg(test)]
mod tests {
    use crate::runners::dxvk::DxvkManager;
    use crate::runners::proton::ProtonManager;
    use crate::runners::{Runner, RunnerCache, RunnerManager, RunnerType};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_proton_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = ProtonManager::new(temp_dir.path().to_path_buf());

        // Test that the manager was created successfully
        assert_eq!(manager.cellar_runners_path, temp_dir.path());
    }

    #[tokio::test]
    async fn test_dxvk_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = DxvkManager::new(temp_dir.path().to_path_buf());

        // Test that the manager was created successfully
        assert_eq!(manager.cellar_runners_path, temp_dir.path());
    }

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

        // Test getting runners by type
        let proton_runners = cache.get_runners_by_type(RunnerType::Proton);
        assert_eq!(proton_runners.len(), 1);

        let wine_runners = cache.get_runners_by_type(RunnerType::Wine);
        assert_eq!(wine_runners.len(), 0);
    }

    #[tokio::test]
    async fn test_proton_discover_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let manager = ProtonManager::new(temp_dir.path().to_path_buf());

        // Test discovering runners in empty directory
        let runners = manager.discover_local_runners().await.unwrap();
        assert_eq!(runners.len(), 0);
    }

    #[tokio::test]
    async fn test_dxvk_discover_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let manager = DxvkManager::new(temp_dir.path().to_path_buf());

        // Test discovering runners in empty directory
        let runners = manager.discover_local_runners().await.unwrap();
        assert_eq!(runners.len(), 0);
    }
}
