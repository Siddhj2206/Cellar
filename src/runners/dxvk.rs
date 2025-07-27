use super::{Runner, RunnerManager, RunnerType};
use anyhow::{anyhow, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DxvkRelease {
    pub tag_name: String,
    pub name: String,
    pub assets: Vec<DxvkAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DxvkAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

pub struct DxvkManager {
    pub cellar_runners_path: PathBuf,
}

impl DxvkManager {
    pub fn new(cellar_runners_path: PathBuf) -> Self {
        Self {
            cellar_runners_path,
        }
    }

    pub async fn discover_cellar_dxvk(&self) -> Result<Vec<Runner>> {
        let mut runners = Vec::new();
        let dxvk_path = self.cellar_runners_path.join("dxvk");

        if dxvk_path.exists() {
            let mut entries = fs::read_dir(&dxvk_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    // Check if this looks like a DXVK installation
                    let x64_path = path.join("x64");
                    let x32_path = path.join("x32");
                    if x64_path.exists() || x32_path.exists() {
                        let version = self.extract_version_from_name(&name);
                        runners.push(Runner {
                            name: format!("DXVK-{name}"),
                            version,
                            path: path.clone(),
                            runner_type: RunnerType::Dxvk,
                            installed: true,
                        });
                    }
                }
            }
        }

        Ok(runners)
    }

    fn extract_version_from_name(&self, name: &str) -> String {
        // Extract version from names like "v2.3.1" or "dxvk-2.3.1"
        if let Some(captures) = regex::Regex::new(r"v?(\d+\.\d+(?:\.\d+)?)")
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

    pub async fn download_dxvk(&self, version: &str) -> Result<PathBuf> {
        let client = reqwest::Client::builder()
            .user_agent("curl/8.15.0")
            .build()?;

        // Get release info from GitHub API
        let url = format!("https://api.github.com/repos/doitsujin/dxvk/releases/tags/v{version}");
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch release info for DXVK version {}",
                version
            ));
        }

        let release: DxvkRelease = response.json().await?;

        // Find the tar.gz asset
        let asset = release
            .assets
            .iter()
            .find(|a| a.name.ends_with(".tar.gz") && !a.name.contains("source"))
            .ok_or_else(|| anyhow!("No binary tar.gz asset found for DXVK version {}", version))?;

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

    pub async fn extract_dxvk(&self, archive_path: &Path, version: &str) -> Result<PathBuf> {
        let dxvk_dir = self.cellar_runners_path.join("dxvk");
        fs::create_dir_all(&dxvk_dir).await?;

        let extract_path = dxvk_dir.join(format!("v{version}"));
        fs::create_dir_all(&extract_path).await?;

        // Extract tar.gz file
        let file = std::fs::File::open(archive_path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);

        // Extract to temporary directory first
        let temp_extract = std::env::temp_dir().join(format!("dxvk-extract-{version}"));
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

    pub async fn install_dxvk_to_prefix(&self, dxvk_path: &Path, prefix_path: &Path) -> Result<()> {
        let system32_path = prefix_path.join("drive_c/windows/system32");
        let syswow64_path = prefix_path.join("drive_c/windows/syswow64");

        // Ensure directories exist
        fs::create_dir_all(&system32_path).await?;
        fs::create_dir_all(&syswow64_path).await?;

        // Copy x64 DLLs to system32
        let x64_path = dxvk_path.join("x64");
        if x64_path.exists() {
            let mut entries = fs::read_dir(&x64_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let src = entry.path();
                if src.extension().and_then(|s| s.to_str()) == Some("dll") {
                    let dest = system32_path.join(entry.file_name());
                    fs::copy(&src, &dest).await?;
                }
            }
        }

        // Copy x32 DLLs to syswow64
        let x32_path = dxvk_path.join("x32");
        if x32_path.exists() {
            let mut entries = fs::read_dir(&x32_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let src = entry.path();
                if src.extension().and_then(|s| s.to_str()) == Some("dll") {
                    let dest = syswow64_path.join(entry.file_name());
                    fs::copy(&src, &dest).await?;
                }
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl RunnerManager for DxvkManager {
    async fn discover_local_runners(&self) -> Result<Vec<Runner>> {
        self.discover_cellar_dxvk().await
    }

    async fn download_runner(&self, _name: &str, version: &str) -> Result<PathBuf> {
        self.download_dxvk(version).await
    }

    async fn install_runner(&self, download_path: &Path, _install_path: &Path) -> Result<()> {
        // Extract version from download path filename
        let filename = download_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid download path"))?;

        // Extract version (remove "dxvk-" prefix and ".tar.gz" suffix)
        let version = filename
            .strip_prefix("dxvk-")
            .unwrap_or(filename)
            .strip_suffix(".tar.gz")
            .unwrap_or(filename)
            .strip_prefix("v")
            .unwrap_or(filename);

        self.extract_dxvk(download_path, version).await?;

        Ok(())
    }

    async fn get_available_versions(&self) -> Result<Vec<String>> {
        let client = reqwest::Client::builder()
            .user_agent("cellar/0.1.0")
            .build()?;
        let url = "https://api.github.com/repos/doitsujin/dxvk/releases";

        let response = client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch available DXVK versions: HTTP {}",
                response.status()
            ));
        }

        let releases: Vec<DxvkRelease> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;
        let versions = releases
            .into_iter()
            .map(|r| {
                r.tag_name
                    .strip_prefix("v")
                    .unwrap_or(&r.tag_name)
                    .to_string()
            })
            .collect();

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
