use anyhow::{anyhow, Result};
use crate::utils::archive::extract_tar_gz_secure;
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Common configuration for GitHub-based runners
pub struct GitHubRunnerConfig {
    pub repo_owner: String,
    pub repo_name: String,
    pub user_agent: String,
    pub max_download_size: u64,
    pub max_files: usize,
    pub max_total_size: u64,
    pub asset_filter: Box<dyn Fn(&str) -> bool + Send + Sync>,
}

/// Common GitHub release structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Base runner implementation for GitHub-based runners
pub struct BaseGitHubRunner {
    pub config: GitHubRunnerConfig,
    pub cellar_runners_path: PathBuf,
}

impl BaseGitHubRunner {
    pub fn new(config: GitHubRunnerConfig, cellar_runners_path: PathBuf) -> Self {
        Self {
            config,
            cellar_runners_path,
        }
    }

    /// Download a runner from GitHub releases
    pub async fn download_from_github(&self, version: &str, tag_prefix: &str) -> Result<PathBuf> {
        let client = reqwest::Client::builder()
            .user_agent(&self.config.user_agent)
            .build()?;

        // Get release info from GitHub API
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/tags/{}{}",
            self.config.repo_owner, 
            self.config.repo_name, 
            tag_prefix,
            version
        );
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch release info for version {} from {}/{}",
                version,
                self.config.repo_owner,
                self.config.repo_name
            ));
        }

        let release: GitHubRelease = response.json().await?;

        // Find the appropriate asset using the filter
        let asset = release
            .assets
            .iter()
            .find(|a| (self.config.asset_filter)(&a.name))
            .ok_or_else(|| anyhow!("No suitable asset found for version {}", version))?;

        // Check asset size limit
        if asset.size > self.config.max_download_size {
            return Err(anyhow!(
                "Asset too large: {} bytes (max {} bytes)", 
                asset.size, 
                self.config.max_download_size
            ));
        }

        // Download the asset
        let download_response = client.get(&asset.browser_download_url).send().await?;

        if !download_response.status().is_success() {
            return Err(anyhow!("Failed to download {}", asset.name));
        }

        // Verify content length matches expected size
        if let Some(content_length) = download_response.content_length() {
            if content_length != asset.size {
                return Err(anyhow!(
                    "Content length mismatch: expected {}, got {}",
                    asset.size,
                    content_length
                ));
            }
        }

        // Save to temporary file with size verification
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(&asset.name);

        let bytes = download_response.bytes().await?;
        
        // Verify downloaded size
        if bytes.len() as u64 != asset.size {
            return Err(anyhow!(
                "Downloaded size mismatch: expected {}, got {}",
                asset.size,
                bytes.len()
            ));
        }
        
        fs::write(&temp_file, bytes).await?;

        Ok(temp_file)
    }

    /// Extract a tar.gz runner archive securely
    pub async fn extract_runner_archive(
        &self,
        archive_path: &Path,
        runner_subdir: &str,
        version: &str,
    ) -> Result<PathBuf> {
        let runner_dir = self.cellar_runners_path.join(runner_subdir);
        let extract_path = runner_dir.join(version);
        
        // Use secure extraction with configured limits
        extract_tar_gz_secure(
            archive_path, 
            &extract_path, 
            self.config.max_files, 
            self.config.max_total_size
        ).await?;

        // Clean up archive
        fs::remove_file(archive_path).await?;

        Ok(extract_path)
    }

    /// Get available versions from GitHub releases
    pub async fn get_github_versions(&self) -> Result<Vec<String>> {
        let client = reqwest::Client::builder()
            .user_agent(&self.config.user_agent)
            .build()?;
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.config.repo_owner,
            self.config.repo_name
        );

        let response = client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch available versions from {}/{}: HTTP {}",
                self.config.repo_owner,
                self.config.repo_name,
                response.status()
            ));
        }

        let releases: Vec<GitHubRelease> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;
        
        let versions = releases.into_iter().map(|r| r.tag_name).collect();

        Ok(versions)
    }

    /// Common runner deletion logic
    pub async fn delete_runner_common(&self, runner_path: &Path) -> Result<()> {
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