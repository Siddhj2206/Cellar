#[cfg(test)]
mod tests {
    use crate::runners::{Runner, RunnerCache, RunnerType};
    use std::path::PathBuf;

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
}