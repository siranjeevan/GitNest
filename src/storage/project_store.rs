use crate::domain::error::{GitNestError, Result};
use crate::domain::project::Project;
use crate::storage::json_store::{read_json_file, write_json_file_atomic};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub trait ProjectRepository: Send + Sync {
    fn map_project(&mut self, project: Project) -> Result<()>;
    fn unmap_project(&mut self, target_path: &Path) -> Result<Project>;
    fn find_by_path(&self, target_path: &Path) -> Result<Option<Project>>;
    fn list_all(&self) -> Result<Vec<Project>>;
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectsWrapper {
    projects: Vec<Project>,
}

pub fn load_projects(path: &Path) -> Result<Vec<Project>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let wrapper: ProjectsWrapper = read_json_file(path)?;
    Ok(wrapper.projects)
}

pub fn save_projects(path: &Path, projects: &[Project]) -> Result<()> {
    let wrapper = ProjectsWrapper {
        projects: projects.to_vec(),
    };
    write_json_file_atomic(path, &wrapper)
}

pub struct JsonProjectRepository {
    storage_path: PathBuf,
}

impl JsonProjectRepository {
    pub fn new(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    fn normalize_path(path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }
}

use std::fs;

impl ProjectRepository for JsonProjectRepository {
    fn map_project(&mut self, mut project: Project) -> Result<()> {
        project.path = Self::normalize_path(&project.path);
        let mut projects = load_projects(&self.storage_path)?;

        if let Some(idx) = projects.iter().position(|p| p.path == project.path) {
            projects[idx] = project;
        } else {
            projects.push(project);
        }

        save_projects(&self.storage_path, &projects)
    }

    fn unmap_project(&mut self, target_path: &Path) -> Result<Project> {
        let normalized = Self::normalize_path(target_path);
        let mut projects = load_projects(&self.storage_path)?;

        let idx = projects
            .iter()
            .position(|p| p.path == normalized)
            .ok_or_else(|| GitNestError::ProjectNotFound(normalized.clone()))?;

        let removed = projects.remove(idx);
        save_projects(&self.storage_path, &projects)?;
        Ok(removed)
    }

    fn find_by_path(&self, target_path: &Path) -> Result<Option<Project>> {
        let normalized = Self::normalize_path(target_path);
        let projects = load_projects(&self.storage_path)?;
        Ok(projects.into_iter().find(|p| p.path == normalized))
    }

    fn list_all(&self) -> Result<Vec<Project>> {
        load_projects(&self.storage_path)
    }
}
