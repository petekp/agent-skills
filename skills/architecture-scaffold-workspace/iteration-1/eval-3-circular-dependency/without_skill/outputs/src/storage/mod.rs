//! Persistence layer.
//! Layer 2: Depends only on domain types.

use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use crate::domain::{Task, Project, User, TaskFlowError, UserId, ProjectId};

/// Trait defining the persistence contract.
pub trait StorageBackend {
    fn save_tasks(&self, tasks: &HashMap<Uuid, Task>) -> Result<(), TaskFlowError>;
    fn load_tasks(&self) -> Result<HashMap<Uuid, Task>, TaskFlowError>;
    fn save_projects(&self, projects: &HashMap<ProjectId, Project>) -> Result<(), TaskFlowError>;
    fn load_projects(&self) -> Result<HashMap<ProjectId, Project>, TaskFlowError>;
    fn save_users(&self, users: &HashMap<UserId, User>) -> Result<(), TaskFlowError>;
    fn load_users(&self) -> Result<HashMap<UserId, User>, TaskFlowError>;
}

/// Stub implementation for JSON file-based storage.
pub struct JsonFileStorage {
    pub storage_path: PathBuf,
}

impl StorageBackend for JsonFileStorage {
    fn save_tasks(&self, _tasks: &HashMap<Uuid, Task>) -> Result<(), TaskFlowError> {
        todo!("StorageBackend::save_tasks")
    }

    fn load_tasks(&self) -> Result<HashMap<Uuid, Task>, TaskFlowError> {
        todo!("StorageBackend::load_tasks")
    }

    fn save_projects(&self, _projects: &HashMap<ProjectId, Project>) -> Result<(), TaskFlowError> {
        todo!("StorageBackend::save_projects")
    }

    fn load_projects(&self) -> Result<HashMap<ProjectId, Project>, TaskFlowError> {
        todo!("StorageBackend::load_projects")
    }

    fn save_users(&self, _users: &HashMap<UserId, User>) -> Result<(), TaskFlowError> {
        todo!("StorageBackend::save_users")
    }

    fn load_users(&self) -> Result<HashMap<UserId, User>, TaskFlowError> {
        todo!("StorageBackend::load_users")
    }
}
