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

    pub async fn download_dxvk(&self, version: &str) -> Result<PathBuf> {
        self.base_runner.download_from_github(version, "v").await
    }

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
        let versions = self.base_runner.get_github_versions().await?;
        // Strip 'v' prefix from versions for consistency
        let stripped_versions = versions
            .into_iter()
            .map(|v| v.strip_prefix("v").unwrap_or(&v).to_string())
            .collect();
        Ok(stripped_versions)
    }

    async fn delete_runner(&self, runner_path: &Path) -> Result<()> {
        self.base_runner.delete_runner_common(runner_path).await
    }
}
