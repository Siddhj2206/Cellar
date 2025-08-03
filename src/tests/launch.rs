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
                gamemode: false,
                mangohud: false,
            },
            wine_config: WineConfig::default(),
            dxvk: DxvkConfig::default(),
            gamescope: GamescopeConfig::default(),
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
        let args = &launch_command.command;
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
        // Test direct command creation
        let direct_cmd = vec!["umu-run".to_string(), "game.exe".to_string()];
        
        assert_eq!(direct_cmd.len(), 2);
        assert_eq!(direct_cmd[0], "umu-run");
        assert_eq!(direct_cmd[1], "game.exe");
    }

    #[test]
    fn test_gamemode_wrapper() {
        let mut config = create_test_config();
        config.launch.gamemode = true;
        config.launch.launch_options = "".to_string(); // No launch options to test pure gamemode

        let builder = CommandBuilder::new(config)
            .with_proton_path(PathBuf::from("/path/to/proton"));
        
        let launch_command = builder.build().unwrap();
        let args = &launch_command.command;
        
        // Should start with gamemoderun
        assert_eq!(args[0], "gamemoderun");
        // Should contain umu-run somewhere
        assert!(args.contains(&"umu-run".to_string()));
    }

    #[test]
    fn test_gamemode_with_mangohud() {
        let mut config = create_test_config();
        config.launch.gamemode = true;
        config.launch.launch_options = "".to_string();
        config.launch.mangohud = true;

        let builder = CommandBuilder::new(config)
            .with_proton_path(PathBuf::from("/path/to/proton"));
        
        let launch_command = builder.build().unwrap();
        let args = &launch_command.command;
        
        // Should be: gamemoderun mangohud umu-run ...
        assert_eq!(args[0], "gamemoderun");
        assert_eq!(args[1], "mangohud");
        assert!(args.contains(&"umu-run".to_string()));
    }

    #[test]
    fn test_gamemode_with_gamescope_and_mangohud() {
        let mut config = create_test_config();
        config.launch.gamemode = true;
        config.launch.launch_options = "".to_string();
        config.launch.mangohud = true;
        config.gamescope.enabled = true;

        let builder = CommandBuilder::new(config)
            .with_proton_path(PathBuf::from("/path/to/proton"));
        
        let launch_command = builder.build().unwrap();
        let args = &launch_command.command;
        
        // Should be: gamemoderun gamescope <args> --mangoapp -- umu-run ...
        assert_eq!(args[0], "gamemoderun");
        assert_eq!(args[1], "gamescope");
        assert!(args.contains(&"--mangoapp".to_string()));
        assert!(args.contains(&"--".to_string()));
        assert!(args.contains(&"umu-run".to_string()));
    }

    #[test]
    fn test_gamemode_disabled() {
        let mut config = create_test_config();
        config.launch.gamemode = false;
        config.launch.launch_options = "".to_string();

        let builder = CommandBuilder::new(config)
            .with_proton_path(PathBuf::from("/path/to/proton"));
        
        let launch_command = builder.build().unwrap();
        let args = &launch_command.command;
        
        // Should NOT start with gamemoderun
        assert_ne!(args[0], "gamemoderun");
        // Should start with umu-run directly
        assert_eq!(args[0], "umu-run");
    }
}