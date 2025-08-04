use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use crate::utils::fs::CellarDirectories;

/// Extracts the icon from an executable file and converts it to PNG format.
///
/// This function uses `wrestool` to extract the icon resource from the specified executable,
/// then converts the extracted ICO file to PNG format using ImageMagick's `magick` command.
/// The resulting PNG icon is stored in the application's icon directory, and the intermediate
/// ICO file is removed after conversion.
///
/// # Arguments
///
/// * `exe_path` - Path to the executable file from which to extract the icon.
/// * `game_name` - Name used to identify and store the icon files.
///
/// # Returns
///
/// Returns the path to the generated PNG icon file on success.
///
/// # Errors
///
/// Returns an error if required tools are missing, extraction or conversion fails, or file operations fail.
///
/// # Examples
///
/// ```
/// let exe_path = std::path::Path::new("/path/to/game.exe");
/// let png_icon = extract_and_convert_icon(exe_path, "my_game").await?;
/// assert!(png_icon.ends_with("my_game.png"));
/// ```
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

/// Asynchronously verifies that the `wrestool` and `magick` commands are available in the system path.
///
/// Returns an error if either tool is not found.
///
/// # Examples
///
/// ```
/// tokio_test::block_on(async {
///     check_required_tools().expect("Required tools should be installed");
/// });
/// ```
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

/// Extracts the icon resource from an executable file using `wrestool`.
///
/// Attempts to extract the icon (resource type 14) from the specified executable and writes it to the given output path. Returns an error if extraction fails or if no icon is found.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # tokio_test::block_on(async {
/// let exe_path = Path::new("game.exe");
/// let output_path = Path::new("icon.ico");
/// let result = extract_icon_with_wrestool(exe_path, output_path).await;
/// assert!(result.is_ok() || result.is_err());
/// # });
/// ```
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

/// Converts an ICO file to PNG format using ImageMagick.
///
/// Uses the highest resolution icon from the ICO file for conversion.
///
/// # Arguments
///
/// * `ico_path` - Path to the source ICO file.
/// * `png_path` - Path where the resulting PNG file will be saved.
///
/// # Errors
///
/// Returns an error if the conversion fails or if ImageMagick is not available.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # tokio_test::block_on(async {
/// let ico = Path::new("icon.ico");
/// let png = Path::new("icon.png");
/// convert_ico_to_png(ico, png).await.unwrap();
/// assert!(png.exists());
/// # });
/// ```
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

/// Returns the PNG icon path for a game, extracting it from the executable if not already present.
///
/// If the PNG icon for the specified game exists, returns its path. Otherwise, attempts to extract and convert the icon from the provided executable. If extraction fails, returns `None`.
///
/// # Arguments
///
/// * `exe_path` - Path to the game's executable file.
/// * `game_name` - Name of the game, used to determine icon file naming.
///
/// # Returns
///
/// An `Option<PathBuf>` containing the path to the PNG icon if available or successfully extracted, or `None` if extraction fails.
///
/// # Examples
///
/// ```
/// let icon_path = get_or_extract_icon(Path::new("game.exe"), "MyGame").await?;
/// if let Some(path) = icon_path {
///     // Use the icon at `path`
/// } else {
///     // Fallback to a default icon
/// }
/// ```
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

/// Deletes both ICO and PNG icon files associated with the specified game, if they exist.
///
/// # Examples
///
/// ```
/// remove_game_icons("ExampleGame")?;
/// ```
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

/// Returns a sorted list of game names for which PNG icons have been extracted.
///
/// Scans the icons directory for `.png` files and extracts the corresponding game names by removing the file extension.
///
/// # Returns
/// A sorted vector of game names as strings.
///
/// # Examples
///
/// ```
/// let icons = list_game_icons().unwrap();
/// assert!(icons.contains(&"my_game".to_string()));
/// ```
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
