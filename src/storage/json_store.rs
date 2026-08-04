use std::io::Write;
use std::path::Path;
use crate::domain::error::{GitNestError, Result};
use serde::{de::DeserializeOwned, Serialize};

pub fn read_json_file<T: DeserializeOwned>(path: &Path) -> Result<T> {
    if !path.exists() {
        return Err(GitNestError::NotInitialized);
    }
    let content = std::fs::read_to_string(path)?;
    let data: T = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn write_json_file_atomic<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        GitNestError::InvalidPath(format!("Path has no parent directory: {:?}", path))
    })?;

    let mut temp_file = tempfile::NamedTempFile::new_in(parent)?;
    let json_bytes = serde_json::to_vec_pretty(data)?;
    
    temp_file.write_all(&json_bytes)?;
    temp_file.as_file().sync_all()?;
    
    temp_file.persist(path).map_err(|e| e.error)?;

    Ok(())
}
