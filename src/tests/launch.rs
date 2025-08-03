#[cfg(test)]
mod tests {
    use crate::config::game::*;
    use crate::launch::command::*;
    use std::path::PathBuf;

    fn create_test_config() -> GameConfig {
        GameConfig {
            game: GameInfo {
                name: "Test Game".to_string(),
                executable: PathBuf::from("/path/to/game.exe"),
                wine_prefix: PathBuf::from("/path/to/prefix"),
                proton_version: "GE-Proton8-32".to_string(),
                dxvk_version: None,
            },
            launch: LaunchConfig {
                launch_options: "PROTON_ENABLE_WAYLAND=1 gamemoderun %command%".to_string(),
                game_args: vec!["--windowed".to_string(), "--dx11".to_string()],
            },
            wine_config: WineConfig::default(),
            dxvk: DxvkConfig::default(),
            gamescope: GamescopeConfig::default(),
            mangohud: MangohudConfig::default(),
            desktop: DesktopConfig::default(),

            installation: None,
        }
    }

    #[test] 
    fn test_build_complete_launch_command() {
        let config = create_test_config();
        let builder = CommandBuilder::new(config)
            .with_proton_path(PathBuf::from("/path/to/proton"));
        
        let launch_command = builder.build().unwrap();
        
        // Verify command structure
        let args = launch_command.command.as_args();
        assert!(!args.is_empty());
        assert!(args.contains(&"umu-run".to_string()));
        assert!(args.contains(&"/path/to/game.exe".to_string()));
        
        // Verify environment variables
        assert!(launch_command.environment.contains_key("WINEPREFIX"));
        assert!(launch_command.environment.contains_key("PROTONPATH"));
        assert!(launch_command.environment.contains_key("WINEARCH"));
        
        // Verify working directory
        assert_eq!(launch_command.working_directory, PathBuf::from("/path/to/prefix"));
    }

    #[test]
    fn test_command_builder_creation() {
        let config = create_test_config();
        let builder = CommandBuilder::new(config);
        
        // Test that we can create a builder without errors
        let builder_with_proton = builder.with_proton_path(PathBuf::from("/path/to/proton"));
        
        // This should not panic
        drop(builder_with_proton);
    }

    #[test]
    fn test_processed_command_types() {
        // Test ProcessedCommand enum
        let direct_cmd = ProcessedCommand::Direct(vec!["umu-run".to_string(), "game.exe".to_string()]);
        let gamescope_cmd = ProcessedCommand::Gamescope(vec!["gamescope".to_string(), "--".to_string(), "umu-run".to_string()]);
        
        assert_eq!(direct_cmd.as_args().len(), 2);
        assert_eq!(direct_cmd.program(), "umu-run");
        assert_eq!(direct_cmd.args()[0], "game.exe");
        
        assert_eq!(gamescope_cmd.as_args().len(), 3);
        assert_eq!(gamescope_cmd.program(), "gamescope");
    }
}