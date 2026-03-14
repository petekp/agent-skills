//! Persistence layer.
//! Layer 2: Depends only on domain types.

use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use crate::domain::{Project, ProjectId, Task, User, UserId};

/// Persistence error.
#[derive(Debug)]
pub struct StorageError {
    pub message: String,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "storage error: {}", self.message)
    }
}

impl std::error::Error for StorageError {}

/// Persistence adapter trait.
pub trait StorageBackend {
    fn save_tasks(&self, tasks: &HashMap<Uuid, Task>) -> Result<(), StorageError>;
    fn load_tasks(&self) -> Result<HashMap<Uuid, Task>, StorageError>;
    fn save_projects(&self, projects: &HashMap<ProjectId, Project>) -> Result<(), StorageError>;
    fn load_projects(&self) -> Result<HashMap<ProjectId, Project>, StorageError>;
    fn save_users(&self, users: &HashMap<UserId, User>) -> Result<(), StorageError>;
    fn load_users(&self) -> Result<HashMap<UserId, User>, StorageError>;
}

/// JSON file-based storage stub.
pub struct JsonFileStorage {
    pub storage_path: PathBuf,
}

impl StorageBackend for JsonFileStorage {
    fn save_tasks(&self, _tasks: &HashMap<Uuid, Task>) -> Result<(), StorageError> {
        todo!("JsonFileStorage::save_tasks")
    }
    fn load_tasks(&self) -> Result<HashMap<Uuid, Task>, StorageError> {
        todo!("JsonFileStorage::load_tasks")
    }
    fn save_projects(&self, _projects: &HashMap<ProjectId, Project>) -> Result<(), StorageError> {
        todo!("JsonFileStorage::save_projects")
    }
    fn load_projects(&self) -> Result<HashMap<ProjectId, Project>, StorageError> {
        todo!("JsonFileStorage::load_projects")
    }
    fn save_users(&self, _users: &HashMap<UserId, User>) -> Result<(), StorageError> {
        todo!("JsonFileStorage::save_users")
    }
    fn load_users(&self) -> Result<HashMap<UserId, User>, StorageError> {
        todo!("JsonFileStorage::load_users")
    }
}
