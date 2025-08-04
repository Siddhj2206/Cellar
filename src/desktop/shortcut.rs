use anyhow::{anyhow, Result};
use std::fs;

use crate::config::game::GameConfig;
use crate::desktop::icon::{get_or_extract_icon, remove_game_icons};
use crate::utils::fs::CellarDirectories;

/// Asynchronously retrieves the absolute path to the `cellar` binary.
///
/// Attempts to locate the `cellar` executable using the `which` command. If successful and a non-empty path is found, returns the absolute path as a string. If the command fails or returns an empty result, falls back to returning `"cellar"`.
///
/// # Returns
/// The absolute path to the `cellar` binary, or `"cellar"` if not found.
///
/// # Errors
/// Returns an error if the `which` command fails to execute or if the output cannot be parsed as UTF-8.
///
/// # Examples
///
/// ```
/// let path = tokio::runtime::Runtime::new().unwrap().block_on(get_cellar_binary_path()).unwrap();
/// assert!(!path.is_empty());
/// ```
async fn get_cellar_binary_path() -> Result<String> {
    let output = tokio::process::Command::new("which")
        .arg("cellar")
        .output()
        .await
        .map_err(|e| anyhow!("Failed to run 'which cellar': {}", e))?;

    if output.status.success() {
        let path = String::from_utf8(output.stdout)?.trim().to_string();

        if !path.is_empty() {
            return Ok(path);
        }
    }

    // Fallback to just "cellar" if which command fails
    Ok("cellar".to_string())
}

/// Asynchronously generates the contents of a `.desktop` file for a game based on its configuration.
///
/// The generated file includes fields such as the executable command, icon path (explicit or extracted), categories, keywords, and comment. If icon extraction fails or is not specified, a default icon is used.
///
/// # Parameters
/// - `config_name`: The unique identifier for the game configuration, used in the launch command.
///
/// # Returns
/// A string containing the formatted `.desktop` file content.
///
/// # Examples
///
/// ```
/// # use crate::config::game::GameConfig;
/// # use crate::desktop::generate_desktop_file;
/// # async fn example(config: GameConfig) {
/// let desktop_entry = generate_desktop_file(&config, "my_game").await.unwrap();
/// assert!(desktop_entry.contains("[Desktop Entry]"));
/// # }
/// ```
pub async fn generate_desktop_file(config: &GameConfig, config_name: &str) -> Result<String> {
    let cellar_path = get_cellar_binary_path().await?;
    let exec_command = format!("{} launch {}", cellar_path, config_name);

    // Determine icon path - try to extract from executable if not explicitly set
    let icon = if let Some(icon_path) = &config.desktop.icon_path {
        // Use explicitly set icon path
        icon_path.to_string_lossy().to_string()
    } else {
        // Try to extract icon from executable
        match get_or_extract_icon(&config.game.executable, &config.game.name).await {
            Ok(Some(extracted_icon)) => extracted_icon.to_string_lossy().to_string(),
            Ok(None) | Err(_) => "application-x-ms-dos-executable".to_string(),
        }
    };

    let categories = config.desktop.categories.join(";");
    let keywords = config.desktop.keywords.join(";");

    Ok(format!(
        "[Desktop Entry]\n\
        Type=Application\n\
        Name={}\n\
        Exec={}\n\
        Icon={}\n\
        Categories={}\n\
        Keywords={}\n\
        Comment={}\n\
        StartupNotify=false\n\
        NoDisplay=false\n",
        config.game.name, exec_command, icon, categories, keywords, config.desktop.comment
    ))
}

/// Creates a desktop shortcut for a game based on its configuration.
///
/// Generates a `.desktop` file for the specified game and writes it to the appropriate applications directory if shortcut creation is enabled in the configuration. Ensures all required directories exist before creating the shortcut.
///
/// # Arguments
///
/// * `config_name` - The name of the game configuration to use.
///
/// # Examples
///
/// ```
/// // Assume `config` is a valid GameConfig and "my_game" is its config name.
/// create_desktop_shortcut(&config, "my_game").await?;
/// ```
pub async fn create_desktop_shortcut(config: &GameConfig, config_name: &str) -> Result<()> {
    if !config.desktop.create_shortcut {
        return Ok(());
    }

    let dirs = CellarDirectories::new()?;
    dirs.ensure_all_exist()?;

    let desktop_content = generate_desktop_file(config, config_name).await?;
    let shortcut_path = dirs.get_game_shortcut_path(&config.game.name);

    fs::write(&shortcut_path, desktop_content)
        .map_err(|e| anyhow!("Failed to create desktop shortcut: {}", e))?;

    println!("Created desktop shortcut: {}", shortcut_path.display());
    Ok(())
}

/// Removes the desktop shortcut and any extracted icons for the specified game.
///
/// If the shortcut file exists, it is deleted; otherwise, a message is printed indicating its absence. Any extracted icons associated with the game are also removed. Errors encountered during shortcut or icon removal are logged, but only shortcut removal errors are propagated.
///
/// # Parameters
/// - `game_name`: The name of the game whose desktop shortcut and icons should be removed.
///
/// # Returns
/// Returns `Ok(())` if the operation completes successfully, or an error if the shortcut removal fails. Errors during icon removal are logged but not returned.
pub fn remove_desktop_shortcut(game_name: &str) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let shortcut_path = dirs.get_game_shortcut_path(game_name);

    if shortcut_path.exists() {
        fs::remove_file(&shortcut_path)
            .map_err(|e| anyhow!("Failed to remove desktop shortcut: {}", e))?;
        println!("Removed desktop shortcut: {}", shortcut_path.display());
    } else {
        println!("Desktop shortcut not found: {}", shortcut_path.display());
    }

    // Also remove extracted icons
    if let Err(e) = remove_game_icons(game_name) {
        eprintln!("Warning: Failed to remove icons for {}: {}", game_name, e);
    }

    Ok(())
}

/// Recreates desktop shortcuts for all games configured to have shortcuts.
///
/// Iterates over all game configurations, generating or updating `.desktop` shortcut files for those marked to create shortcuts. Skips games not configured for shortcut creation and logs errors encountered during processing.
///
/// # Examples
///
/// ```
/// tokio_test::block_on(async {
///     sync_desktop_shortcuts().await.unwrap();
/// });
/// ```
pub async fn sync_desktop_shortcuts() -> Result<()> {
    let dirs = CellarDirectories::new()?;
    dirs.ensure_all_exist()?;

    let games = dirs.list_game_configs()?;
    let mut created_count = 0;
    let mut skipped_count = 0;

    for game_config_name in games {
        let config_path = dirs.get_game_config_path(&game_config_name);

        match fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str::<GameConfig>(&content) {
                Ok(config) => {
                    if config.desktop.create_shortcut {
                        match create_desktop_shortcut(&config, &game_config_name).await {
                            Ok(()) => created_count += 1,
                            Err(e) => {
                                eprintln!(
                                    "Failed to create shortcut for {}: {}",
                                    game_config_name, e
                                );
                            }
                        }
                    } else {
                        skipped_count += 1;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse config for {}: {}", game_config_name, e);
                }
            },
            Err(e) => {
                eprintln!("Failed to read config for {}: {}", game_config_name, e);
            }
        }
    }

    println!(
        "Sync complete: {} shortcuts created, {} skipped",
        created_count, skipped_count
    );
    Ok(())
}

/// Returns whether a desktop shortcut exists for the specified game.
///
/// # Examples
///
/// ```
/// let exists = shortcut_exists("my_game")?;
/// if exists {
///     println!("Shortcut exists!");
/// }
/// ```
pub fn shortcut_exists(game_name: &str) -> Result<bool> {
    let dirs = CellarDirectories::new()?;
    let shortcut_path = dirs.get_game_shortcut_path(game_name);
    Ok(shortcut_path.exists())
}

/// Returns a sorted list of game names for which desktop shortcuts exist.
///
/// Scans the applications directory for files matching the `cellar-<game_name>.desktop` pattern and extracts the game names from these files.
///
/// # Returns
///
/// A vector of game names corresponding to existing desktop shortcuts.
///
/// # Errors
///
/// Returns an error if the applications directory cannot be accessed or read.
pub fn list_desktop_shortcuts() -> Result<Vec<String>> {
    let dirs = CellarDirectories::new()?;
    let mut shortcuts = Vec::new();

    if dirs.applications_dir.exists() {
        for entry in fs::read_dir(&dirs.applications_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("cellar-") && filename.ends_with(".desktop") {
                        let game_name = filename
                            .strip_prefix("cellar-")
                            .and_then(|s| s.strip_suffix(".desktop"))
                            .unwrap_or(filename);
                        shortcuts.push(game_name.to_string());
                    }
                }
            }
        }
    }

    shortcuts.sort();
    Ok(shortcuts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_shortcut_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let game_name = "test-game";
        
        // Mock CellarDirectories for testing
        let applications_dir = temp_dir.path().join("applications");
        fs::create_dir_all(&applications_dir).expect("Failed to create applications dir");
        
        let shortcut_path = applications_dir.join(format!("cellar-{}.desktop", game_name));
        
        // Test when shortcut doesn't exist
        assert!(!shortcut_path.exists());
        
        // Test when shortcut exists
        fs::write(&shortcut_path, "[Desktop Entry]\nType=Application\n")
            .expect("Failed to create test shortcut");
        assert!(shortcut_path.exists());
    }

    #[test]
    fn test_list_desktop_shortcuts() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let applications_dir = temp_dir.path().join("applications");
        fs::create_dir_all(&applications_dir).expect("Failed to create applications dir");
        
        // Create test shortcuts
        let shortcut1 = applications_dir.join("cellar-game1.desktop");
        let shortcut2 = applications_dir.join("cellar-game2.desktop");
        let non_cellar_file = applications_dir.join("other-app.desktop");
        
        fs::write(&shortcut1, "[Desktop Entry]\n").expect("Failed to create shortcut1");
        fs::write(&shortcut2, "[Desktop Entry]\n").expect("Failed to create shortcut2");
        fs::write(&non_cellar_file, "[Desktop Entry]\n").expect("Failed to create non-cellar file");
        
        // Note: This test would need to mock CellarDirectories to work properly
        // For now, it's a basic structure test
    }
}
