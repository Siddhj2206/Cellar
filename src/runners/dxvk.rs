use super::common::{AssetFilter, BaseGitHubRunner, GitHubRunnerConfig};
use super::{Runner, RunnerManager, RunnerType};
use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct DxvkManager {
    pub base_runner: BaseGitHubRunner,
}

impl DxvkManager {
    /// Creates a new `DxvkManager` for managing DXVK runners in the specified cellar directory.
    ///
    /// Configures the manager to interact with the "doitsujin/dxvk" GitHub repository, filtering for non-source `.tar.gz` release assets up to 1GB in size.
    ///
    /// # Examples
    ///
    /// ```
    /// let cellar_path = std::path::PathBuf::from("/path/to/cellar/runners");
    /// let manager = DxvkManager::new(cellar_path);
    /// ```
    pub fn new(cellar_runners_path: PathBuf) -> Self {
        fn asset_filter(name: &str) -> bool {
            name.ends_with(".tar.gz") && !name.contains("source")
        }

        let config = GitHubRunnerConfig {
            repo_owner: "doitsujin".to_string(),
            repo_name: "dxvk".to_string(),
            user_agent: "cellar/0.1.0".to_string(),
            max_download_size: 1024 * 1024 * 1024, // 1GB
            asset_filter: asset_filter as AssetFilter,
        };

        let base_runner = BaseGitHubRunner::new(config, cellar_runners_path);

        Self { base_runner }
    }

    /// Discovers locally installed DXVK runners in the cellar directory.
    ///
    /// Searches the `dxvk` subdirectory of the cellar runners path for valid DXVK installations,
    /// identified by the presence of `x64` or `x32` subdirectories. Returns a list of `Runner`
    /// instances representing each discovered DXVK installation.
    ///
    /// # Returns
    /// A vector of `Runner` objects for each detected DXVK installation.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = DxvkManager::new(cellar_path);
    /// let runners = tokio_test::block_on(manager.discover_cellar_dxvk()).unwrap();
    /// for runner in runners {
    ///     assert_eq!(runner.runner_type, RunnerType::Dxvk);
    /// }
    /// ```
    pub async fn discover_cellar_dxvk(&self) -> Result<Vec<Runner>> {
        let mut runners = Vec::new();
        let dxvk_path = self.base_runner.cellar_runners_path.join("dxvk");

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

    /// Extracts the version number from a DXVK release name.
    ///
    /// Searches for a version pattern (e.g., "v2.3.1" or "dxvk-2.3.1") in the input string and returns the version number.
    /// If no version pattern is found, returns the original string.
    ///
    /// # Examples
    ///
    /// ```
    /// let version = manager.extract_version_from_name("dxvk-2.3.1");
    /// assert_eq!(version, "2.3.1");
    ///
    /// let version = manager.extract_version_from_name("v1.10");
    /// assert_eq!(version, "1.10");
    ///
    /// let version = manager.extract_version_from_name("unknown");
    /// assert_eq!(version, "unknown");
    /// ```
    fn extract_version_from_name(&self, name: &str) -> String {
        // Extract version from names like "v2.3.1" or "dxvk-2.3.1"
        if let Some(captures) = Regex::new(r"v?(\d+\.\d+(?:\.\d+)?)")
            .unwrap()
            .captures(name)
        {
            captures
                .get(1)
                .map_or_else(|| name.to_string(), |m| m.as_str().to_string())
        } else {
            name.to_string()
        }
    }

    /// Downloads the specified version of DXVK from GitHub.
    ///
    /// Returns the path to the downloaded archive on success.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = DxvkManager::new(cellar_path);
    /// let archive_path = manager.download_dxvk("2.3.1").await?;
    /// assert!(archive_path.ends_with(".tar.gz"));
    /// ```
    pub async fn download_dxvk(&self, version: &str) -> Result<PathBuf> {
        self.base_runner.download_from_github(version, "v").await
    }

    /// Extracts a DXVK `.tar.gz` archive to the cellar directory for the specified version.
    ///
    /// The archive is first unpacked to a temporary directory, then its contents are moved to the final extraction path under `dxvk/v{version}` in the cellar. Temporary files and the original archive are deleted after extraction.
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to the DXVK `.tar.gz` archive.
    /// * `version` - Version string used to determine the extraction directory.
    ///
    /// # Returns
    ///
    /// The path to the extracted DXVK directory.
    ///
    /// # Examples
    ///
    /// ```
    /// let extracted_path = manager.extract_dxvk(Path::new("/tmp/dxvk-2.3.1.tar.gz"), "2.3.1").await?;
    /// assert!(extracted_path.ends_with("dxvk/v2.3.1"));
    /// ```
    pub async fn extract_dxvk(&self, archive_path: &Path, version: &str) -> Result<PathBuf> {
        let dxvk_dir = self.base_runner.cellar_runners_path.join("dxvk");
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

    /// Installs a DXVK runner by extracting the downloaded archive to the appropriate location.
    ///
    /// Extracts the version from the archive filename and unpacks the contents using `extract_dxvk`.
    ///
    /// # Errors
    ///
    /// Returns an error if the download path is invalid or extraction fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = DxvkManager::new(cellar_path);
    /// let archive = Path::new("/tmp/dxvk-v2.3.1.tar.gz");
    /// manager.install_runner(archive, Path::new("")).await?;
    /// ```
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

    /// Retrieves a list of available DXVK versions from GitHub, with leading 'v' prefixes removed.
    ///
    /// # Returns
    /// A vector of version strings without the leading 'v' character.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = DxvkManager::new(cellar_path);
    /// let versions = tokio_test::block_on(manager.get_available_versions()).unwrap();
    /// assert!(versions.iter().all(|v| !v.starts_with('v')));
    /// ```
    async fn get_available_versions(&self) -> Result<Vec<String>> {
        let versions = self.base_runner.get_github_versions().await?;
        // Strip 'v' prefix from versions for consistency
        let stripped_versions = versions
            .into_iter()
            .map(|v| v.strip_prefix("v").unwrap_or(&v).to_string())
            .collect();
        Ok(stripped_versions)
    }

    /// Deletes the specified DXVK runner directory and its contents asynchronously.
    ///
    /// # Arguments
    ///
    /// * `runner_path` - Path to the runner directory to be deleted.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = DxvkManager::new(cellar_path);
    /// manager.delete_runner(Path::new("/path/to/runner")).await?;
    /// ```
    async fn delete_runner(&self, runner_path: &Path) -> Result<()> {
        self.base_runner.delete_runner_common(runner_path).await
    }
}
