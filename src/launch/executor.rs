use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use crate::config::game::GameConfig;
use crate::runners::proton::ProtonManager;
use crate::runners::RunnerManager;
use crate::utils::fs::CellarDirectories;

use super::command::{CommandBuilder, LaunchCommand};

/// Handles the execution of games with proper Proton integration
pub struct GameLauncher {
    dirs: CellarDirectories,
}

impl GameLauncher {
    pub fn new() -> Result<Self> {
        let dirs = CellarDirectories::new()?;
        Ok(Self { dirs })
    }

    /// Launch a game using its configuration
    pub async fn launch_game(&self, game_config: &GameConfig) -> Result<()> {
        println!("Launching game: {}", game_config.game.name);
        println!("  Executable: {}", game_config.game.executable.display());
        println!("  Wine Prefix: {}", game_config.game.wine_prefix.display());
        println!("  Proton Version: {}", game_config.game.proton_version);

        // Validate the configuration before launching
        self.validate_launch_config(game_config)?;

        // Find the Proton installation
        let proton_path = self
            .find_proton_installation(&game_config.game.proton_version)
            .await?;
        println!("  Proton Path: {}", proton_path.display());

        // Build the launch command
        let launch_command = CommandBuilder::new(game_config.clone())
            .with_proton_path(proton_path)
            .build()?;

        // Execute the command
        self.execute_launch_command(&launch_command).await?;

        println!("Game exited.");
        Ok(())
    }

    /// Validate that the game configuration is ready for launching
    fn validate_launch_config(&self, config: &GameConfig) -> Result<()> {
        // Check if executable exists
        if !config.game.executable.exists() {
            return Err(anyhow!(
                "Game executable not found: {}",
                config.game.executable.display()
            ));
        }

        // Check if wine prefix exists
        if !config.game.wine_prefix.exists() {
            return Err(anyhow!(
                "Wine prefix not found: {}. Create it first with 'cellar prefix create'",
                config.game.wine_prefix.display()
            ));
        }

        // Validate wine prefix structure
        let system32_path = config.game.wine_prefix.join("drive_c/windows/system32");
        if !system32_path.exists() {
            return Err(anyhow!(
                "Wine prefix appears to be incomplete: {}",
                config.game.wine_prefix.display()
            ));
        }

        // Check if this is a Proton prefix if we're using Proton
        let version_file = config.game.wine_prefix.join("version");
        if !version_file.exists() {
            println!("⚠ Warning: No Proton version file found in prefix. This may not be a Proton-compatible prefix.");
            println!("  Consider creating a new Proton prefix with: cellar prefix create <name> --proton {}", config.game.proton_version);
        }

        Ok(())
    }

    /// Find the Proton installation path
    async fn find_proton_installation(&self, proton_version: &str) -> Result<PathBuf> {
        let runners_path = self.dirs.get_runners_path();
        let proton_manager = ProtonManager::new(runners_path);

        let runners = proton_manager.discover_local_runners().await?;
        let proton_runner = runners
            .iter()
            .find(|r| r.version == proton_version || r.name.contains(proton_version))
            .ok_or_else(|| {
                anyhow!(
                    "Proton version '{}' not found. Install it first with 'cellar runners install proton {}'",
                    proton_version, proton_version
                )
            })?;

        Ok(proton_runner.path.clone())
    }

    /// Execute the launch command with proper environment and error handling
    async fn execute_launch_command(&self, launch_command: &LaunchCommand) -> Result<()> {
        let args = launch_command.command.as_args();

        // Check if the first argument looks like an environment variable assignment
        let needs_shell = args.first().map(|arg| arg.contains('=')).unwrap_or(false);

        if needs_shell {
            // Use shell execution for complex command lines with environment variables
            self.execute_shell_command(launch_command).await
        } else {
            // Direct execution for simple commands
            self.execute_direct_command(launch_command).await
        }
    }

    /// Execute command directly without shell
    async fn execute_direct_command(&self, launch_command: &LaunchCommand) -> Result<()> {
        let program = launch_command.command.program();
        let cmd_args = launch_command.command.args();

        println!("Executing command:");
        println!("  Program: {}", program);
        if !cmd_args.is_empty() {
            println!("  Arguments: {}", cmd_args.join(" "));
        }

        // Print environment variables that are game-specific (filter out system ones)
        self.print_environment_variables(&launch_command.environment);

        println!("\nStarting game...");

        let mut command = Command::new(program);
        command
            .args(cmd_args)
            .envs(&launch_command.environment)
            .current_dir(&launch_command.working_directory)
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped());

        let child = command.spawn()?;
        self.handle_command_output(child).await
    }

    /// Execute command through shell for complex command lines
    async fn execute_shell_command(&self, launch_command: &LaunchCommand) -> Result<()> {
        let args = launch_command.command.as_args();
        let command_line = self.shell_quote_command(args);

        println!("Executing shell command:");
        println!("  Command: {}", command_line);

        // Print environment variables that are game-specific (filter out system ones)
        self.print_environment_variables(&launch_command.environment);

        println!("\nStarting game...");

        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(&command_line)
            .envs(&launch_command.environment)
            .current_dir(&launch_command.working_directory)
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped());

        let child = command.spawn()?;
        self.handle_command_output(child).await
    }

    /// Print interesting environment variables
    fn print_environment_variables(&self, environment: &std::collections::HashMap<String, String>) {
        let interesting_env_vars: Vec<_> = environment
            .iter()
            .filter(|(key, _)| {
                key.starts_with("WINE")
                    || key.starts_with("PROTON")
                    || key.starts_with("DXVK")
                    || key.starts_with("GAMEID")
                    || key.starts_with("HOST_LC_ALL")
            })
            .collect();

        if !interesting_env_vars.is_empty() {
            println!("  Environment variables:");
            for (key, value) in interesting_env_vars {
                println!("    {}={}", key, value);
            }
        }
    }

    /// Properly quote shell command arguments that contain spaces or special characters
    fn shell_quote_command(&self, args: &[String]) -> String {
        args.iter()
            .map(|arg| {
                if arg.contains(' ')
                    || arg.contains('"')
                    || arg.contains('\'')
                    || arg.contains('\\')
                {
                    // Escape any existing double quotes and wrap in double quotes
                    format!("\"{}\"", arg.replace('\"', "\\\""))
                } else {
                    arg.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Handle command output and error filtering
    async fn handle_command_output(&self, child: tokio::process::Child) -> Result<()> {
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
                    "Game launch failed with errors:\n{}",
                    critical_errors.join("\n")
                ));
            } else {
                println!("Game exited with non-zero status but no critical errors detected.");
            }
        }

        Ok(())
    }

    /// Launch a game by name (convenience method)
    pub async fn launch_game_by_name(&self, game_name: &str) -> Result<()> {
        let config_path = self.dirs.get_game_config_path(game_name);

        if !config_path.exists() {
            return Err(anyhow!("Game '{}' not found", game_name));
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("Failed to read game config: {}", e))?;

        let config: GameConfig =
            toml::from_str(&content).map_err(|e| anyhow!("Failed to parse game config: {}", e))?;

        self.launch_game(&config).await
    }
}

impl Default for GameLauncher {
    fn default() -> Self {
        Self::new().expect("Failed to create GameLauncher")
    }
}
