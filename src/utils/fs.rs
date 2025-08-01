use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Expand tilde (~) in paths to the actual home directory
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();
    
    if path_str.starts_with("~/") {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Unable to determine home directory"))?;
        let without_tilde = &path_str[2..]; // Remove "~/"
        Ok(home_dir.join(without_tilde))
    } else if path_str == "~" {
        dirs::home_dir()
            .ok_or_else(|| anyhow!("Unable to determine home directory"))
    } else {
        Ok(path.to_path_buf())
    }
}

pub struct CellarDirectories {
    pub base_dir: PathBuf,
    pub runners_dir: PathBuf,
    pub prefixes_dir: PathBuf,
    pub configs_dir: PathBuf,
    pub icons_dir: PathBuf,
    pub shortcuts_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub presets_dir: PathBuf,
    pub applications_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl CellarDirectories {
    pub fn new() -> Result<Self> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;

        let base_dir = home_dir.join(".local").join("share").join("cellar");
        let applications_dir = home_dir.join(".local").join("share").join("applications");
        let cache_dir = base_dir.join("cache");

        let dirs = CellarDirectories {
            runners_dir: base_dir.join("runners"),
            prefixes_dir: base_dir.join("prefixes"),
            configs_dir: base_dir.join("configs"),
            icons_dir: base_dir.join("icons"),
            shortcuts_dir: base_dir.join("shortcuts"),
            templates_dir: base_dir.join("templates"),
            presets_dir: base_dir.join("presets"),
            base_dir,
            applications_dir,
            cache_dir,
        };

        Ok(dirs)
    }

    pub fn ensure_all_exist(&self) -> Result<()> {
        self.ensure_dir_exists(&self.base_dir)?;
        self.ensure_dir_exists(&self.runners_dir)?;
        self.ensure_dir_exists(&self.prefixes_dir)?;
        self.ensure_dir_exists(&self.configs_dir)?;
        self.ensure_dir_exists(&self.icons_dir)?;
        self.ensure_dir_exists(&self.shortcuts_dir)?;
        self.ensure_dir_exists(&self.templates_dir)?;
        self.ensure_dir_exists(&self.presets_dir)?;
        self.ensure_dir_exists(&self.applications_dir)?;
        self.ensure_dir_exists(&self.cache_dir)?;

        // Create subdirectories
        self.ensure_dir_exists(&self.runners_dir.join("proton"))?;
        self.ensure_dir_exists(&self.runners_dir.join("dxvk"))?;

        Ok(())
    }

    pub fn ensure_dir_exists(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| anyhow!("Failed to create directory {}: {}", path.display(), e))?;
        }
        Ok(())
    }

    pub fn get_game_config_path(&self, game_name: &str) -> PathBuf {
        self.configs_dir
            .join(format!("{}.toml", sanitize_filename(game_name)))
    }

    // pub fn get_game_prefix_path(&self, game_name: &str) -> PathBuf {
    //     self.prefixes_dir.join(sanitize_filename(game_name))
    // }

    #[allow(dead_code)]
    pub fn get_game_icon_path(&self, game_name: &str, extension: &str) -> PathBuf {
        self.icons_dir
            .join(format!("{}.{}", sanitize_filename(game_name), extension))
    }

    #[allow(dead_code)]
    pub fn get_game_shortcut_path(&self, game_name: &str) -> PathBuf {
        self.shortcuts_dir
            .join(format!("{}.desktop", sanitize_filename(game_name)))
    }

    #[allow(dead_code)]
    pub fn get_symlink_path(&self, game_name: &str) -> PathBuf {
        self.applications_dir
            .join(format!("cellar-{}.desktop", sanitize_filename(game_name)))
    }

    pub fn list_game_configs(&self) -> Result<Vec<String>> {
        let mut games = Vec::new();

        if self.configs_dir.exists() {
            for entry in fs::read_dir(&self.configs_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        games.push(stem.to_string());
                    }
                }
            }
        }

        games.sort();
        Ok(games)
    }

    pub fn get_runners_path(&self) -> PathBuf {
        self.runners_dir.clone()
    }

    pub fn get_prefixes_path(&self) -> PathBuf {
        self.prefixes_dir.clone()
    }

    pub fn get_cache_path(&self) -> PathBuf {
        self.cache_dir.clone()
    }
}

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_lowercase()
        .replace(' ', "_")
}
