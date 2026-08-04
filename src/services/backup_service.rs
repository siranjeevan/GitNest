use crate::config::ConfigManager;
use crate::domain::error::{GitNestError, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

pub struct BackupService {
    config_mgr: ConfigManager,
}

impl BackupService {
    pub fn new(config_mgr: ConfigManager) -> Self {
        Self { config_mgr }
    }

    pub fn export_backup(&self, output_zip_path: &Path) -> Result<()> {
        let file = File::create(output_zip_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        let root_dir = self.config_mgr.root_dir();
        let logs_dir = self.config_mgr.logs_dir();

        for entry in WalkDir::new(root_dir) {
            let entry = entry.map_err(|e| GitNestError::Io(e.into()))?;
            let path = entry.path();

            if path.starts_with(&logs_dir) {
                continue;
            }

            let name = path
                .strip_prefix(root_dir)
                .map_err(|_| GitNestError::InvalidPath("Failed to strip path prefix".to_string()))?;

            if path.is_file() {
                zip.start_file(name.to_string_lossy(), options)?;
                let mut f = File::open(path)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            } else if !name.as_os_str().is_empty() {
                zip.add_directory(name.to_string_lossy(), options)?;
            }
        }

        zip.finish()?;
        Ok(())
    }

    pub fn import_backup(&self, input_zip_path: &Path) -> Result<()> {
        let file = File::open(input_zip_path)?;
        let mut archive = ZipArchive::new(file).map_err(|e| {
            GitNestError::InvalidPath(format!("Invalid zip archive: {}", e))
        })?;

        let root_dir = self.config_mgr.root_dir();
        if !root_dir.exists() {
            std::fs::create_dir_all(root_dir)?;
        }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                GitNestError::InvalidPath(format!("Archive entry error: {}", e))
            })?;

            let outpath = match file.enclosed_name() {
                Some(path) => root_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if outpath.starts_with(self.config_mgr.ssh_dir()) && outpath.is_file() {
                    if !outpath.to_string_lossy().ends_with(".pub") {
                        std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(0o600))?;
                    }
                }
            }
        }

        Ok(())
    }
}
