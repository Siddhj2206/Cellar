use anyhow::{anyhow, Result};
use std::path::{Component, Path, PathBuf};
use tokio::fs;

/// Validates an archive entry path to prevent path traversal attacks
pub fn validate_archive_path(entry_path: &Path, destination: &Path) -> Result<PathBuf> {
    // Normalize the path by resolving all components
    let mut normalized = PathBuf::new();
    
    for component in entry_path.components() {
        match component {
            Component::Normal(name) => normalized.push(name),
            Component::CurDir => {
                // Skip current directory references
                continue;
            }
            Component::ParentDir => {
                // Prevent parent directory traversal
                return Err(anyhow!(
                    "Path traversal attempt detected: {:?}",
                    entry_path
                ));
            }
            Component::RootDir => {
                // Prevent absolute paths
                return Err(anyhow!(
                    "Absolute path not allowed in archive: {:?}",
                    entry_path
                ));
            }
            Component::Prefix(_) => {
                // Prevent Windows drive prefixes
                return Err(anyhow!(
                    "Path prefix not allowed in archive: {:?}",
                    entry_path
                ));
            }
        }
    }
    
    // Check if the path contains any suspicious patterns
    let path_str = normalized.to_string_lossy();
    if path_str.contains("..")
        || path_str.starts_with('/')
        || path_str.starts_with('\\')
        || path_str.contains('\0')
        || path_str.contains('\\')
        || path_str.contains(".\\")
        || path_str.contains("./")
        || path_str.contains(":\\")
    {
        return Err(anyhow!(
            "Suspicious path pattern detected: {:?}",
            entry_path
        ));
    }
    
    // Construct the final destination path
    let final_path = destination.join(&normalized);
    
    // Ensure the final path is still within the destination directory
    match final_path.canonicalize() {
        Ok(canonical) => {
            let canonical_dest = destination.canonicalize()
                .map_err(|e| anyhow!("Failed to canonicalize destination: {}", e))?;
            
            if !canonical.starts_with(&canonical_dest) {
                return Err(anyhow!(
                    "Path escapes destination directory: {:?}",
                    entry_path
                ));
            }
            Ok(final_path)
        }
        Err(_) => {
            // Create parent directories if they do not exist, then re-attempt canonicalization
            if let Some(parent) = final_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| anyhow!("Failed to create parent directories: {}", e))?;
                }
                // Now try to canonicalize the final path again
                match final_path.canonicalize() {
                    Ok(canonical) => {
                        let canonical_dest = destination.canonicalize()
                            .map_err(|e| anyhow!("Failed to canonicalize destination: {}", e))?;
                        if !canonical.starts_with(&canonical_dest) {
                            return Err(anyhow!(
                                "Path escapes destination directory: {:?}",
                                entry_path
                            ));
                        }
                        Ok(final_path)
                    }
                    Err(e) => {
                        return Err(anyhow!(
                            "Failed to canonicalize path after creating parent directories: {:?}, error: {}",
                            entry_path,
                            e
                        ));
                    }
                }
            } else {
                return Err(anyhow!(
                    "Final path has no parent: {:?}",
                    entry_path
                ));
            }
        }
    }
}

/// Secure tar extraction with path validation and size limits
pub async fn extract_tar_gz_secure(
    archive_path: &Path,
    destination: &Path,
    max_files: usize,
    max_total_size: u64,
) -> Result<()> {
    // Ensure destination exists
    fs::create_dir_all(destination).await?;
    
    let archive_path = archive_path.to_path_buf();
    let destination = destination.to_path_buf();
    
    // Use spawn_blocking to run sync tar extraction in a background thread
    tokio::task::spawn_blocking(move || -> Result<()> {
        let file = std::fs::File::open(&archive_path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        
        let mut file_count = 0;
        let mut total_size = 0u64;
        
        for entry in archive.entries()? {
            let mut entry = entry?;
            
            // Check file count limit
            file_count += 1;
            if file_count > max_files {
                return Err(anyhow!(
                    "Archive contains too many files (>{} files)",
                    max_files
                ));
            }
            
            // Check size limit
            let size = entry.header().size()?;
            total_size = total_size.saturating_add(size);
            if total_size > max_total_size {
                return Err(anyhow!(
                    "Archive total size exceeds limit ({} bytes)",
                    max_total_size
                ));
            }
            
            // Validate the entry path
            let entry_path = entry.path()?;
            let safe_path = validate_archive_path(&entry_path, &destination)?;
            
            // Create parent directories if needed
            if let Some(parent) = safe_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // Extract the file
            if entry.header().entry_type().is_file() {
                entry.unpack(&safe_path)?;
            } else if entry.header().entry_type().is_dir() {
                std::fs::create_dir_all(&safe_path)?;
            }
            // Skip other entry types (symlinks, etc.) for security
        }
        
        Ok(())
    }).await?
}

/// Secure zip extraction with path validation and size limits  
pub async fn extract_zip_secure(
    archive_path: &Path,
    destination: &Path,
    max_files: usize,
    max_total_size: u64,
) -> Result<()> {
    // Ensure destination exists
    fs::create_dir_all(destination).await?;
    
    let archive_path = archive_path.to_path_buf();
    let destination = destination.to_path_buf();
    
    // Use spawn_blocking to run sync zip extraction in a background thread
    tokio::task::spawn_blocking(move || -> Result<()> {
        let file = std::fs::File::open(&archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        let file_count = archive.len();
        if file_count > max_files {
            return Err(anyhow!(
                "Archive contains too many files (>{} files)",
                max_files
            ));
        }
        
        let mut total_size = 0u64;
        
        for i in 0..file_count {
            let mut file = archive.by_index(i)?;
            
            // Check size limit
            let size = file.size();
            total_size = total_size.saturating_add(size);
            if total_size > max_total_size {
                return Err(anyhow!(
                    "Archive total size exceeds limit ({} bytes)",
                    max_total_size
                ));
            }
            
            // Validate the entry path
            let entry_path = PathBuf::from(file.name());
            let safe_path = validate_archive_path(&entry_path, &destination)?;
            
            if file.is_dir() {
                std::fs::create_dir_all(&safe_path)?;
            } else {
                // Create parent directories if needed
                if let Some(parent) = safe_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                
                // Extract the file
                let mut outfile = std::fs::File::create(&safe_path)?;
                std::io::copy(&mut file, &mut outfile)?;
                
                // Preserve permissions on Unix systems
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        let permissions = std::fs::Permissions::from_mode(mode);
                        std::fs::set_permissions(&safe_path, permissions)?;
                    }
                }
            }
        }
        
        Ok(())
    }).await?
}