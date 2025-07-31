use anyhow::{anyhow, Result};
use std::path::Path;

use super::game::GameConfig;

pub fn validate_game_config(config: &GameConfig) -> Result<()> {
    // Validate game name
    if config.game.name.is_empty() {
        return Err(anyhow!("Game name cannot be empty"));
    }

    // Validate executable path
    if !config.game.executable.exists() {
        return Err(anyhow!(
            "Executable does not exist: {}",
            config.game.executable.display()
        ));
    }

    // Validate wine prefix path
    let prefix_parent = config
        .game
        .wine_prefix
        .parent()
        .ok_or_else(|| anyhow!("Invalid wine prefix path"))?;

    if !prefix_parent.exists() {
        return Err(anyhow!(
            "Wine prefix parent directory does not exist: {}",
            prefix_parent.display()
        ));
    }

    // Validate proton version format
    if config.game.proton_version.is_empty() {
        return Err(anyhow!("Proton version cannot be empty"));
    }

    // Validate gamescope configuration
    if config.gamescope.enabled {
        validate_gamescope_config(&config.gamescope)?;
    }

    // Validate desktop configuration
    validate_desktop_config(&config.desktop)?;

    Ok(())
}

fn validate_gamescope_config(config: &super::game::GamescopeConfig) -> Result<()> {
    if config.width == 0 || config.height == 0 {
        return Err(anyhow!("Gamescope width and height must be greater than 0"));
    }

    if config.output_width == 0 || config.output_height == 0 {
        return Err(anyhow!("Gamescope output width and height must be greater than 0"));
    }

    if config.refresh_rate == 0 {
        return Err(anyhow!("Gamescope refresh rate must be greater than 0"));
    }

    let valid_upscaling = ["fsr", "nis", "integer", "stretch", "linear", "nearest", "off"];
    if !valid_upscaling.contains(&config.upscaling.as_str()) {
        return Err(anyhow!(
            "Invalid upscaling method '{}'. Must be one of: {}",
            config.upscaling,
            valid_upscaling.join(", ")
        ));
    }

    Ok(())
}

fn validate_desktop_config(config: &super::game::DesktopConfig) -> Result<()> {
    if config.categories.is_empty() {
        return Err(anyhow!("Desktop categories cannot be empty"));
    }

    if let Some(icon_path) = &config.icon_path {
        if !icon_path.exists() {
            return Err(anyhow!("Icon file does not exist: {}", icon_path.display()));
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn validate_file_path(path: &Path, description: &str) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!(
            "{} does not exist: {}",
            description,
            path.display()
        ));
    }

    if !path.is_file() {
        return Err(anyhow!("{} is not a file: {}", description, path.display()));
    }

    Ok(())
}

#[allow(dead_code)]
pub fn validate_directory_path(path: &Path, description: &str) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!(
            "{} does not exist: {}",
            description,
            path.display()
        ));
    }

    if !path.is_dir() {
        return Err(anyhow!(
            "{} is not a directory: {}",
            description,
            path.display()
        ));
    }

    Ok(())
}
