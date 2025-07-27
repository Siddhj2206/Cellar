#[cfg(test)]
mod tests {
    use crate::utils::fs::sanitize_filename;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Game"), "my_game");
        assert_eq!(sanitize_filename("Game: The Sequel"), "game__the_sequel");
        assert_eq!(sanitize_filename("Game/Part\\Two"), "game_part_two");
        assert_eq!(sanitize_filename("Game With Spaces"), "game_with_spaces");
        assert_eq!(sanitize_filename("Game*With?Special<Chars>"), "game_with_special_chars_");
        assert_eq!(sanitize_filename("UPPERCASE GAME"), "uppercase_game");
        assert_eq!(sanitize_filename(""), ""); // Edge case: empty string
        assert_eq!(sanitize_filename("123 Game"), "123_game");
    }

    #[test]
    fn test_cellar_directories_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());
        
        let dirs = crate::utils::fs::CellarDirectories::new().unwrap();
        
        // Test that directory creation works
        assert!(dirs.base_dir.ends_with("cellar"));
        
        // Test game config path generation
        let config_path = dirs.get_game_config_path("test-game");
        assert!(config_path.to_string_lossy().contains("test-game.toml"));
        
        // Test runners path
        let runners_path = dirs.get_runners_path();
        assert!(runners_path.ends_with("runners"));
        
        // Test prefixes path  
        let prefixes_path = dirs.get_prefixes_path();
        assert!(prefixes_path.ends_with("prefixes"));
    }

    #[test]
    fn test_directory_structure() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());
        
        let dirs = crate::utils::fs::CellarDirectories::new().unwrap();
        
        // Test that we can ensure directories exist
        let result = dirs.ensure_all_exist();
        assert!(result.is_ok());
        
        // Test that the directories were created
        assert!(dirs.base_dir.exists());
        assert!(dirs.configs_dir.exists());
        assert!(dirs.runners_dir.exists());
        assert!(dirs.prefixes_dir.exists());
    }
}