use crate::domain::error::{GitNestError, Result};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn atomic_write_json<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        GitNestError::InvalidPath(format!("Path {:?} has no parent directory", path))
    })?;

    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }

    let temp_file = tempfile::NamedTempFile::new_in(parent)?;
    let content = serde_json::to_string_pretty(data)?;

    let mut file = temp_file.as_file();
    file.write_all(content.as_bytes())?;
    file.sync_all()?;

    temp_file.persist(path).map_err(|e| {
        GitNestError::Io(std::io::Error::other(format!(
            "Failed to persist atomic file {:?}: {}",
            path, e
        )))
    })?;

    Ok(())
}
