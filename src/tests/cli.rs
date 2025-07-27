#[cfg(test)]
mod tests {
    // Simple CLI integration tests
    use std::path::PathBuf;

    #[test]
    fn test_basic_config_loading() {
        // Test that we can create game configs
        let config = crate::config::game::GameConfig {
            game: crate::config::game::GameInfo {
                name: "Test Game".to_string(),
                executable: PathBuf::from("/tmp/test.exe"),
                wine_prefix: PathBuf::from("/tmp/prefix"),
                proton_version: "GE-Proton8-32".to_string(),
                dxvk_version: None,
                status: "configured".to_string(),
                template: None,
                preset: None,
            },
            launch: crate::config::game::LaunchConfig::default(),
            wine_config: crate::config::game::WineConfig::default(),
            dxvk: crate::config::game::DxvkConfig::default(),
            gamescope: crate::config::game::GamescopeConfig::default(),
            mangohud: crate::config::game::MangohudConfig::default(),
            desktop: crate::config::game::DesktopConfig::default(),
            dependencies: crate::config::game::DependenciesConfig::default(),
            installation: None,
        };

        // Basic validation
        assert_eq!(config.game.name, "Test Game");
        assert!(config.wine_config.dxvk);
    }

    #[test]
    fn test_directory_structure() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let dirs = crate::utils::fs::CellarDirectories::new().unwrap();

        // Test basic functionality
        let config_path = dirs.get_game_config_path("test-game");
        assert!(config_path.to_string_lossy().contains("test-game.toml"));

        // Test runners path functionality
        let runners_path = dirs.get_runners_path();
        assert!(runners_path.ends_with("runners"));
    }

    #[test]
    fn test_runner_cache_basic() {
        let mut cache = crate::runners::RunnerCache::new();

        let runner = crate::runners::Runner {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            path: PathBuf::from("/test"),
            runner_type: crate::runners::RunnerType::Proton,
            installed: true,
        };

        cache.add_runner(runner);
        assert_eq!(cache.runners.len(), 1);
    }
}
