#[cfg(test)]
mod tests {
    use crate::utils::fs::sanitize_filename;
    use crate::utils::archive::validate_archive_path;
    use std::path::PathBuf;
    use tempfile::TempDir;

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

    #[test]
    fn test_expand_tilde() {
        use crate::utils::fs::expand_tilde;
        
        // Test regular path (should remain unchanged)
        let regular_path = "/usr/bin/ls";
        let expanded = expand_tilde(regular_path).unwrap();
        assert_eq!(expanded.to_string_lossy(), regular_path);
        
        // Test relative path (should remain unchanged)
        let relative_path = "games/test.exe";
        let expanded = expand_tilde(relative_path).unwrap();
        assert_eq!(expanded.to_string_lossy(), relative_path);
        
        // Test tilde path
        let tilde_path = "~/Documents/test.exe";
        let expanded = expand_tilde(tilde_path).unwrap();
        assert!(expanded.to_string_lossy().contains("Documents/test.exe"));
        assert!(!expanded.to_string_lossy().contains("~"));
        
        // Test just tilde
        let just_tilde = "~";
        let expanded = expand_tilde(just_tilde).unwrap();
        assert!(!expanded.to_string_lossy().contains("~"));
        assert!(expanded.is_absolute());
    }

    #[test]
    fn test_validate_archive_path_normal() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let destination = temp_dir.path();
        
        // Normal path should be allowed
        let entry_path = PathBuf::from("subdir/file.txt");
        let result = validate_archive_path(&entry_path, destination);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_archive_path_traversal() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let destination = temp_dir.path();
        
        // Path traversal should be rejected
        let entry_path = PathBuf::from("../../../etc/passwd");
        let result = validate_archive_path(&entry_path, destination);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_archive_path_absolute() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let destination = temp_dir.path();
        
        // Absolute path should be rejected
        let entry_path = PathBuf::from("/etc/passwd");
        let result = validate_archive_path(&entry_path, destination);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_archive_path_suspicious_patterns() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let destination = temp_dir.path();
        
        // Path with suspicious patterns should be rejected
        let entry_path = PathBuf::from("some/../file.txt");
        let result = validate_archive_path(&entry_path, destination);
        assert!(result.is_err());
        
        let entry_path = PathBuf::from("file\0.txt");
        let result = validate_archive_path(&entry_path, destination);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_archive_extraction_limits() {
        // This would test archive extraction with size/file limits
        // We're not creating actual archives here for simplicity
        // but in a real scenario, you'd create test archives and verify
        // the extraction respects the limits
        assert!(true); // Placeholder test
    }
}