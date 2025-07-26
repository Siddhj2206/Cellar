use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct CellarDirectories {
    pub base_dir: PathBuf,
    pub runners_dir: PathBuf,
    pub prefixes_dir: PathBuf,
    pub configs_dir: PathBuf,
    pub icons_dir: PathBuf,
    pub shortcuts_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub presets_dir: PathBuf,
    pub deps_dir: PathBuf,
    pub applications_dir: PathBuf,
}

impl CellarDirectories {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Unable to determine home directory"))?;
        
        let base_dir = home_dir.join(".local").join("share").join("cellar");
        let applications_dir = home_dir.join(".local").join("share").join("applications");
        
        let dirs = CellarDirectories {
            runners_dir: base_dir.join("runners"),
            prefixes_dir: base_dir.join("prefixes"),
            configs_dir: base_dir.join("configs"),
            icons_dir: base_dir.join("icons"),
            shortcuts_dir: base_dir.join("shortcuts"),
            templates_dir: base_dir.join("templates"),
            presets_dir: base_dir.join("presets"),
            deps_dir: base_dir.join("deps"),
            base_dir,
            applications_dir,
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
        self.ensure_dir_exists(&self.deps_dir)?;
        self.ensure_dir_exists(&self.applications_dir)?;
        
        // Create subdirectories
        self.ensure_dir_exists(&self.runners_dir.join("proton"))?;
        self.ensure_dir_exists(&self.runners_dir.join("dxvk"))?;
        self.ensure_dir_exists(&self.deps_dir.join("vcredist"))?;
        self.ensure_dir_exists(&self.deps_dir.join("dotnet"))?;
        self.ensure_dir_exists(&self.deps_dir.join("directx"))?;
        
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
        self.configs_dir.join(format!("{}.toml", sanitize_filename(game_name)))
    }
    
    pub fn get_game_prefix_path(&self, game_name: &str) -> PathBuf {
        self.prefixes_dir.join(sanitize_filename(game_name))
    }
    
    pub fn get_game_icon_path(&self, game_name: &str, extension: &str) -> PathBuf {
        self.icons_dir.join(format!("{}.{}", sanitize_filename(game_name), extension))
    }
    
    pub fn get_game_shortcut_path(&self, game_name: &str) -> PathBuf {
        self.shortcuts_dir.join(format!("{}.desktop", sanitize_filename(game_name)))
    }
    
    pub fn get_symlink_path(&self, game_name: &str) -> PathBuf {
        self.applications_dir.join(format!("cellar-{}.desktop", sanitize_filename(game_name)))
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
}

fn sanitize_filename(name: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Game"), "my_game");
        assert_eq!(sanitize_filename("Game: The Sequel"), "game__the_sequel");
        assert_eq!(sanitize_filename("Game/Part\\Two"), "game_part_two");
    }
}