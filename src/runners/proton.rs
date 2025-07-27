use super::{Runner, RunnerManager, RunnerType};
use anyhow::{anyhow, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtonRelease {
    pub tag_name: String,
    pub name: String,
    pub assets: Vec<ProtonAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtonAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

pub struct ProtonManager {
    pub steam_path: Option<PathBuf>,
    pub cellar_runners_path: PathBuf,
}

impl ProtonManager {
    pub fn new(cellar_runners_path: PathBuf) -> Self {
        let steam_path = Self::find_steam_path();
        Self {
            steam_path,
            cellar_runners_path,
        }
    }

    fn find_steam_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let steam_paths = [home.join(".steam/steam"), home.join(".local/share/Steam")];

        for path in &steam_paths {
            if path.join("steamapps/common").exists() {
                return Some(path.clone());
            }
        }
        None
    }

    pub async fn discover_steam_proton(&self) -> Result<Vec<Runner>> {
        let mut runners = Vec::new();

        if let Some(steam_path) = &self.steam_path {
            let proton_path = steam_path.join("steamapps/common");
            if proton_path.exists() {
                let mut entries = fs::read_dir(&proton_path).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        // Check if this looks like a Proton installation
                        if name.to_lowercase().contains("proton") {
                            // Look for proton executable
                            let proton_exe = path.join("proton");
                            if proton_exe.exists() {
                                let version = self.extract_version_from_name(&name);
                                runners.push(Runner {
                                    name: name.clone(),
                                    version,
                                    path: path.clone(),
                                    runner_type: RunnerType::Proton,
                                    installed: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(runners)
    }

    pub async fn discover_cellar_proton(&self) -> Result<Vec<Runner>> {
        let mut runners = Vec::new();
        let proton_path = self.cellar_runners_path.join("proton");

        if proton_path.exists() {
            let mut entries = fs::read_dir(&proton_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    // Look for proton executable
                    let proton_exe = path.join("proton");
                    if proton_exe.exists() {
                        let version = self.extract_version_from_name(&name);
                        runners.push(Runner {
                            name: name.clone(),
                            version,
                            path: path.clone(),
                            runner_type: RunnerType::Proton,
                            installed: true,
                        });
                    }
                }
            }
        }

        Ok(runners)
    }

    fn extract_version_from_name(&self, name: &str) -> String {
        // Extract version from names like "GE-Proton8-32" or "Proton 8.0"
        if let Some(captures) = regex::Regex::new(r"(?i)proton[^\d]*(\d+(?:[.-]\d+)*)")
            .unwrap()
            .captures(name)
        {
            captures
                .get(1)
                .map_or(name.to_string(), |m| m.as_str().to_string())
        } else {
            name.to_string()
        }
    }

    pub async fn download_ge_proton(&self, version: &str) -> Result<PathBuf> {
        let client = reqwest::Client::builder()
            .user_agent("cellar/0.1.0")
            .build()?;

        // Get release info from GitHub API
        let url = format!(
            "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases/tags/GE-Proton{version}"
        );
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch release info for version {}",
                version
            ));
        }

        let release: ProtonRelease = response.json().await?;

        // Find the tar.gz asset (Proton-GE uses tar.gz)
        let asset = release
            .assets
            .iter()
            .find(|a| a.name.ends_with(".tar.gz"))
            .ok_or_else(|| anyhow!("No tar.gz asset found for version {}", version))?;

        // Download the asset
        let download_response = client.get(&asset.browser_download_url).send().await?;

        if !download_response.status().is_success() {
            return Err(anyhow!("Failed to download {}", asset.name));
        }

        // Save to temporary file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(&asset.name);

        let bytes = download_response.bytes().await?;
        fs::write(&temp_file, bytes).await?;

        Ok(temp_file)
    }

    pub async fn extract_proton(&self, archive_path: &Path, version: &str) -> Result<PathBuf> {
        let proton_dir = self.cellar_runners_path.join("proton");
        fs::create_dir_all(&proton_dir).await?;

        let extract_path = proton_dir.join(version);
        fs::create_dir_all(&extract_path).await?;

        // Extract tar.gz file
        let file = std::fs::File::open(archive_path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);

        // Extract to temporary directory first
        let temp_extract = std::env::temp_dir().join(format!("proton-extract-{version}"));
        std::fs::create_dir_all(&temp_extract)?;
        archive.unpack(&temp_extract)?;

        // Find the extracted directory (usually the first subdirectory)
        let mut entries = std::fs::read_dir(&temp_extract)?;
        if let Some(entry) = entries.next() {
            let extracted_dir = entry?.path();
            if extracted_dir.is_dir() {
                // Move contents to final destination
                self.move_directory_contents(&extracted_dir, &extract_path)
                    .await?;
            }
        }

        // Clean up
        std::fs::remove_dir_all(&temp_extract)?;
        std::fs::remove_file(archive_path)?;

        Ok(extract_path)
    }

    async fn move_directory_contents(&self, src: &Path, dest: &Path) -> Result<()> {
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();
        queue.push_back((src.to_path_buf(), dest.to_path_buf()));

        while let Some((src_dir, dest_dir)) = queue.pop_front() {
            let mut entries = fs::read_dir(&src_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let src_path = entry.path();
                let dest_path = dest_dir.join(entry.file_name());

                if src_path.is_dir() {
                    fs::create_dir_all(&dest_path).await?;
                    queue.push_back((src_path, dest_path));
                } else {
                    fs::copy(&src_path, &dest_path).await?;
                }
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl RunnerManager for ProtonManager {
    async fn discover_local_runners(&self) -> Result<Vec<Runner>> {
        let mut runners = Vec::new();

        // Discover Steam Proton installations
        runners.extend(self.discover_steam_proton().await?);

        // Discover Cellar Proton installations
        runners.extend(self.discover_cellar_proton().await?);

        Ok(runners)
    }

    async fn download_runner(&self, _name: &str, version: &str) -> Result<PathBuf> {
        self.download_ge_proton(version).await
    }

    async fn install_runner(&self, download_path: &Path, _install_path: &Path) -> Result<()> {
        // Extract version from download path filename
        let filename = download_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid download path"))?;

        let version = filename.replace(".tar.gz", "");
        self.extract_proton(download_path, &version).await?;

        Ok(())
    }

    async fn get_available_versions(&self) -> Result<Vec<String>> {
        let client = reqwest::Client::builder()
            .user_agent("curl/8.15.0")
            .build()?;
        let url = "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases";

        let response = client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch available versions: HTTP {}",
                response.status()
            ));
        }

        let releases: Vec<ProtonRelease> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;
        let versions = releases.into_iter().map(|r| r.tag_name).collect();

        Ok(versions)
    }

    async fn delete_runner(&self, runner_path: &Path) -> Result<()> {
        if !runner_path.exists() {
            return Err(anyhow!(
                "Runner path does not exist: {}",
                runner_path.display()
            ));
        }

        if !runner_path.is_dir() {
            return Err(anyhow!(
                "Runner path is not a directory: {}",
                runner_path.display()
            ));
        }

        fs::remove_dir_all(runner_path).await.map_err(|e| {
            anyhow!(
                "Failed to delete runner at {}: {}",
                runner_path.display(),
                e
            )
        })?;

        Ok(())
    }
}
