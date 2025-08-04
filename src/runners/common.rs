use anyhow::{anyhow, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Type alias for asset filter function
pub type AssetFilter = fn(&str) -> bool;

/// Common configuration for GitHub-based runners
pub struct GitHubRunnerConfig {
    pub repo_owner: String,
    pub repo_name: String,
    pub user_agent: String,
    pub max_download_size: u64,
    pub asset_filter: AssetFilter,
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
    /// Creates a new `BaseGitHubRunner` with the specified configuration and runners directory path.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = GitHubRunnerConfig {
    ///     repo_owner: "owner".to_string(),
    ///     repo_name: "repo".to_string(),
    ///     user_agent: "my-agent".to_string(),
    ///     max_download_size: 100_000_000,
    ///     asset_filter: |name| name.ends_with(".tar.gz"),
    /// };
    /// let runners_path = std::path::PathBuf::from("/tmp/runners");
    /// let runner = BaseGitHubRunner::new(config, runners_path);
    /// ```
    pub fn new(config: GitHubRunnerConfig, cellar_runners_path: PathBuf) -> Self {
        Self {
            config,
            cellar_runners_path,
        }
    }

    /// Downloads a runner asset from a specific GitHub release and saves it to a temporary file.
    ///
    /// Fetches release information for the given version and tag prefix, selects an asset matching the configured filter,
    /// verifies its size constraints, downloads the asset, and writes it to the system's temporary directory. Returns the path
    /// to the downloaded file if successful.
    ///
    /// # Parameters
    /// - `version`: The release version to fetch.
    /// - `tag_prefix`: The prefix to prepend to the version when constructing the release tag.
    ///
    /// # Returns
    /// The path to the downloaded asset file in the temporary directory.
    ///
    /// # Errors
    /// Returns an error if the release or asset cannot be found, if the asset exceeds the maximum allowed size,
    /// if the download fails, or if the downloaded file does not match the expected size.
    ///
    /// # Examples
    ///
    /// ```
    /// # use runners::common::{BaseGitHubRunner, GitHubRunnerConfig};
    /// # async fn example(runner: BaseGitHubRunner) {
    /// let path = runner.download_from_github("v1.2.3", "v").await.unwrap();
    /// assert!(path.exists());
    /// # }
    /// ```
    pub async fn download_from_github(&self, version: &str, tag_prefix: &str) -> Result<PathBuf> {
        let client = reqwest::Client::builder()
            .user_agent(&self.config.user_agent)
            .build()?;

        // Get release info from GitHub API
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/tags/{}{}",
            self.config.repo_owner, self.config.repo_name, tag_prefix, version
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

    /// Retrieves a list of available release versions from the configured GitHub repository.
    ///
    /// Sends a request to the GitHub releases API and returns the tag names of all releases as a vector of strings.
    ///
    /// # Returns
    /// A vector of release tag names on success.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or if the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// let runner = BaseGitHubRunner::new(config, cellar_runners_path);
    /// let versions = tokio_test::block_on(runner.get_github_versions()).unwrap();
    /// assert!(!versions.is_empty());
    /// ```
    pub async fn get_github_versions(&self) -> Result<Vec<String>> {
        let client = reqwest::Client::builder()
            .user_agent(&self.config.user_agent)
            .build()?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.config.repo_owner, self.config.repo_name
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

    /// Deletes the specified runner directory and its contents.
    ///
    /// Returns an error if the path does not exist, is not a directory, or if deletion fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # async fn example(runner: &BaseGitHubRunner, path: &Path) {
    /// let result = runner.delete_runner_common(path).await;
    /// assert!(result.is_ok());
    /// # }
    /// ```
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
