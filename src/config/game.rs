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
    pub mangohud: MangohudConfig,
    #[serde(default)]
    pub desktop: DesktopConfig,
    #[serde(default)]
    pub dependencies: DependenciesConfig,
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
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LaunchConfig {
    #[serde(default)]
    pub launch_options: String,
    #[serde(default)]
    pub game_args: Vec<String>,
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
pub struct MangohudConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub fps: bool,
    #[serde(default = "default_true")]
    pub gpu_stats: bool,
    #[serde(default = "default_true")]
    pub cpu_stats: bool,
    #[serde(default)]
    pub frame_timing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    #[serde(default = "default_true")]
    pub create_shortcut: bool,
    #[serde(default = "default_true")]
    pub create_symlink: bool,
    #[serde(default)]
    pub install_system: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<PathBuf>,
    #[serde(default = "default_categories")]
    pub categories: Vec<String>,
    #[serde(default = "default_keywords")]
    pub keywords: Vec<String>,
    #[serde(default = "default_comment")]
    pub comment: String,
    #[serde(default = "default_true")]
    pub prefix_name: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependenciesConfig {
    #[serde(default)]
    pub vcredist2019: bool,
    #[serde(default)]
    pub dotnet48: bool,
    #[serde(default)]
    pub directx: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    pub installer_path: PathBuf,
    pub install_date: String,
    pub install_location: String,
}

// Default value functions
fn default_status() -> String {
    "configured".to_string()
}

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

impl Default for MangohudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            fps: true,
            gpu_stats: true,
            cpu_stats: true,
            frame_timing: false,
            config_file: None,
        }
    }
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            create_shortcut: true,
            create_symlink: true,
            install_system: false,
            icon_path: None,
            categories: vec!["Game".to_string()],
            keywords: vec!["game".to_string(), "windows".to_string()],
            comment: "Windows game via Cellar".to_string(),
            prefix_name: true,
        }
    }
}
