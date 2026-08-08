use crate::domain::error::Result;
use crate::domain::project::Project;
use crate::storage::project_store::ProjectRepository;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use walkdir::WalkDir;

pub struct ProjectService {
    repo: Arc<Mutex<dyn ProjectRepository>>,
}

impl ProjectService {
    pub fn new(repo: Arc<Mutex<dyn ProjectRepository>>) -> Self {
        Self { repo }
    }

    pub fn detect_repo_root(&self, starting_dir: &Path) -> Result<Option<PathBuf>> {
        let mut curr = starting_dir.to_path_buf();
        loop {
            let git_dir = curr.join(".git");
            if git_dir.exists() {
                return Ok(Some(curr));
            }
            if !curr.pop() {
                break;
            }
        }
        Ok(None)
    }

    pub async fn map_project(&self, raw_path: &Path, account_id: &str) -> Result<Project> {
        let repo_root = self
            .detect_repo_root(raw_path)?
            .unwrap_or_else(|| raw_path.to_path_buf());

        let project_name = repo_root
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "project".to_string());

        let project = Project::new(repo_root.clone(), project_name, account_id);

        let mut guard = self.repo.lock().await;
        guard.map_project(project.clone())?;
        Ok(project)
    }

    pub async fn unmap_project(&self, raw_path: &Path) -> Result<Project> {
        let repo_root = self
            .detect_repo_root(raw_path)?
            .unwrap_or_else(|| raw_path.to_path_buf());

        let mut guard = self.repo.lock().await;
        guard.unmap_project(&repo_root)
    }

    pub async fn find_project(&self, raw_path: &Path) -> Result<Option<Project>> {
        let repo_root = self
            .detect_repo_root(raw_path)?
            .unwrap_or_else(|| raw_path.to_path_buf());

        let guard = self.repo.lock().await;
        guard.find_by_path(&repo_root)
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let guard = self.repo.lock().await;
        guard.list_all()
    }

    pub fn scan_directory_for_git_repos(&self, root_dir: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        for entry in WalkDir::new(root_dir)
            .into_iter()
            .filter_entry(|e| {
                let fname = e.file_name().to_string_lossy();
                // Skip hidden folders other than .git or node_modules/target
                !(fname.starts_with('.') && fname != ".git")
                    && fname != "node_modules"
                    && fname != "target"
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() && entry.file_name() == ".git" {
                if let Some(parent) = entry.path().parent() {
                    found.push(parent.to_path_buf());
                }
            }
        }
        found
    }
}
