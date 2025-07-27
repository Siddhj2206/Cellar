#[cfg(test)]
mod tests {
    use crate::config::game::*;

    #[test]
    fn test_game_config_basic_functionality() {
        let config = GameConfig {
            game: GameInfo {
                name: "Test Game".to_string(),
                executable: std::path::PathBuf::from("/path/to/game.exe"),
                wine_prefix: std::path::PathBuf::from("/path/to/prefix"),
                proton_version: "GE-Proton8-32".to_string(),
                dxvk_version: None,
                status: "configured".to_string(),
                template: None,
                preset: None,
            },
            launch: LaunchConfig::default(),
            wine_config: WineConfig::default(),
            dxvk: DxvkConfig::default(),
            gamescope: GamescopeConfig::default(),
            mangohud: MangohudConfig::default(),
            desktop: DesktopConfig::default(),
            dependencies: DependenciesConfig::default(),
            installation: None,
        };

        assert_eq!(config.game.name, "Test Game");
        assert_eq!(config.game.proton_version, "GE-Proton8-32");
        assert_eq!(config.game.status, "configured");
        assert!(config.wine_config.esync);
        assert!(config.mangohud.enabled);
        assert!(!config.gamescope.enabled);
    }

    #[test]
    fn test_game_config_serialization() {
        let config = GameConfig {
            game: GameInfo {
                name: "Test Game".to_string(),
                executable: std::path::PathBuf::from("/path/to/game.exe"),
                wine_prefix: std::path::PathBuf::from("/path/to/prefix"),
                proton_version: "GE-Proton8-32".to_string(),
                dxvk_version: None,
                status: "configured".to_string(),
                template: None,
                preset: None,
            },
            launch: LaunchConfig::default(),
            wine_config: WineConfig::default(),
            dxvk: DxvkConfig::default(),
            gamescope: GamescopeConfig::default(),
            mangohud: MangohudConfig::default(),
            desktop: DesktopConfig::default(),
            dependencies: DependenciesConfig::default(),
            installation: None,
        };

        let toml_string = toml::to_string(&config).unwrap();
        
        // Verify it contains expected sections
        assert!(toml_string.contains("[game]"));
        assert!(toml_string.contains("[launch]"));
        assert!(toml_string.contains("[wine_config]"));
        assert!(toml_string.contains("Test Game"));
        assert!(toml_string.contains("GE-Proton8-32"));
    }

    #[test]
    fn test_game_config_deserialization() {
        let toml_string = r#"
[game]
name = "Test Game"
executable = "/path/to/game.exe"
wine_prefix = "/path/to/prefix"
proton_version = "GE-Proton8-32"
status = "configured"

[launch]
launch_options = "PROTON_ENABLE_WAYLAND=1 %command%"
game_args = ["--windowed"]

[wine_config]
esync = true
fsync = true
dxvk = true
dxvk_async = true
large_address_aware = false
wineserver_kill_timeout = 5
"#;

        let config: GameConfig = toml::from_str(toml_string).unwrap();
        assert_eq!(config.game.name, "Test Game");
        assert_eq!(config.game.proton_version, "GE-Proton8-32");
        assert_eq!(config.launch.game_args, vec!["--windowed"]);
        assert!(config.wine_config.esync);
    }
}