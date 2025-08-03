use anyhow::{anyhow, Result};
use clap::Subcommand;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::game::{
    DesktopConfig, GameConfig, GameInfo, GamescopeConfig, LaunchConfig, MangohudConfig, WineConfig,
};
use crate::config::validation::validate_game_config;
use crate::runners::dxvk::DxvkManager;
use crate::runners::proton::ProtonManager;
use crate::runners::{RunnerCache, RunnerManager, RunnerType};
use crate::utils::fs::{sanitize_filename, CellarDirectories};

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new game
    Add {
        /// Name of the game
        name: String,
        /// Path to existing executable
        #[arg(long)]
        exe: Option<String>,
        /// Path to installer executable
        #[arg(long)]
        installer: Option<String>,
        /// Interactive setup
        #[arg(short, long)]
        interactive: bool,
        /// Proton version to use for the game
        #[arg(long)]
        proton: Option<String>,
        /// Prefix name to use (defaults to game name)
        #[arg(long)]
        prefix: Option<String>,
    },
    /// Launch a game
    Launch {
        /// Name of the game to launch
        name: String,
    },
    /// List all games
    List,
    /// Remove a game
    Remove {
        /// Name of the game to remove
        name: String,
    },
    /// Show game information
    Info {
        /// Name of the game
        name: String,
    },
    /// List games
    Status {
        /// Name of the game (optional, shows all if not provided)
        name: Option<String>,
    },
    /// Runner management commands
    Runners {
        #[command(subcommand)]
        command: RunnerCommands,
    },
    /// Prefix management commands
    Prefix {
        #[command(subcommand)]
        command: PrefixCommands,
    },
}

#[derive(Subcommand)]
pub enum RunnerCommands {
    /// List installed runners
    List,
    /// Refresh runner cache
    Refresh,
    /// Show available runners for download
    Available,
    /// Install a runner
    Install {
        /// Runner type (proton, dxvk)
        runner_type: String,
        /// Version to install
        version: String,
    },
    /// Install DXVK into a prefix
    InstallDxvk {
        /// DXVK version to install
        version: String,
        /// Prefix name to install into
        prefix: String,
    },
    /// Remove/uninstall a runner
    Remove {
        /// Runner type (proton, dxvk)
        runner_type: String,
        /// Version to remove
        version: String,
    },
}

#[derive(Subcommand)]
pub enum PrefixCommands {
    /// Create a new wine prefix
    Create {
        /// Name of the prefix
        name: String,
        /// Proton version to use
        #[arg(long)]
        proton: Option<String>,
    },
    /// List all prefixes
    List,
    /// Remove a prefix
    Remove {
        /// Name of the prefix to remove
        name: String,
    },
    /// Run executable in prefix
    Run {
        /// Name of the prefix
        prefix: String,
        /// Path to executable
        exe: String,
        /// Proton version to use (optional, autodetects if not provided)
        #[arg(long)]
        proton: Option<String>,
    },
}

pub async fn add_game(
    name: String,
    exe: Option<String>,
    installer: Option<String>,
    interactive: bool,
    proton: Option<String>,
    prefix: Option<String>,
) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    dirs.ensure_all_exist()?;

    if interactive {
        println!("Interactive mode not yet implemented. Using basic mode.");
    }

    if installer.is_some() {
        return Err(anyhow!("Installer mode not yet implemented in Phase 1"));
    }

    let exe_path =
        exe.ok_or_else(|| anyhow!("Executable path is required for basic game addition"))?;
    let exe_path = crate::utils::fs::expand_tilde(exe_path)?;

    if !exe_path.exists() {
        return Err(anyhow!("Executable does not exist: {}", exe_path.display()));
    }

    if !exe_path.is_file() {
        return Err(anyhow!("Path is not a file: {}", exe_path.display()));
    }

    if name.trim().is_empty() {
        return Err(anyhow!("Game name cannot be empty"));
    }

    let config =
        create_basic_game_config(&name, exe_path, &dirs, proton.as_deref(), prefix.as_deref())
            .await?;
    save_game_config(&dirs, &name, &config)?;

    println!("Successfully added game: {name}");
    println!(
        "  Config saved to: {}",
        dirs.get_game_config_path(&name).display()
    );

    Ok(())
}

pub async fn launch_game(name: String) -> Result<()> {
    let launcher = crate::launch::GameLauncher::new()?;
    launcher.launch_game_by_name(&name).await
}

pub fn list_games(name: Option<String>) -> Result<()> {
    let dirs = CellarDirectories::new()?;

    match name {
        Some(game_name) => {
            let config = load_game_config(&dirs, &game_name)?;
            println!("Game: {}", config.game.name);
        }
        None => {
            let games = dirs.list_game_configs()?;

            if games.is_empty() {
                println!("No games configured.");
                return Ok(());
            }

            println!("Configured games:");
            for game_name in &games {
                match load_game_config(&dirs, game_name) {
                    Ok(config) => {
                        println!("  {}", config.game.name);
                        println!("    Executable: {}", config.game.executable.display());
                        println!("    Proton: {}", config.game.proton_version);
                    }
                    Err(_) => {
                        println!("  {game_name} [error loading config]");
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn remove_game(name: String) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let config_path = dirs.get_game_config_path(&name);

    if !config_path.exists() {
        return Err(anyhow!("Game '{}' not found", name));
    }

    fs::remove_file(&config_path).map_err(|e| anyhow!("Failed to remove config file: {}", e))?;

    println!("Successfully removed game: {name}");

    Ok(())
}

pub fn show_game_info(name: String) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let config = load_game_config(&dirs, &name)?;

    println!("Game Information for: {}", config.game.name);
    println!("  Executable: {}", config.game.executable.display());
    println!("  Wine Prefix: {}", config.game.wine_prefix.display());
    println!("  Proton Version: {}", config.game.proton_version);

    if let Some(dxvk_version) = &config.game.dxvk_version {
        println!("  DXVK Version: {dxvk_version}");
    }

    println!("\nWine Configuration:");
    println!("  esync: {}", config.wine_config.esync);
    println!("  fsync: {}", config.wine_config.fsync);
    println!("  dxvk: {}", config.wine_config.dxvk);
    println!("  dxvk_async: {}", config.wine_config.dxvk_async);

    if config.gamescope.enabled {
        println!("\nGamescope Configuration:");
        println!(
            "  Game Resolution: {}x{}",
            config.gamescope.width, config.gamescope.height
        );
        println!(
            "  Output Resolution: {}x{}",
            config.gamescope.output_width, config.gamescope.output_height
        );
        println!("  Refresh Rate: {}Hz", config.gamescope.refresh_rate);
        println!("  Upscaling: {}", config.gamescope.upscaling);
    }

    Ok(())
}

async fn create_basic_game_config(
    name: &str,
    exe_path: PathBuf,
    dirs: &CellarDirectories,
    proton_version: Option<&str>,
    prefix_name: Option<&str>,
) -> Result<GameConfig> {
    // Determine prefix name: use provided or default to game name
    let prefix_name = match prefix_name {
        Some(provided_prefix) => provided_prefix.to_string(),
        None => sanitize_filename(name), // Only sanitize when using game name as prefix
    };
    let wine_prefix = dirs.get_prefixes_path().join(&prefix_name);

    // Determine Proton version to use BEFORE creating prefix
    let proton_version = match proton_version {
        Some(version) => {
            println!("Using specified Proton version: {version}");

            // Check if the specified version is available locally
            let proton_manager = ProtonManager::new(dirs.get_runners_path());
            let local_runners = proton_manager.discover_local_runners().await?;
            let version_found = local_runners
                .iter()
                .any(|r| r.version == version || r.name.contains(version));

            if !version_found {
                println!("Proton version '{version}' not found locally.");

                // Check if version is available for download
                match check_proton_version_available(&proton_manager, version).await {
                    Ok(download_version) => {
                        // Ask user for permission to download
                        if prompt_user_for_download(version).await? {
                            download_and_install_proton(&proton_manager, &download_version).await?;
                            println!("Successfully installed Proton version: {version}");
                        } else {
                            return Err(anyhow!("Proton version '{}' is required but not available locally. Operation cancelled.", version));
                        }
                    }
                    Err(_) => {
                        return Err(anyhow!(
                            "Proton version '{}' not found locally and not available for download.\nAvailable versions can be seen with: cellar runners available",
                            version
                        ));
                    }
                }
            }

            version.to_string()
        }
        None => {
            println!("No Proton version specified, finding latest available...");
            let latest_version = get_latest_proton_version(dirs).await?;
            println!("Using latest available Proton version: {latest_version}");
            latest_version
        }
    };

    // Check if prefix exists, if not create it
    if !wine_prefix.exists() {
        create_prefix(&prefix_name, Some(&proton_version)).await?;
    } else {
        println!("Using existing prefix: {prefix_name}");
    }

    let config = GameConfig {
        game: GameInfo {
            name: name.to_string(),
            executable: exe_path,
            wine_prefix,
            proton_version,
            dxvk_version: None,
        },
        launch: LaunchConfig::default(),
        wine_config: WineConfig::default(),
        dxvk: Default::default(),
        gamescope: GamescopeConfig::default(),
        mangohud: MangohudConfig::default(),
        desktop: DesktopConfig::default(),
        installation: None,
    };

    validate_game_config(&config)?;
    Ok(config)
}

/// Get the latest available Proton version from cache, or discover if cache is missing/old
async fn get_latest_proton_version(dirs: &CellarDirectories) -> Result<String> {
    let cache_path = dirs.get_cache_path().join("runners.toml");

    // Try to load from cache first
    let mut proton_runners = Vec::new();

    if cache_path.exists() {
        if let Ok(cache_content) = fs::read_to_string(&cache_path) {
            if let Ok(cache) = toml::from_str::<RunnerCache>(&cache_content) {
                // Check if cache is recent (less than 1 hour old)
                let cache_age = chrono::Utc::now().signed_duration_since(cache.last_updated);
                if cache_age.num_hours() < 1 {
                    proton_runners = cache
                        .runners
                        .into_iter()
                        .filter(|r| matches!(r.runner_type, RunnerType::Proton))
                        .collect();
                }
            }
        }
    }

    // If cache is empty or old, discover live
    if proton_runners.is_empty() {
        let runners_path = dirs.get_runners_path();
        let proton_manager = ProtonManager::new(runners_path);
        proton_runners = proton_manager.discover_local_runners().await?;
    }

    if proton_runners.is_empty() {
        return Err(anyhow!(
            "No Proton versions found. Please install a Proton version first using:\n  cellar runners install proton <version>\n\nTo see available versions for download, use:\n  cellar runners available"
        ));
    }

    // Sort versions to get the latest (assuming semantic versioning-like names)
    proton_runners.sort_by(|a, b| {
        // Extract version numbers for comparison (e.g., "GE-Proton9-1" -> "9.1")
        let version_a = extract_version_number(&a.version);
        let version_b = extract_version_number(&b.version);
        version_b
            .partial_cmp(&version_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(proton_runners[0].version.clone())
}

/// Check if a Proton version is available for download
async fn check_proton_version_available(
    proton_manager: &ProtonManager,
    version: &str,
) -> Result<String> {
    let available_versions = proton_manager.get_available_versions().await?;

    // Try exact match first
    if available_versions.iter().any(|v| v == version) {
        return Ok(version.to_string());
    }

    // Try partial match (e.g., user provides "9-1" for "GE-Proton9-1")
    if let Some(found) = available_versions
        .iter()
        .find(|v| v.contains(version) || v.ends_with(&format!("-{version}")))
    {
        return Ok(found.clone());
    }

    Err(anyhow!(
        "Version '{}' not found in available releases",
        version
    ))
}

/// Prompt user for permission to download a Proton version
async fn prompt_user_for_download(version: &str) -> Result<bool> {
    use std::io::{self, Write};

    print!("Download Proton version '{version}'? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(input == "y" || input == "yes")
}

/// Download and install a Proton version
async fn download_and_install_proton(proton_manager: &ProtonManager, version: &str) -> Result<()> {
    println!("Downloading Proton version: {version}");

    // Extract the actual version number from the full version string
    // e.g., "GE-Proton10-10" -> "10-10"
    let version_number = if version.starts_with("GE-Proton") {
        version.strip_prefix("GE-Proton").unwrap_or(version)
    } else {
        version
    };

    let download_path = proton_manager
        .download_runner("proton-ge", version_number)
        .await?;
    println!("Installing Proton version: {version}");

    proton_manager
        .install_runner(&download_path, std::path::Path::new(""))
        .await?;

    // Refresh runner cache after installation
    let dirs = CellarDirectories::new()?;
    refresh_runners_cache(&dirs).await?;

    Ok(())
}

/// Refresh runner cache without printing messages
async fn refresh_runners_cache(dirs: &CellarDirectories) -> Result<()> {
    let cache_path = dirs.get_cache_path().join("runners.toml");

    // Remove existing cache
    if cache_path.exists() {
        std::fs::remove_file(&cache_path)?;
    }

    // Discover all runners and cache them
    let runners_path = dirs.get_runners_path();
    let proton_manager = ProtonManager::new(runners_path.clone());
    let dxvk_manager = DxvkManager::new(runners_path);

    let mut all_runners = Vec::new();
    all_runners.extend(proton_manager.discover_local_runners().await?);
    all_runners.extend(dxvk_manager.discover_local_runners().await?);

    // Save to cache
    let cache = crate::runners::RunnerCache {
        runners: all_runners,
        last_updated: chrono::Utc::now(),
    };

    let cache_content = toml::to_string_pretty(&cache)?;
    std::fs::write(&cache_path, cache_content)?;

    Ok(())
}

/// Extract version number from Proton version string for comparison
/// E.g., "GE-Proton9-1" -> 9.1, "GE-Proton10-10" -> 10.10
pub fn extract_version_number(version: &str) -> f64 {
    // Try to extract major.minor version from patterns like "GE-Proton9-1"
    if let Some(captures) = regex::Regex::new(r"GE-Proton(\d+)-(\d+)")
        .unwrap()
        .captures(version)
    {
        let major: u32 = captures[1].parse().unwrap_or(0);
        let minor: u32 = captures[2].parse().unwrap_or(0);
        return major as f64 + (minor as f64 / 100.0);
    }

    // Fallback: try to extract any number from the version string
    if let Some(captures) = regex::Regex::new(r"(\d+)").unwrap().captures(version) {
        return captures[1].parse::<f64>().unwrap_or(0.0);
    }

    0.0
}

fn save_game_config(dirs: &CellarDirectories, name: &str, config: &GameConfig) -> Result<()> {
    let config_path = dirs.get_game_config_path(name);
    let toml_content =
        toml::to_string_pretty(config).map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, toml_content)
        .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

    Ok(())
}

fn load_game_config(dirs: &CellarDirectories, name: &str) -> Result<GameConfig> {
    let config_path = dirs.get_game_config_path(name);

    if !config_path.exists() {
        return Err(anyhow!("Game '{}' not found", name));
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| anyhow!("Failed to read config file: {}", e))?;

    let config: GameConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse config file: {}", e))?;

    Ok(config)
}

// Runner management functions
pub async fn handle_runners_command(command: RunnerCommands) -> Result<()> {
    match command {
        RunnerCommands::List => list_runners().await,
        RunnerCommands::Refresh => refresh_runners().await,
        RunnerCommands::Available => show_available_runners().await,
        RunnerCommands::Install {
            runner_type,
            version,
        } => install_runner(&runner_type, &version).await,
        RunnerCommands::InstallDxvk { version, prefix } => {
            install_dxvk_to_prefix(&version, &prefix).await
        }
        RunnerCommands::Remove {
            runner_type,
            version,
        } => remove_runner(&runner_type, &version).await,
    }
}

async fn list_runners() -> Result<()> {
    let dirs = CellarDirectories::new()?;
    dirs.ensure_all_exist()?; // Ensure all directories exist
    let cache_path = dirs.get_cache_path().join("runners.toml");

    // Try to load from cache first
    if cache_path.exists() {
        if let Ok(cache_content) = fs::read_to_string(&cache_path) {
            if let Ok(cache) = toml::from_str::<crate::runners::RunnerCache>(&cache_content) {
                // Check if cache is recent (less than 1 hour old)
                let cache_age = chrono::Utc::now().signed_duration_since(cache.last_updated);
                if cache_age.num_hours() < 1 {
                    println!("Installed Runners (cached):");

                    let proton_runners: Vec<_> = cache
                        .runners
                        .iter()
                        .filter(|r| matches!(r.runner_type, crate::runners::RunnerType::Proton))
                        .collect();

                    let dxvk_runners: Vec<_> = cache
                        .runners
                        .iter()
                        .filter(|r| matches!(r.runner_type, crate::runners::RunnerType::Dxvk))
                        .collect();

                    if !proton_runners.is_empty() {
                        println!("\nProton Runners:");
                        for runner in &proton_runners {
                            println!("  {} ({})", runner.name, runner.version);
                            println!("    Path: {}", runner.path.display());
                        }
                    }

                    if !dxvk_runners.is_empty() {
                        println!("\nDXVK Runners:");
                        for runner in &dxvk_runners {
                            println!("  {} ({})", runner.name, runner.version);
                            println!("    Path: {}", runner.path.display());
                        }
                    }

                    if proton_runners.is_empty() && dxvk_runners.is_empty() {
                        println!(
                            "  No runners found. Use 'cellar runners install' to install runners."
                        );
                    }

                    return Ok(());
                }
            }
        }
    }

    // Cache is old or doesn't exist, scan live
    let runners_path = dirs.get_runners_path();

    let proton_manager = ProtonManager::new(runners_path.clone());
    let dxvk_manager = DxvkManager::new(runners_path);

    println!("Installed Runners:");

    // List Proton runners
    let proton_runners = proton_manager.discover_local_runners().await?;
    if !proton_runners.is_empty() {
        println!("\nProton Runners:");
        for runner in &proton_runners {
            println!("  {} ({})", runner.name, runner.version);
            println!("    Path: {}", runner.path.display());
        }
    }

    // List DXVK runners
    let dxvk_runners = dxvk_manager.discover_local_runners().await?;
    if !dxvk_runners.is_empty() {
        println!("\nDXVK Runners:");
        for runner in &dxvk_runners {
            println!("  {} ({})", runner.name, runner.version);
            println!("    Path: {}", runner.path.display());
        }
    }

    if proton_runners.is_empty() && dxvk_runners.is_empty() {
        println!("  No runners found. Use 'cellar runners install' to install runners.");
    }

    Ok(())
}

async fn refresh_runners() -> Result<()> {
    let dirs = CellarDirectories::new()?;
    dirs.ensure_all_exist()?; // Ensure all directories exist including cache
    let cache_path = dirs.get_cache_path().join("runners.toml");

    // Remove existing cache
    if cache_path.exists() {
        fs::remove_file(&cache_path)?;
    }

    println!("Refreshing runner cache...");

    // Discover all runners and cache them
    let runners_path = dirs.get_runners_path();
    let proton_manager = ProtonManager::new(runners_path.clone());
    let dxvk_manager = DxvkManager::new(runners_path);

    let mut all_runners = Vec::new();
    all_runners.extend(proton_manager.discover_local_runners().await?);
    all_runners.extend(dxvk_manager.discover_local_runners().await?);

    // Save to cache
    let cache = crate::runners::RunnerCache {
        runners: all_runners,
        last_updated: chrono::Utc::now(),
    };

    let cache_content = toml::to_string_pretty(&cache)?;
    fs::write(&cache_path, cache_content)?;

    println!(
        "Runner cache refreshed with {} runners.",
        cache.runners.len()
    );

    Ok(())
}

async fn show_available_runners() -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let runners_path = dirs.get_runners_path();

    println!("Fetching available runners...");

    // Get available Proton versions
    let proton_manager = ProtonManager::new(runners_path.clone());
    match proton_manager.get_available_versions().await {
        Ok(versions) => {
            println!("\nAvailable Proton-GE versions:");
            for version in versions.iter().take(10) {
                // Show first 10
                println!("  {version}");
            }
            if versions.len() > 10 {
                println!("  ... and {} more", versions.len() - 10);
            }
        }
        Err(e) => println!("Failed to fetch Proton versions: {e}"),
    }

    // Get available DXVK versions
    let dxvk_manager = DxvkManager::new(runners_path);
    match dxvk_manager.get_available_versions().await {
        Ok(versions) => {
            println!("\nAvailable DXVK versions:");
            for version in versions.iter().take(10) {
                // Show first 10
                println!("  {version}");
            }
            if versions.len() > 10 {
                println!("  ... and {} more", versions.len() - 10);
            }
        }
        Err(e) => println!("Failed to fetch DXVK versions: {e}"),
    }

    Ok(())
}

async fn install_runner(runner_type: &str, version: &str) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let runners_path = dirs.get_runners_path();

    match runner_type.to_lowercase().as_str() {
        "proton" => {
            println!("Installing Proton-GE {version}...");
            let proton_manager = ProtonManager::new(runners_path);

            // Extract the actual version number from the full version string
            // e.g., "GE-Proton10-10" -> "10-10"
            let version_number = if version.starts_with("GE-Proton") {
                version.strip_prefix("GE-Proton").unwrap_or(version)
            } else {
                version
            };

            let download_path = proton_manager.download_runner("proton-ge", version_number).await?;
            proton_manager
                .install_runner(&download_path, Path::new(""))
                .await?;

            println!("Successfully installed Proton-GE {version}");
        }
        "dxvk" => {
            println!("Installing DXVK {version}...");
            let dxvk_manager = DxvkManager::new(runners_path);

            let download_path = dxvk_manager.download_runner("dxvk", version).await?;
            dxvk_manager
                .install_runner(&download_path, Path::new(""))
                .await?;

            println!("Successfully installed DXVK {version}");
        }
        _ => {
            return Err(anyhow!(
                "Unsupported runner type: {}. Supported types: proton, dxvk",
                runner_type
            ));
        }
    }

    // Refresh cache after installation
    refresh_runners().await?;

    Ok(())
}

async fn remove_runner(runner_type: &str, version: &str) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let runners_path = dirs.get_runners_path();

    match runner_type.to_lowercase().as_str() {
        "proton" => {
            println!("Removing Proton-GE {version}...");
            let proton_manager = ProtonManager::new(runners_path);

            let runners = proton_manager.discover_local_runners().await?;
            let runner = runners
                .iter()
                .find(|r| r.version == version || r.name.contains(version))
                .ok_or_else(|| anyhow!("Proton version '{}' not found", version))?;

            proton_manager.delete_runner(&runner.path).await?;
            println!("Successfully removed Proton-GE {version}");
        }
        "dxvk" => {
            println!("Removing DXVK {version}...");
            let dxvk_manager = DxvkManager::new(runners_path);

            let runners = dxvk_manager.discover_local_runners().await?;
            let runner = runners
                .iter()
                .find(|r| r.version == version || r.name.contains(version))
                .ok_or_else(|| anyhow!("DXVK version '{}' not found", version))?;

            dxvk_manager.delete_runner(&runner.path).await?;
            println!("Successfully removed DXVK {version}");
        }
        _ => {
            return Err(anyhow!(
                "Unsupported runner type: {}. Supported types: proton, dxvk",
                runner_type
            ));
        }
    }

    // Refresh cache after removal
    refresh_runners().await?;

    Ok(())
}

// Prefix management functions
pub async fn handle_prefix_command(command: PrefixCommands) -> Result<()> {
    match command {
        PrefixCommands::Create { name, proton } => create_prefix(&name, proton.as_deref()).await,
        PrefixCommands::List => list_prefixes().await,
        PrefixCommands::Remove { name } => remove_prefix(&name).await,
        PrefixCommands::Run {
            prefix,
            exe,
            proton,
        } => run_in_prefix(&prefix, &exe, proton.as_deref()).await,
    }
}

async fn create_prefix(name: &str, proton_version: Option<&str>) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let prefix_path = dirs.get_prefixes_path().join(name);

    if prefix_path.exists() {
        return Err(anyhow!("Prefix '{}' already exists", name));
    }

    println!("Creating wine prefix: {name}");

    if let Some(proton) = proton_version {
        // Create Proton prefix using umu
        println!("Using Proton version: {proton}");

        let runners_path = dirs.get_runners_path();
        let proton_manager = ProtonManager::new(runners_path);

        // Find the Proton installation
        let runners = proton_manager.discover_local_runners().await?;
        let proton_runner = runners.iter()
            .find(|r| r.version == proton || r.name.contains(proton))
            .ok_or_else(|| anyhow!("Proton version '{}' not found. Install it first with 'cellar runners install proton {}'", proton, proton))?;

        println!("Initializing prefix...");

        // Set up cache directories for Wine Mono and Gecko like Lutris does
        let mono_cache = proton_runner.path.join("files/share/wine/mono");
        let gecko_cache = proton_runner.path.join("files/share/wine/gecko");

        let output = tokio::process::Command::new("umu-run")
            .env("WINEARCH", "win64")
            .env("WINEPREFIX", &prefix_path)
            .env("WINEDLLOVERRIDES", "")
            .env("WINE_MONO_CACHE_DIR", &mono_cache)
            .env("WINE_GECKO_CACHE_DIR", &gecko_cache)
            .env("PROTON_VERB", "run")
            .env("PROTONPATH", &proton_runner.path)
            .env("GAMEID", "umu-default")
            .arg("createprefix")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Check if prefix was actually created despite non-zero exit code
            let system32_path = prefix_path.join("drive_c/windows/system32");
            let version_file = prefix_path.join("version");

            if system32_path.exists() && version_file.exists() {
                // Prefix was created successfully despite umu-run's exit code
                // This is common with umu-run's verbose output
                println!("Prefix created successfully.");
            } else {
                // Filter out common umu-run informational messages
                let critical_errors: Vec<&str> = stderr
                    .lines()
                    .filter(|line| {
                        let line_lower = line.to_lowercase();
                        line_lower.contains("error")
                            && !line.contains("INFO:")
                            && !line.contains("WARN:")
                            && !line.contains("Proton:")
                            && !line.contains("ProtonFixes")
                            && !line.contains("fsync:")
                            && !line.trim().is_empty()
                    })
                    .collect();

                if !critical_errors.is_empty() {
                    return Err(anyhow!(
                        "Failed to create Proton prefix: {}",
                        critical_errors.join("\n")
                    ));
                }

                // If no critical errors but prefix wasn't created, show full stderr
                return Err(anyhow!("Failed to create Proton prefix: {}", stderr));
            }
        }

        // Verify the prefix was created successfully
        let system32_path = prefix_path.join("drive_c/windows/system32");
        if !system32_path.exists() {
            return Err(anyhow!(
                "Prefix creation appeared to succeed but system32 directory not found"
            ));
        }

        // Verify the version file was created by UMU
        let version_file = prefix_path.join("version");
        if !version_file.exists() {
            return Err(anyhow!("Prefix creation succeeded but version file not found - may not be a proper Proton prefix"));
        }
    } else {
        // Create basic wine prefix
        fs::create_dir_all(&prefix_path)?;

        println!("Initializing prefix...");
        let output = tokio::process::Command::new("wineboot")
            .env("WINEPREFIX", &prefix_path)
            .env("WINEARCH", "win64")
            .env("WINEDEBUG", "-all") // Suppress all debug output
            .env("WINEFSYNC", "1")
            .env("WINEESYNC", "1")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null()) // Completely suppress stderr during creation
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create wine prefix: {}", stderr));
        }
    }

    println!("Successfully created prefix: {name}");
    println!("  Path: {}", prefix_path.display());

    Ok(())
}

async fn list_prefixes() -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let prefixes_path = dirs.get_prefixes_path();

    if !prefixes_path.exists() {
        println!("No prefixes found.");
        return Ok(());
    }

    println!("Wine Prefixes:");

    let mut entries = fs::read_dir(&prefixes_path)?;
    let mut found_any = false;

    while let Some(entry) = entries.next().transpose()? {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("invalid");

            println!("  {name}");
            println!("    Path: {}", path.display());

            // Check if it's a valid wine prefix
            let system32_path = path.join("drive_c/windows/system32");
            if system32_path.exists() {
                println!("    Status: Valid");
            } else {
                println!("    Status: Incomplete");
            }

            found_any = true;
        }
    }

    if !found_any {
        println!("  No prefixes found.");
    }

    Ok(())
}

async fn remove_prefix(name: &str) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let prefix_path = dirs.get_prefixes_path().join(name);

    if !prefix_path.exists() {
        return Err(anyhow!("Prefix '{}' not found", name));
    }

    println!("Removing prefix: {name}");
    fs::remove_dir_all(&prefix_path)?;
    println!("Successfully removed prefix: {name}");

    Ok(())
}

async fn run_in_prefix(prefix: &str, exe: &str, proton_version: Option<&str>) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let prefix_path = dirs.get_prefixes_path().join(prefix);

    if !prefix_path.exists() {
        return Err(anyhow!("Prefix '{}' not found", prefix));
    }

    let exe_path = crate::utils::fs::expand_tilde(exe)?;
    if !exe_path.exists() {
        return Err(anyhow!("Executable not found: {}", exe));
    }

    println!("Running {exe} in prefix {prefix}");

    if let Some(proton) = proton_version {
        // Run using Proton via umu-run
        println!("Using Proton version: {proton}");

        let runners_path = dirs.get_runners_path();
        let proton_manager = ProtonManager::new(runners_path);

        // Find the Proton installation
        let runners = proton_manager.discover_local_runners().await?;
        let proton_runner = runners.iter()
            .find(|r| r.version == proton || r.name.contains(proton))
            .ok_or_else(|| anyhow!("Proton version '{}' not found. Install it first with 'cellar runners install proton {}'", proton, proton))?;

        let child = tokio::process::Command::new("umu-run")
            .env("WINEARCH", "win64")
            .env("WINEPREFIX", &prefix_path)
            .env("PROTONPATH", &proton_runner.path)
            .env("PROTON_VERB", "waitforexitandrun")
            .env("GAMEID", "umu-default")
            .env("WINE_LARGE_ADDRESS_AWARE", "1")
            .arg(&exe_path)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let output = child.wait_with_output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Filter out Wine debug noise but show critical errors
            let critical_errors: Vec<&str> = stderr
                .lines()
                .filter(|line| {
                    let line_lower = line.to_lowercase();
                    (line_lower.contains("error") || line_lower.contains("failed"))
                        && !line.contains("fixme:")
                        && !line.contains("err:setupapi:create_dest_file")
                        && !line.contains("wine-staging")
                        && !line.contains("experimental patches")
                        && !line.contains("winediag:")
                        && !line_lower.contains("stub")
                        && !line.trim().is_empty()
                })
                .collect();

            if !critical_errors.is_empty() {
                return Err(anyhow!(
                    "Execution failed with errors:\n{}",
                    critical_errors.join("\n")
                ));
            }
        }
    } else {
        // Check if this might be a Proton prefix by looking for version file
        let version_file = prefix_path.join("version");
        if version_file.exists() {
            // Try to auto-detect Proton version from version file
            if let Ok(version_content) = fs::read_to_string(&version_file) {
                let version = version_content.trim();
                if !version.is_empty() {
                    println!("Auto-detected Proton prefix (version: {version})");
                    println!("Using Proton for execution...");

                    let runners_path = dirs.get_runners_path();
                    let proton_manager = ProtonManager::new(runners_path);
                    let runners = proton_manager.discover_local_runners().await?;

                    if let Some(proton_runner) = runners
                        .iter()
                        .find(|r| r.version == version || r.name.contains(version))
                    {
                        let child = tokio::process::Command::new("umu-run")
                            .env("WINEARCH", "win64")
                            .env("WINEPREFIX", &prefix_path)
                            .env("PROTONPATH", &proton_runner.path)
                            .env("PROTON_VERB", "waitforexitandrun")
                            .env("GAMEID", "umu-default")
                            .env("WINE_LARGE_ADDRESS_AWARE", "1")
                            .arg(&exe_path)
                            .stdout(std::process::Stdio::inherit())
                            .stderr(std::process::Stdio::piped())
                            .spawn()?;

                        let output = child.wait_with_output().await?;

                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let critical_errors: Vec<&str> = stderr
                                .lines()
                                .filter(|line| {
                                    let line_lower = line.to_lowercase();
                                    (line_lower.contains("error") || line_lower.contains("failed"))
                                        && !line.contains("fixme:")
                                        && !line.contains("err:setupapi:create_dest_file")
                                        && !line.contains("wine-staging")
                                        && !line.contains("experimental patches")
                                        && !line.contains("winediag:")
                                        && !line_lower.contains("stub")
                                        && !line.trim().is_empty()
                                })
                                .collect();

                            if !critical_errors.is_empty() {
                                return Err(anyhow!(
                                    "Execution failed with errors:\n{}",
                                    critical_errors.join("\n")
                                ));
                            }
                        }

                        println!("Execution completed.");
                        return Ok(());
                    } else {
                        println!(
                            "⚠ Proton version '{version}' not found, falling back to regular Wine"
                        );
                    }
                } else {
                    println!("⚠ Version file exists but is empty or invalid.");
                    println!(
                        "  Consider using: cellar prefix run {prefix} {exe} --proton <version>"
                    );
                }
            }
        }

        // Run using regular Wine
        let child = tokio::process::Command::new("wine")
            .env("WINEPREFIX", &prefix_path)
            .env("WINEDEBUG", "-all,+dll,-setupapi")
            .env("WINEFSYNC", "1")
            .env("WINEESYNC", "1")
            .arg(&exe_path)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let output = child.wait_with_output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let critical_errors: Vec<&str> = stderr
                .lines()
                .filter(|line| {
                    let line_lower = line.to_lowercase();
                    (line_lower.contains("error") || line_lower.contains("failed"))
                        && !line.contains("fixme:")
                        && !line.contains("err:setupapi:create_dest_file")
                        && !line.contains("wine-staging")
                        && !line.contains("experimental patches")
                        && !line.contains("winediag:")
                        && !line_lower.contains("stub")
                        && !line.trim().is_empty()
                })
                .collect();

            if !critical_errors.is_empty() {
                return Err(anyhow!(
                    "Execution failed with errors:\n{}",
                    critical_errors.join("\n")
                ));
            }
        }
    }

    println!("Execution completed.");
    Ok(())
}

async fn install_dxvk_to_prefix(version: &str, prefix_name: &str) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let prefix_path = dirs.get_prefixes_path().join(prefix_name);

    if !prefix_path.exists() {
        return Err(anyhow!("Prefix '{}' not found", prefix_name));
    }

    let runners_path = dirs.get_runners_path();
    let dxvk_manager = DxvkManager::new(runners_path);

    // Find the DXVK installation
    let runners = dxvk_manager.discover_local_runners().await?;
    let dxvk_runner = runners.iter()
        .find(|r| r.version == version || r.name.contains(version))
        .ok_or_else(|| anyhow!("DXVK version '{}' not found. Install it first with 'cellar runners install dxvk {}'", version, version))?;

    println!("Installing DXVK {version} to prefix '{prefix_name}'...");

    // Install DXVK DLLs to the prefix
    dxvk_manager
        .install_dxvk_to_prefix(&dxvk_runner.path, &prefix_path)
        .await?;

    println!("Successfully installed DXVK {version} to prefix '{prefix_name}'");

    Ok(())
}
