use crate::models::FileEntry;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum FileServiceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path error: {0}")]
    Path(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub type FileServiceResult<T> = Result<T, FileServiceError>;

pub struct FileService;

impl FileService {
    pub fn new() -> Self {
        Self
    }

    // ============== Sync Methods (Legacy) ==============

    pub fn read_file(&self, path: &Path) -> FileServiceResult<String> {
        if !path.exists() {
            return Err(FileServiceError::NotFound(path.display().to_string()));
        }
        fs::read_to_string(path).map_err(FileServiceError::Io)
    }

    pub fn write_file(&self, path: &Path, content: &str) -> FileServiceResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content).map_err(FileServiceError::Io)
    }

    pub fn delete_file(&self, path: &Path) -> FileServiceResult<()> {
        if !path.exists() {
            return Err(FileServiceError::NotFound(path.display().to_string()));
        }
        let metadata = fs::metadata(path)?;
        if metadata.is_dir() {
            fs::remove_dir_all(path).map_err(FileServiceError::Io)
        } else {
            fs::remove_file(path).map_err(FileServiceError::Io)
        }
    }

    pub fn create_directory(&self, path: &Path) -> FileServiceResult<()> {
        fs::create_dir_all(path).map_err(FileServiceError::Io)
    }

    pub fn list_directory(&self, path: &Path) -> FileServiceResult<Vec<FileEntry>> {
        if !path.exists() {
            return Err(FileServiceError::NotFound(path.display().to_string()));
        }
        if !path.is_dir() {
            return Err(FileServiceError::Path(format!("{} is not a directory", path.display())));
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry.map_err(FileServiceError::Io)?;
            let metadata = entry.metadata().map_err(FileServiceError::Io)?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path();

            let modified = metadata.modified().ok().map(|t| {
                chrono_lite_format(t)
            });

            entries.push(FileEntry {
                name: file_name,
                path: file_path.display().to_string(),
                is_dir: metadata.is_dir(),
                size: if metadata.is_file() { Some(metadata.len()) } else { None },
                modified,
            });
        }

        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        Ok(entries)
    }

    pub fn copy_file(&self, src: &Path, dst: &Path) -> FileServiceResult<()> {
        if !src.exists() {
            return Err(FileServiceError::NotFound(src.display().to_string()));
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        if src.is_dir() {
            self.copy_dir_recursive_sync(src, dst)
        } else {
            fs::copy(src, dst).map(|_| ()).map_err(FileServiceError::Io)
        }
    }

    pub fn move_file(&self, src: &Path, dst: &Path) -> FileServiceResult<()> {
        if !src.exists() {
            return Err(FileServiceError::NotFound(src.display().to_string()));
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(src, dst).map_err(FileServiceError::Io)
    }

    pub fn walk_directory(&self, path: &Path) -> FileServiceResult<Vec<FileEntry>> {
        if !path.exists() {
            return Err(FileServiceError::NotFound(path.display().to_string()));
        }

        let mut entries = Vec::new();
        for entry in WalkDir::new(path).min_depth(1) {
            let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            let metadata = entry.metadata().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            let file_name = entry.file_name().to_string_lossy().to_string();

            let modified = metadata.modified().ok().map(|t| {
                chrono_lite_format(t)
            });

            entries.push(FileEntry {
                name: file_name,
                path: entry.path().display().to_string(),
                is_dir: metadata.is_dir(),
                size: if metadata.is_file() { Some(metadata.len()) } else { None },
                modified,
            });
        }
        Ok(entries)
    }

    pub fn get_file_info(&self, path: &Path) -> FileServiceResult<FileEntry> {
        if !path.exists() {
            return Err(FileServiceError::NotFound(path.display().to_string()));
        }
        let metadata = fs::metadata(path)?;
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let modified = metadata.modified().ok().map(|t| {
            chrono_lite_format(t)
        });

        Ok(FileEntry {
            name: file_name,
            path: path.display().to_string(),
            is_dir: metadata.is_dir(),
            size: if metadata.is_file() { Some(metadata.len()) } else { None },
            modified,
        })
    }

    pub fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    pub fn is_symlink(&self, path: &Path) -> bool {
        fs::symlink_metadata(path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }

    pub fn read_symlink(&self, path: &Path) -> FileServiceResult<PathBuf> {
        fs::read_link(path).map_err(FileServiceError::Io)
    }

    pub fn create_symlink(&self, target: &Path, link: &Path) -> FileServiceResult<()> {
        #[cfg(windows)]
        {
            if target.is_dir() {
                std::os::windows::fs::symlink_dir(target, link)?;
            } else {
                std::os::windows::fs::symlink_file(target, link)?;
            }
            Ok(())
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link)
        }
        .map_err(FileServiceError::Io)
    }

    // ============== Async Methods ==============

    /// Async file read
    pub async fn read_file_async(&self, path: &Path) -> FileServiceResult<String> {
        tokio::fs::read_to_string(path).await.map_err(FileServiceError::Io)
    }

    /// Async file write
    pub async fn write_file_async(&self, path: &Path, content: &str) -> FileServiceResult<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, content).await.map_err(FileServiceError::Io)
    }

    /// Async directory listing
    pub async fn list_directory_async(&self, path: &Path) -> FileServiceResult<Vec<FileEntry>> {
        if !path.exists() {
            return Err(FileServiceError::NotFound(path.display().to_string()));
        }
        if !path.is_dir() {
            return Err(FileServiceError::Path(format!("{} is not a directory", path.display())));
        }

        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path).await?;

        while let Some(entry) = dir.next_entry().await.map_err(FileServiceError::Io)? {
            let metadata = entry.metadata().await.map_err(FileServiceError::Io)?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path();

            let modified = metadata.modified().ok().map(|t| {
                chrono_lite_format(t)
            });

            entries.push(FileEntry {
                name: file_name,
                path: file_path.display().to_string(),
                is_dir: metadata.is_dir(),
                size: if metadata.is_file() { Some(metadata.len()) } else { None },
                modified,
            });
        }

        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        Ok(entries)
    }

    /// Async recursive directory copy with optional progress callback
    pub async fn copy_dir_recursive_async(
        &self,
        src: &Path,
        dst: &Path,
        progress_callback: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> FileServiceResult<u64> {
        if !src.is_dir() {
            return Err(FileServiceError::Path(format!("{} is not a directory", src.display())));
        }

        tokio::fs::create_dir_all(dst).await?;

        let mut total_size: u64 = 0;
        let mut entries: Vec<(PathBuf, PathBuf)> = Vec::new();

        // First pass: collect all files
        let mut stack = vec![src.to_path_buf()];
        while let Some(current) = stack.pop() {
            let mut dir = tokio::fs::read_dir(&current).await.map_err(FileServiceError::Io)?;
            while let Some(entry) = dir.next_entry().await.map_err(FileServiceError::Io)? {
                let entry_path = entry.path();
                let relative = entry_path.strip_prefix(src).unwrap_or(&entry_path);
                let dest = dst.join(relative);

                if entry_path.is_dir() {
                    stack.push(entry_path);
                } else {
                    entries.push((entry_path, dest));
                }
            }
        }

        // Second pass: copy files
        let total_files = entries.len() as u64;
        for (idx, (src_path, dst_path)) in entries.into_iter().enumerate() {
            if let Some(parent) = dst_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::copy(&src_path, &dst_path).await.map_err(FileServiceError::Io)?;

            let file_size = tokio::fs::metadata(&src_path).await.map_err(FileServiceError::Io)?.len();
            total_size += file_size;

            if let Some(ref callback) = progress_callback {
                callback(idx as u64 + 1, total_files);
            }
        }

        Ok(total_size)
    }

    /// Async file copy
    pub async fn copy_file_async(&self, src: &Path, dst: &Path) -> FileServiceResult<u64> {
        if !src.exists() {
            return Err(FileServiceError::NotFound(src.display().to_string()));
        }
        if let Some(parent) = dst.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        if src.is_dir() {
            self.copy_dir_recursive_async(src, dst, None).await
        } else {
            let size = tokio::fs::copy(src, dst).await.map_err(FileServiceError::Io)?;
            Ok(size)
        }
    }

    /// Async directory walk
    pub async fn walk_directory_async(&self, path: &Path) -> FileServiceResult<Vec<FileEntry>> {
        if !path.exists() {
            return Err(FileServiceError::NotFound(path.display().to_string()));
        }

        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path).await.map_err(FileServiceError::Io)?;

        while let Some(entry) = dir.next_entry().await.map_err(FileServiceError::Io)? {
            let metadata = entry.metadata().await.map_err(FileServiceError::Io)?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path();

            let modified = metadata.modified().ok().map(|t| {
                chrono_lite_format(t)
            });

            entries.push(FileEntry {
                name: file_name,
                path: file_path.display().to_string(),
                is_dir: metadata.is_dir(),
                size: if metadata.is_file() { Some(metadata.len()) } else { None },
                modified,
            });
        }

        Ok(entries)
    }

    /// Async file info
    pub async fn get_file_info_async(&self, path: &Path) -> FileServiceResult<FileEntry> {
        if !path.exists() {
            return Err(FileServiceError::NotFound(path.display().to_string()));
        }
        let metadata = tokio::fs::metadata(path).await.map_err(FileServiceError::Io)?;
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let modified = metadata.modified().ok().map(|t| {
            chrono_lite_format(t)
        });

        Ok(FileEntry {
            name: file_name,
            path: path.display().to_string(),
            is_dir: metadata.is_dir(),
            size: if metadata.is_file() { Some(metadata.len()) } else { None },
            modified,
        })
    }

    // ============== Internal Sync Helpers ==============

    fn copy_dir_recursive_sync(&self, src: &Path, dst: &Path) -> FileServiceResult<()> {
        if !src.is_dir() {
            return Err(FileServiceError::Path(format!("{} is not a directory", src.display())));
        }

        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry.map_err(FileServiceError::Io)?;
            let new_dst = dst.join(entry.file_name());
            let entry_path = entry.path();
            if entry_path.is_dir() {
                self.copy_dir_recursive_sync(&entry_path, &new_dst)?;
            } else {
                fs::copy(&entry_path, &new_dst).map_err(FileServiceError::Io)?;
            }
        }
        Ok(())
    }
}

impl Default for FileService {
    fn default() -> Self {
        Self::new()
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> FileServiceResult<()> {
    if !src.is_dir() {
        return Err(FileServiceError::Path(format!("{} is not a directory", src.display())));
    }

    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry.map_err(FileServiceError::Io)?;
        let new_dst = dst.join(entry.file_name());
        let entry_path = entry.path();
        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &new_dst)?;
        } else {
            fs::copy(&entry_path, &new_dst).map_err(FileServiceError::Io)?;
        }
    }
    Ok(())
}

fn chrono_lite_format(time: std::time::SystemTime) -> String {
    // Use the `chrono` crate for accurate date math. The previous
    // hand-rolled implementation accumulated a ~1 day error every 4
    // years because of the simplified leap-year handling.
    chrono::DateTime::<chrono::Utc>::from(time).to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_service_read_write() {
        let service = FileService::new();
        let temp_dir = std::env::temp_dir().join("forge-test");
        let test_file = temp_dir.join("test.txt");

        service.create_directory(&temp_dir).unwrap();
        service.write_file(&test_file, "Hello, World!").unwrap();

        let content = service.read_file(&test_file).unwrap();
        assert_eq!(content, "Hello, World!");

        service.delete_file(&temp_dir).unwrap();
    }
}
