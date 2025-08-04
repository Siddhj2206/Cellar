use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use crate::utils::fs::CellarDirectories;

/// Extract icon from executable using wrestool and convert to PNG using ImageMagick
pub async fn extract_and_convert_icon(exe_path: &Path, game_name: &str) -> Result<PathBuf> {
    let dirs = CellarDirectories::new()?;
    dirs.ensure_all_exist()?;

    // Check if tools are available
    check_required_tools().await?;

    // Paths for intermediate and final icon files
    let ico_path = dirs.get_game_icon_path(game_name, "ico");
    let png_path = dirs.get_game_icon_path(game_name, "png");

    // Step 1: Extract icon using wrestool
    extract_icon_with_wrestool(exe_path, &ico_path).await?;

    // Step 2: Convert ICO to PNG using ImageMagick
    convert_ico_to_png(&ico_path, &png_path).await?;

    // Clean up intermediate ICO file
    if ico_path.exists() {
        std::fs::remove_file(&ico_path)?;
    }

    Ok(png_path)
}

/// Check if wrestool and magick commands are available
async fn check_required_tools() -> Result<()> {
    // Check for wrestool
    let wrestool_check = tokio::process::Command::new("which")
        .arg("wrestool")
        .output()
        .await?;

    if !wrestool_check.status.success() {
        return Err(anyhow!(
            "wrestool not found. Please install icoutils package (e.g., 'sudo apt install icoutils' on Ubuntu)"
        ));
    }

    // Check for magick (ImageMagick)
    let magick_check = tokio::process::Command::new("which")
        .arg("magick")
        .output()
        .await?;

    if !magick_check.status.success() {
        return Err(anyhow!(
            "magick not found. Please install ImageMagick package (e.g., 'sudo apt install imagemagick' on Ubuntu)"
        ));
    }

    Ok(())
}

/// Extract icon from executable using wrestool
async fn extract_icon_with_wrestool(exe_path: &Path, output_path: &Path) -> Result<()> {
    let output = tokio::process::Command::new("wrestool")
        .arg("-x")
        .arg("-t")
        .arg("14") // Icon resource type
        .arg(exe_path)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "Failed to extract icon from {}: {}",
            exe_path.display(),
            stderr
        ));
    }

    if output.stdout.is_empty() {
        return Err(anyhow!(
            "No icon found in executable: {}",
            exe_path.display()
        ));
    }

    // Write the extracted icon data to file
    std::fs::write(output_path, &output.stdout)?;

    Ok(())
}

/// Convert ICO file to PNG using ImageMagick
async fn convert_ico_to_png(ico_path: &Path, png_path: &Path) -> Result<()> {
    // Use [0] to get the highest resolution icon from the ICO file
    let ico_input = format!("{}[0]", ico_path.display());

    let output = tokio::process::Command::new("magick")
        .arg(&ico_input)
        .arg(png_path)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "Failed to convert icon {} to PNG: {}",
            ico_path.display(),
            stderr
        ));
    }

    Ok(())
}

/// Get icon path for a game, extracting if necessary
pub async fn get_or_extract_icon(exe_path: &Path, game_name: &str) -> Result<Option<PathBuf>> {
    let dirs = CellarDirectories::new()?;
    let png_path = dirs.get_game_icon_path(game_name, "png");

    // If PNG icon already exists, return it
    if png_path.exists() {
        return Ok(Some(png_path));
    }

    // Try to extract and convert icon
    match extract_and_convert_icon(exe_path, game_name).await {
        Ok(icon_path) => {
            println!(
                "Extracted icon for {} to {}",
                game_name,
                icon_path.display()
            );
            Ok(Some(icon_path))
        }
        Err(e) => {
            eprintln!("Warning: Failed to extract icon for {}: {}", game_name, e);
            eprintln!("Desktop shortcut will use default icon.");
            Ok(None)
        }
    }
}

/// Remove icon files for a game
pub fn remove_game_icons(game_name: &str) -> Result<()> {
    let dirs = CellarDirectories::new()?;

    // Remove both ICO and PNG versions if they exist
    let ico_path = dirs.get_game_icon_path(game_name, "ico");
    let png_path = dirs.get_game_icon_path(game_name, "png");

    if ico_path.exists() {
        std::fs::remove_file(&ico_path)?;
    }

    if png_path.exists() {
        std::fs::remove_file(&png_path)?;
        println!("Removed icon: {}", png_path.display());
    }

    Ok(())
}

/// List all extracted game icons
pub fn list_game_icons() -> Result<Vec<String>> {
    let dirs = CellarDirectories::new()?;
    let mut icons = Vec::new();

    if dirs.icons_dir.exists() {
        for entry in std::fs::read_dir(&dirs.icons_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.ends_with(".png") {
                        let game_name = filename.strip_suffix(".png").unwrap_or(filename);
                        icons.push(game_name.to_string());
                    }
                }
            }
        }
    }

    icons.sort();
    Ok(icons)
}
