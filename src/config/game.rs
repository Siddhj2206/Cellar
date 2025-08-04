use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game: GameInfo,
    pub launch: LaunchConfig,
    pub wine_config: WineConfig,
    #[serde(default)]
    pub dxvk: DxvkConfig,
    #[serde(default)]
    pub gamescope: GamescopeConfig,
    #[serde(default)]
    pub desktop: DesktopConfig,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation: Option<InstallationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub name: String,
    pub executable: PathBuf,
    pub wine_prefix: PathBuf,
    pub proton_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dxvk_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LaunchConfig {
    #[serde(default)]
    pub launch_options: String,
    #[serde(default)]
    pub game_args: Vec<String>,
    #[serde(default)]
    pub gamemode: bool,
    #[serde(default)]
    pub mangohud: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineConfig {
    #[serde(default = "default_true")]
    pub esync: bool,
    #[serde(default = "default_true")]
    pub fsync: bool,
    #[serde(default = "default_true")]
    pub dxvk: bool,
    #[serde(default = "default_true")]
    pub dxvk_async: bool,
    #[serde(default)]
    pub large_address_aware: bool,
    #[serde(default = "default_wineserver_timeout")]
    pub wineserver_kill_timeout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DxvkConfig {
    #[serde(default)]
    pub hud: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamescopeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_output_width")]
    pub output_width: u32,
    #[serde(default = "default_output_height")]
    pub output_height: u32,
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate: u32,
    #[serde(default = "default_upscaling")]
    pub upscaling: String,
    #[serde(default = "default_true")]
    pub fullscreen: bool,
    #[serde(default)]
    pub force_grab_cursor: bool,
    #[serde(default)]
    pub expose_wayland: bool,
    #[serde(default)]
    pub hdr: bool,
    #[serde(default)]
    pub adaptive_sync: bool,
    #[serde(default)]
    pub immediate_flips: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    #[serde(default = "default_true")]
    pub create_shortcut: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<PathBuf>,
    #[serde(default = "default_categories")]
    pub categories: Vec<String>,
    #[serde(default = "default_keywords")]
    pub keywords: Vec<String>,
    #[serde(default = "default_comment")]
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    pub installer_path: PathBuf,
    pub install_date: String,
    pub install_location: String,
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_wineserver_timeout() -> u32 {
    5
}

fn default_width() -> u32 {
    1920
}

fn default_height() -> u32 {
    1080
}

fn default_refresh_rate() -> u32 {
    60
}

fn default_upscaling() -> String {
    "fsr".to_string()
}

fn default_output_width() -> u32 {
    1920
}

fn default_output_height() -> u32 {
    1080
}

fn default_categories() -> Vec<String> {
    vec!["Game".to_string()]
}

fn default_keywords() -> Vec<String> {
    vec!["game".to_string(), "windows".to_string()]
}

fn default_comment() -> String {
    "Windows game via Cellar".to_string()
}

impl Default for WineConfig {
    fn default() -> Self {
        Self {
            esync: true,
            fsync: true,
            dxvk: true,
            dxvk_async: true,
            large_address_aware: false,
            wineserver_kill_timeout: 5,
        }
    }
}

impl Default for GamescopeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            width: 1920,
            height: 1080,
            output_width: 1920,
            output_height: 1080,
            refresh_rate: 60,
            upscaling: "fsr".to_string(),
            fullscreen: true,
            force_grab_cursor: false,
            expose_wayland: false,
            hdr: false,
            adaptive_sync: false,
            immediate_flips: false,
        }
    }
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            create_shortcut: true,
            icon_path: None,
            categories: vec!["Game".to_string()],
            keywords: vec!["game".to_string(), "windows".to_string()],
            comment: "Windows game via Cellar".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_config_basic_functionality() {
        let config = GameConfig {
            game: GameInfo {
                name: "Test Game".to_string(),
                executable: std::path::PathBuf::from("/path/to/game.exe"),
                wine_prefix: std::path::PathBuf::from("/path/to/prefix"),
                proton_version: "GE-Proton8-32".to_string(),
                dxvk_version: None,
            },
            launch: LaunchConfig::default(),
            wine_config: WineConfig::default(),
            dxvk: DxvkConfig::default(),
            gamescope: GamescopeConfig::default(),
            desktop: DesktopConfig::default(),
            installation: None,
        };

        assert_eq!(config.game.name, "Test Game");
        assert_eq!(config.game.proton_version, "GE-Proton8-32");
        assert!(config.wine_config.esync);
        assert!(!config.launch.mangohud);
        assert!(!config.gamescope.enabled);
    }

    #[test]
    fn test_game_config_serialization() {
        let config = GameConfig {
            game: GameInfo {
                name: "Test Game".to_string(),
                executable: std::path::PathBuf::from("/path/to/game.exe"),
                wine_prefix: std::path::PathBuf::from("/path/to/prefix"),
                proton_version: "GE-Proton8-32".to_string(),
                dxvk_version: None,
            },
            launch: LaunchConfig::default(),
            wine_config: WineConfig::default(),
            dxvk: DxvkConfig::default(),
            gamescope: GamescopeConfig::default(),
            desktop: DesktopConfig::default(),
            installation: None,
        };

        let toml_string = toml::to_string(&config).unwrap();

        // Verify it contains expected sections
        assert!(toml_string.contains("[game]"));
        assert!(toml_string.contains("[launch]"));
        assert!(toml_string.contains("[wine_config]"));
        assert!(toml_string.contains("Test Game"));
        assert!(toml_string.contains("GE-Proton8-32"));
    }

    #[test]
    fn test_game_config_deserialization() {
        let toml_string = r#"
[game]
name = "Test Game"
executable = "/path/to/game.exe"
wine_prefix = "/path/to/prefix"
proton_version = "GE-Proton8-32"

[launch]
launch_options = "PROTON_ENABLE_WAYLAND=1 %command%"
game_args = ["--windowed"]

[wine_config]
esync = true
fsync = true
dxvk = true
dxvk_async = true
large_address_aware = false
wineserver_kill_timeout = 5
"#;

        let config: GameConfig = toml::from_str(toml_string).unwrap();
        assert_eq!(config.game.name, "Test Game");
        assert_eq!(config.game.proton_version, "GE-Proton8-32");
        assert_eq!(config.launch.game_args, vec!["--windowed"]);
        assert!(config.wine_config.esync);
    }
}
