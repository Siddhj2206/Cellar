#[cfg(test)]
mod tests {
    use crate::config::validation::{validate_directory_path, validate_file_path};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_validate_file_path() {
        let temp_file = "/tmp/test_validate_file.txt";
        fs::write(temp_file, "test").unwrap();

        assert!(validate_file_path(&PathBuf::from(temp_file), "Test file").is_ok());
        assert!(
            validate_file_path(&PathBuf::from("/nonexistent/file.txt"), "Nonexistent file")
                .is_err()
        );

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_validate_directory_path() {
        assert!(validate_directory_path(&PathBuf::from("/tmp"), "Temp directory").is_ok());
        assert!(validate_directory_path(
            &PathBuf::from("/nonexistent/dir"),
            "Nonexistent directory"
        )
        .is_err());
    }
}
