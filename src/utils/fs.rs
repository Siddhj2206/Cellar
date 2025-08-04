use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Expands a path beginning with `~` or `~/` to the user's home directory.
///
/// If the input path starts with `~/`, it is replaced with the user's home directory joined with the remainder of the path. If the path is exactly `"~"`, it is replaced with the home directory. All other paths are returned unchanged.
///
/// # Errors
///
/// Returns an error if the user's home directory cannot be determined.
///
/// # Examples
///
/// ```
/// let home = dirs::home_dir().unwrap();
/// assert_eq!(
///     expand_tilde("~/mydir").unwrap(),
///     home.join("mydir")
/// );
/// assert_eq!(
///     expand_tilde("~").unwrap(),
///     home
/// );
/// assert_eq!(
///     expand_tilde("/tmp/foo").unwrap(),
///     PathBuf::from("/tmp/foo")
/// );
/// ```
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();

    if let Some(without_tilde) = path_str.strip_prefix("~/") {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
        // Remove "~/"
        Ok(home_dir.join(without_tilde))
    } else if path_str == "~" {
        dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))
    } else {
        Ok(path.to_path_buf())
    }
}

pub struct CellarDirectories {
    pub base_dir: PathBuf,
    pub runners_dir: PathBuf,
    pub prefixes_dir: PathBuf,
    pub configs_dir: PathBuf,
    pub icons_dir: PathBuf,
    pub applications_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl CellarDirectories {
    /// Creates a new `CellarDirectories` instance with paths initialized relative to the user's home directory.
    ///
    /// The base cellar directory is set to `~/.local/share/cellar`, with subdirectories for runners, prefixes, configs, icons, and cache. The applications directory is set to `~/.local/share/applications`.
    ///
    /// # Returns
    ///
    /// A `CellarDirectories` struct with all directory paths set. Returns an error if the home directory cannot be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// let dirs = CellarDirectories::new().unwrap();
    /// assert!(dirs.base_dir.ends_with(".local/share/cellar"));
    /// ```
    pub fn new() -> Result<Self> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;

        let base_dir = home_dir.join(".local").join("share").join("cellar");
        let applications_dir = home_dir.join(".local").join("share").join("applications");
        let cache_dir = base_dir.join("cache");

        let dirs = CellarDirectories {
            runners_dir: base_dir.join("runners"),
            prefixes_dir: base_dir.join("prefixes"),
            configs_dir: base_dir.join("configs"),
            icons_dir: base_dir.join("icons"),
            base_dir,
            applications_dir,
            cache_dir,
        };

        Ok(dirs)
    }

    /// Ensures that all required cellar directories and key subdirectories exist, creating them if necessary.
    ///
    /// This includes the main cellar directories as well as the `proton` and `dxvk` subdirectories under the runners directory.
    ///
    /// # Errors
    ///
    /// Returns an error if any directory cannot be created.
    pub fn ensure_all_exist(&self) -> Result<()> {
        self.ensure_dir_exists(&self.base_dir)?;
        self.ensure_dir_exists(&self.runners_dir)?;
        self.ensure_dir_exists(&self.prefixes_dir)?;
        self.ensure_dir_exists(&self.configs_dir)?;
        self.ensure_dir_exists(&self.icons_dir)?;
        self.ensure_dir_exists(&self.applications_dir)?;
        self.ensure_dir_exists(&self.cache_dir)?;

        // Create subdirectories
        self.ensure_dir_exists(&self.runners_dir.join("proton"))?;
        self.ensure_dir_exists(&self.runners_dir.join("dxvk"))?;

        Ok(())
    }

    pub fn ensure_dir_exists(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| anyhow!("Failed to create directory {}: {}", path.display(), e))?;
        }
        Ok(())
    }

    pub fn get_game_config_path(&self, game_name: &str) -> PathBuf {
        self.configs_dir
            .join(format!("{}.toml", sanitize_filename(game_name)))
    }

    // pub fn get_game_prefix_path(&self, game_name: &str) -> PathBuf {
    //     self.prefixes_dir.join(sanitize_filename(game_name))
    // }

    #[allow(dead_code)]
    pub fn get_game_icon_path(&self, game_name: &str, extension: &str) -> PathBuf {
        self.icons_dir
            .join(format!("{}.{}", sanitize_filename(game_name), extension))
    }

    /// Returns the path to a game's desktop shortcut file in the applications directory.
    ///
    /// The filename is constructed as `cellar-<sanitized_game_name>.desktop`, where the game name is sanitized to be filesystem-safe.
    ///
    /// # Examples
    ///
    /// ```
    /// let dirs = CellarDirectories::new().unwrap();
    /// let shortcut_path = dirs.get_game_shortcut_path("My Game!");
    /// assert!(shortcut_path.ends_with("cellar-my_game.desktop"));
    /// ```
    #[allow(dead_code)]
    pub fn get_game_shortcut_path(&self, game_name: &str) -> PathBuf {
        self.applications_dir
            .join(format!("cellar-{}.desktop", sanitize_filename(game_name)))
    }

    pub fn list_game_configs(&self) -> Result<Vec<String>> {
        let mut games = Vec::new();

        if self.configs_dir.exists() {
            for entry in fs::read_dir(&self.configs_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        games.push(stem.to_string());
                    }
                }
            }
        }

        games.sort();
        Ok(games)
    }

    pub fn get_runners_path(&self) -> PathBuf {
        self.runners_dir.clone()
    }

    pub fn get_prefixes_path(&self) -> PathBuf {
        self.prefixes_dir.clone()
    }

    pub fn get_cache_path(&self) -> PathBuf {
        self.cache_dir.clone()
    }
}

/// Converts a string into a safe, lowercase filename by replacing invalid characters and formatting whitespace.
///
/// Replaces characters that are not allowed in filenames (`/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`, and control characters) with underscores, trims whitespace, converts to lowercase, and replaces spaces with underscores.
///
/// # Examples
///
/// ```
/// let sanitized = sanitize_filename("My Game: Deluxe Edition*");
/// assert_eq!(sanitized, "my_game__deluxe_edition_");
/// ```
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_lowercase()
        .replace(' ', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Game"), "my_game");
        assert_eq!(sanitize_filename("Game: The Sequel"), "game__the_sequel");
        assert_eq!(sanitize_filename("Game/Part\\Two"), "game_part_two");
        assert_eq!(sanitize_filename("Game With Spaces"), "game_with_spaces");
        assert_eq!(
            sanitize_filename("Game*With?Special<Chars>"),
            "game_with_special_chars_"
        );
        assert_eq!(sanitize_filename("UPPERCASE GAME"), "uppercase_game");
        assert_eq!(sanitize_filename(""), ""); // Edge case: empty string
        assert_eq!(sanitize_filename("123 Game"), "123_game");
    }

    #[test]
    fn test_cellar_directories_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let dirs = CellarDirectories::new().unwrap();

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

        let dirs = CellarDirectories::new().unwrap();

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
}
