use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use super::Storage;
use crate::models::{Project, ProjectId, Task, TaskFlowError, User, UserId};

/// Persists data as JSON files in a directory.
///
/// Files written:
/// - `tasks.json`
/// - `projects.json`
/// - `users.json`
pub struct JsonFileStorage {
    path: PathBuf,
}

impl JsonFileStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Storage for JsonFileStorage {
    fn load_tasks(&self) -> Result<HashMap<Uuid, Task>, TaskFlowError> {
        let data = fs::read_to_string(self.path.join("tasks.json"))
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        serde_json::from_str(&data).map_err(|e| TaskFlowError::StorageError(e.to_string()))
    }

    fn load_projects(&self) -> Result<HashMap<ProjectId, Project>, TaskFlowError> {
        let data = fs::read_to_string(self.path.join("projects.json"))
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        serde_json::from_str(&data).map_err(|e| TaskFlowError::StorageError(e.to_string()))
    }

    fn load_users(&self) -> Result<HashMap<UserId, User>, TaskFlowError> {
        let data = fs::read_to_string(self.path.join("users.json"))
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        serde_json::from_str(&data).map_err(|e| TaskFlowError::StorageError(e.to_string()))
    }

    fn save_tasks(&self, tasks: &HashMap<Uuid, Task>) -> Result<(), TaskFlowError> {
        fs::create_dir_all(&self.path)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        let json = serde_json::to_string_pretty(tasks)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        fs::write(self.path.join("tasks.json"), json)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))
    }

    fn save_projects(&self, projects: &HashMap<ProjectId, Project>) -> Result<(), TaskFlowError> {
        fs::create_dir_all(&self.path)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        let json = serde_json::to_string_pretty(projects)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        fs::write(self.path.join("projects.json"), json)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))
    }

    fn save_users(&self, users: &HashMap<UserId, User>) -> Result<(), TaskFlowError> {
        fs::create_dir_all(&self.path)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        let json = serde_json::to_string_pretty(users)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        fs::write(self.path.join("users.json"), json)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))
    }
}
