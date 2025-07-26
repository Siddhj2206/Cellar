use anyhow::{anyhow, Result};
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

use crate::config::game::{
    DependenciesConfig, DesktopConfig, GameConfig, GameInfo, GamescopeConfig, LaunchConfig,
    MangohudConfig, WineConfig,
};
use crate::config::validation::validate_game_config;
use crate::utils::fs::CellarDirectories;

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
    /// Show game status
    Status {
        /// Name of the game (optional, shows all if not provided)
        name: Option<String>,
    },
}

pub fn add_game(
    name: String,
    exe: Option<String>,
    installer: Option<String>,
    interactive: bool,
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
    let exe_path = PathBuf::from(exe_path);

    if !exe_path.exists() {
        return Err(anyhow!("Executable does not exist: {}", exe_path.display()));
    }

    if !exe_path.is_file() {
        return Err(anyhow!("Path is not a file: {}", exe_path.display()));
    }

    if name.trim().is_empty() {
        return Err(anyhow!("Game name cannot be empty"));
    }

    let config = create_basic_game_config(&name, exe_path, &dirs)?;
    save_game_config(&dirs, &name, &config)?;

    println!("Successfully added game: {}", name);
    println!(
        "  Config saved to: {}",
        dirs.get_game_config_path(&name).display()
    );

    Ok(())
}

pub fn launch_game(name: String) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let config = load_game_config(&dirs, &name)?;

    println!("Launching game: {}", name);
    println!("  Executable: {}", config.game.executable.display());
    println!("  Wine Prefix: {}", config.game.wine_prefix.display());
    println!("  Proton Version: {}", config.game.proton_version);

    // TODO: Implement actual game launching logic
    println!("Note: Actual launching not implemented in Phase 1");

    Ok(())
}

pub fn list_games() -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let games = dirs.list_game_configs()?;

    if games.is_empty() {
        println!("No games configured.");
        return Ok(());
    }

    println!("Configured games:");
    for game_name in games {
        match load_game_config(&dirs, &game_name) {
            Ok(config) => {
                println!("  {} [{}]", config.game.name, config.game.status);
                println!("    Executable: {}", config.game.executable.display());
                println!("    Proton: {}", config.game.proton_version);
            }
            Err(_) => {
                println!("  {} [error loading config]", game_name);
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

    println!("Successfully removed game: {}", name);

    Ok(())
}

pub fn show_game_info(name: String) -> Result<()> {
    let dirs = CellarDirectories::new()?;
    let config = load_game_config(&dirs, &name)?;

    println!("Game Information for: {}", config.game.name);
    println!("  Status: {}", config.game.status);
    println!("  Executable: {}", config.game.executable.display());
    println!("  Wine Prefix: {}", config.game.wine_prefix.display());
    println!("  Proton Version: {}", config.game.proton_version);

    if let Some(dxvk_version) = &config.game.dxvk_version {
        println!("  DXVK Version: {}", dxvk_version);
    }

    if let Some(template) = &config.game.template {
        println!("  Template: {}", template);
    }

    if let Some(preset) = &config.game.preset {
        println!("  Preset: {}", preset);
    }

    println!("\nWine Configuration:");
    println!("  esync: {}", config.wine_config.esync);
    println!("  fsync: {}", config.wine_config.fsync);
    println!("  dxvk: {}", config.wine_config.dxvk);
    println!("  dxvk_async: {}", config.wine_config.dxvk_async);

    if config.gamescope.enabled {
        println!("\nGamescope Configuration:");
        println!(
            "  Resolution: {}x{}",
            config.gamescope.width, config.gamescope.height
        );
        println!("  Refresh Rate: {}Hz", config.gamescope.refresh_rate);
        println!("  Upscaling: {}", config.gamescope.upscaling);
    }

    Ok(())
}

pub fn show_status(name: Option<String>) -> Result<()> {
    let dirs = CellarDirectories::new()?;

    match name {
        Some(game_name) => {
            let config = load_game_config(&dirs, &game_name)?;
            println!("Status for {}: {}", config.game.name, config.game.status);
        }
        None => {
            let games = dirs.list_game_configs()?;
            if games.is_empty() {
                println!("No games configured.");
            } else {
                println!("Game Status Summary:");
                for game_name in games {
                    match load_game_config(&dirs, &game_name) {
                        Ok(config) => {
                            println!("  {}: {}", config.game.name, config.game.status);
                        }
                        Err(_) => {
                            println!("  {}: error", game_name);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn create_basic_game_config(
    name: &str,
    exe_path: PathBuf,
    dirs: &CellarDirectories,
) -> Result<GameConfig> {
    let wine_prefix = dirs.get_game_prefix_path(name);

    let config = GameConfig {
        game: GameInfo {
            name: name.to_string(),
            executable: exe_path,
            wine_prefix,
            proton_version: "GE-Proton8-32".to_string(), // Default version
            dxvk_version: None,
            status: "configured".to_string(),
            template: None,
            preset: None,
        },
        launch: LaunchConfig::default(),
        wine_config: WineConfig::default(),
        dxvk: Default::default(),
        gamescope: GamescopeConfig::default(),
        mangohud: MangohudConfig::default(),
        desktop: DesktopConfig::default(),
        dependencies: DependenciesConfig::default(),
        installation: None,
    };

    validate_game_config(&config)?;
    Ok(config)
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
