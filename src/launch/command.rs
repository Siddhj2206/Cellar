use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::game::GameConfig;

/// Builds launch commands for games with proper environment variable management
pub struct CommandBuilder {
    config: GameConfig,
    proton_path: Option<PathBuf>,
}

impl CommandBuilder {
    pub fn new(config: GameConfig) -> Self {
        Self {
            config,
            proton_path: None,
        }
    }

    pub fn with_proton_path(mut self, proton_path: PathBuf) -> Self {
        self.proton_path = Some(proton_path);
        self
    }

    /// Build the complete launch command with all components
    pub fn build(&self) -> Result<LaunchCommand> {
        // First, build the base umu-run command
        let base_command = self.build_base_command()?;

        // Apply Wine environment variables
        let mut env_vars = self.build_wine_environment()?;

        // Apply DXVK environment variables
        env_vars.extend(self.build_dxvk_environment()?);

        // Process Steam-style launch options with %command% placeholder
        let final_command = self.process_launch_options(base_command, &env_vars)?;

        Ok(LaunchCommand {
            command: final_command,
            environment: env_vars,
            working_directory: self.config.game.wine_prefix.clone(),
        })
    }

    /// Build the base umu-run command that will replace %command%
    fn build_base_command(&self) -> Result<Vec<String>> {
        let _proton_path = self
            .proton_path
            .as_ref()
            .ok_or_else(|| anyhow!("Proton path is required for game launching"))?;

        let mut cmd = vec!["umu-run".to_string()];

        // Add the game executable
        cmd.push(self.config.game.executable.to_string_lossy().to_string());

        // Add game arguments
        cmd.extend(self.config.launch.game_args.iter().cloned());

        Ok(cmd)
    }

    /// Build Wine-specific environment variables based on configuration
    fn build_wine_environment(&self) -> Result<HashMap<String, String>> {
        let mut env = HashMap::new();
        let wine_config = &self.config.wine_config;

        // Base Wine environment for Proton via umu-run
        env.insert("WINEARCH".to_string(), "win64".to_string());
        env.insert(
            "WINEPREFIX".to_string(),
            self.config.game.wine_prefix.to_string_lossy().to_string(),
        );

        if let Some(proton_path) = &self.proton_path {
            env.insert(
                "PROTONPATH".to_string(),
                proton_path.to_string_lossy().to_string(),
            );
        }

        // Essential Proton environment variables
        env.insert("PROTON_VERB".to_string(), "waitforexitandrun".to_string());
        env.insert("GAMEID".to_string(), "umu-default".to_string());
        env.insert("HOST_LC_ALL".to_string(), "en_US.UTF-8".to_string());

        // Wine-specific configurations
        if wine_config.esync {
            env.insert("WINEESYNC".to_string(), "1".to_string());
        }

        if wine_config.fsync {
            env.insert("WINEFSYNC".to_string(), "1".to_string());
        }

        if wine_config.large_address_aware {
            env.insert("WINE_LARGE_ADDRESS_AWARE".to_string(), "1".to_string());
        }

        // DXVK DLL overrides if DXVK is enabled
        if wine_config.dxvk {
            let dll_overrides = "d3d10core,d3d11,d3d9,dxgi=n,b";
            env.insert("WINEDLLOVERRIDES".to_string(), dll_overrides.to_string());
        } else {
            env.insert("WINEDLLOVERRIDES".to_string(), "".to_string());
        }

        Ok(env)
    }

    /// Build DXVK-specific environment variables
    fn build_dxvk_environment(&self) -> Result<HashMap<String, String>> {
        let mut env = HashMap::new();

        if self.config.wine_config.dxvk {
            // DXVK HUD configuration
            if !self.config.dxvk.hud.is_empty() {
                env.insert("DXVK_HUD".to_string(), self.config.dxvk.hud.clone());
            } else {
                env.insert("DXVK_HUD".to_string(), "0".to_string());
            }

            // DXVK async configuration
            if self.config.wine_config.dxvk_async {
                env.insert("DXVK_ASYNC".to_string(), "1".to_string());
            }

            // DXVK state cache path (managed automatically by Cellar)
            let cache_path = self.config.game.wine_prefix.join("dxvk_cache");
            env.insert(
                "DXVK_STATE_CACHE_PATH".to_string(),
                cache_path.to_string_lossy().to_string(),
            );
        }

        Ok(env)
    }

    /// Process Steam-style launch options with %command% placeholder
    fn process_launch_options(
        &self,
        base_command: Vec<String>,
        _env_vars: &HashMap<String, String>,
    ) -> Result<Vec<String>> {
        let launch_options = &self.config.launch.launch_options;

        if launch_options.is_empty() {
            // No launch options, wrap with mangohud first, then gamescope, then gamemode
            let mangohud_wrapped = self.wrap_with_mangohud(base_command)?;
            let gamescope_wrapped = self.wrap_with_gamescope(mangohud_wrapped)?;
            let gamemode_wrapped = self.wrap_with_gamemode(gamescope_wrapped)?;
            return Ok(gamemode_wrapped);
        }

        // Parse launch options into tokens
        let tokens = self.parse_launch_options(launch_options)?;

        // Find and replace %command% placeholder
        let mut final_command = Vec::with_capacity(tokens.len() + base_command.len());
        let mut found_command_placeholder = false;

        for token in tokens {
            if token == "%command%" {
                if found_command_placeholder {
                    return Err(anyhow!("Multiple %command% placeholders found"));
                }
                found_command_placeholder = true;

                // Replace %command% with the base command without cloning
                final_command.extend_from_slice(&base_command);
            } else {
                final_command.push(token);
            }
        }

        if !found_command_placeholder {
            // No %command% placeholder found, append the base command at the end
            final_command.extend_from_slice(&base_command);
        }

        // Wrap with mangohud first, then gamescope, then gamemode
        let mangohud_wrapped = self.wrap_with_mangohud(final_command)?;
        let gamescope_wrapped = self.wrap_with_gamescope(mangohud_wrapped)?;
        let gamemode_wrapped = self.wrap_with_gamemode(gamescope_wrapped)?;
        Ok(gamemode_wrapped)
    }

    /// Parse launch options string into tokens, handling quotes and environment variables safely
    fn parse_launch_options(&self, launch_options: &str) -> Result<Vec<String>> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_quotes = false;
        let chars = launch_options.chars().peekable();

        for ch in chars {
            match ch {
                '"' if !in_quotes => {
                    in_quotes = true;
                }
                '"' if in_quotes => {
                    in_quotes = false;
                }
                ' ' if !in_quotes => {
                    if !current_token.is_empty() {
                        // Validate and sanitize token before adding
                        let sanitized = self.sanitize_token(&current_token)?;
                        tokens.push(sanitized);
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            let sanitized = self.sanitize_token(&current_token)?;
            tokens.push(sanitized);
        }

        if in_quotes {
            return Err(anyhow!("Unclosed quote in launch options"));
        }

        Ok(tokens)
    }

    /// Sanitize a command token to prevent shell injection
    fn sanitize_token(&self, token: &str) -> Result<String> {
        // Check for dangerous characters and patterns
        let dangerous_chars = [
            '|', '&', ';', '`', '$', '(', ')', '{', '}', '[', ']', '*', '?', '~', '\n',
            '\r', '\t', '\'', '"',
        ];

        for ch in dangerous_chars {
            if token.contains(ch) {
                return Err(anyhow!(
                    "Dangerous character '{}' found in launch option: {}",
                    ch,
                    token
                ));
            }
        }

        // Check for dangerous patterns
        let dangerous_patterns = ["../", "./", "//", "\\\\", "\n", "\r"];
        for pattern in dangerous_patterns {
            if token.contains(pattern) {
                return Err(anyhow!(
                    "Dangerous pattern '{}' found in launch option: {}",
                    pattern,
                    token
                ));
            }
        }

        // Ensure the token doesn't start with dangerous prefixes
        let dangerous_prefixes = ["-", "--"];
        for prefix in dangerous_prefixes {
            if token.starts_with(prefix) && token != "%command%" {
                // Allow well-known safe options only
                if !self.is_safe_option(token) {
                    return Err(anyhow!("Potentially dangerous option: {}", token));
                }
            }
        }

        Ok(token.to_string())
    }

    /// Check if an option is in the allowlist of safe options
    fn is_safe_option(&self, option: &str) -> bool {
        // Allowlist of safe command line options
        let safe_options = [
            // Common safe options
            "--fullscreen",
            "--windowed",
            "--width",
            "--height",
            "--vsync",
            "--no-vsync",
            // Mangohud options
            "--dlsym",
            // Gamescope options
            "-f",
            "-w",
            "-h",
            "-W",
            "-H",
            "-r",
            "-F",
            "-S",
            "-n",
            "-b",
            "--force-grab-cursor",
            "--expose-wayland",
            "--hdr-enabled",
            "--adaptive-sync",
            "--immediate-flips",
            "--mangoapp",
        ];

        safe_options.contains(&option) ||
        // Allow numeric values
        option.parse::<i32>().is_ok() ||
        // Allow resolution patterns like "1920x1080"
        option.matches('x').count() == 1 && option.split('x').all(|s| s.parse::<u32>().is_ok())
    }

    /// Wrap command with mangohud if enabled (but not when gamescope is enabled)
    fn wrap_with_mangohud(&self, command: Vec<String>) -> Result<Vec<String>> {
        if !self.config.launch.mangohud || self.config.gamescope.enabled {
            return Ok(command);
        }

        let mut mangohud_cmd = vec!["mangohud".to_string()];
        mangohud_cmd.extend(command);
        Ok(mangohud_cmd)
    }

    /// Wrap command with gamescope if enabled
    fn wrap_with_gamescope(&self, command: Vec<String>) -> Result<Vec<String>> {
        if !self.config.gamescope.enabled {
            return Ok(command);
        }

        let gamescope_config = &self.config.gamescope;
        let mut gamescope_cmd = vec!["gamescope".to_string()];

        // Game resolution
        gamescope_cmd.push("-w".to_string());
        gamescope_cmd.push(gamescope_config.width.to_string());
        gamescope_cmd.push("-h".to_string());
        gamescope_cmd.push(gamescope_config.height.to_string());

        // Output resolution
        gamescope_cmd.push("-W".to_string());
        gamescope_cmd.push(gamescope_config.output_width.to_string());
        gamescope_cmd.push("-H".to_string());
        gamescope_cmd.push(gamescope_config.output_height.to_string());

        // Refresh rate
        gamescope_cmd.push("-r".to_string());
        gamescope_cmd.push(gamescope_config.refresh_rate.to_string());

        // Upscaling/Scaling
        match gamescope_config.upscaling.as_str() {
            "fsr" => {
                gamescope_cmd.push("-F".to_string());
                gamescope_cmd.push("fsr".to_string());
            }
            "nis" => {
                gamescope_cmd.push("-F".to_string());
                gamescope_cmd.push("nis".to_string());
            }
            "integer" => {
                gamescope_cmd.push("-S".to_string());
                gamescope_cmd.push("integer".to_string());
            }
            "stretch" => {
                gamescope_cmd.push("-S".to_string());
                gamescope_cmd.push("stretch".to_string());
            }
            "linear" => gamescope_cmd.push("-n".to_string()),
            "nearest" => gamescope_cmd.push("-b".to_string()),
            "off" => {} // No upscaling flag
            _ => {
                return Err(anyhow!(
                    "Invalid upscaling method: {}",
                    gamescope_config.upscaling
                ))
            }
        }

        // Display options
        if gamescope_config.fullscreen {
            gamescope_cmd.push("-f".to_string());
        }

        if gamescope_config.force_grab_cursor {
            gamescope_cmd.push("--force-grab-cursor".to_string());
        }

        if gamescope_config.expose_wayland {
            gamescope_cmd.push("--expose-wayland".to_string());
        }

        if gamescope_config.hdr {
            gamescope_cmd.push("--hdr-enabled".to_string());
        }

        if gamescope_config.adaptive_sync {
            gamescope_cmd.push("--adaptive-sync".to_string());
        }

        if gamescope_config.immediate_flips {
            gamescope_cmd.push("--immediate-flips".to_string());
        }

        // Add --mangoapp if mangohud is enabled
        if self.config.launch.mangohud {
            gamescope_cmd.push("--mangoapp".to_string());
        }

        // Add separator and the actual command
        gamescope_cmd.push("--".to_string());
        gamescope_cmd.extend(command);

        Ok(gamescope_cmd)
    }

    /// Wrap command with gamemode if enabled
    fn wrap_with_gamemode(&self, command: Vec<String>) -> Result<Vec<String>> {
        if !self.config.launch.gamemode {
            return Ok(command);
        }

        let mut gamemode_cmd = vec!["gamemoderun".to_string()];
        gamemode_cmd.extend(command);
        Ok(gamemode_cmd)
    }
}

/// Represents the final launch command with all components
#[derive(Debug, Clone)]
pub struct LaunchCommand {
    pub command: Vec<String>,
    pub environment: HashMap<String, String>,
    pub working_directory: PathBuf,
}
