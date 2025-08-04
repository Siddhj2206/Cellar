use anyhow::{anyhow, Result};
use std::fs;

use crate::config::game::GameConfig;
use crate::desktop::icon::{get_or_extract_icon, remove_game_icons};
use crate::utils::fs::CellarDirectories;

/// Get the full path to the cellar binary using 'which cellar'
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

/// Generate a .desktop file for a game
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

/// Create a desktop shortcut for a game
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

/// Remove a desktop shortcut for a game
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

/// Sync all desktop shortcuts (recreate all shortcuts for configured games)
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

/// Check if a desktop shortcut exists for a game
pub fn shortcut_exists(game_name: &str) -> Result<bool> {
    let dirs = CellarDirectories::new()?;
    let shortcut_path = dirs.get_game_shortcut_path(game_name);
    Ok(shortcut_path.exists())
}

/// List all existing desktop shortcuts
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
