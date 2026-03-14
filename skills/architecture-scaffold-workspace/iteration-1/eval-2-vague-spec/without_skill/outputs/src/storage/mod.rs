//! Storage abstraction layer.
//!
//! Defines the `Storage` trait that decouples persistence from business logic.
//! The `json_file` module provides the default filesystem-based implementation.

mod json_file;

pub use json_file::JsonFileStorage;

use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{Project, ProjectId, Task, TaskFlowError, User, UserId};

/// Trait for persisting and loading TaskFlow data.
///
/// Implementations must be Send + Sync to support shared ownership
/// (e.g., behind an Arc in the facade).
pub trait Storage: Send + Sync {
    fn load_tasks(&self) -> Result<HashMap<Uuid, Task>, TaskFlowError>;
    fn load_projects(&self) -> Result<HashMap<ProjectId, Project>, TaskFlowError>;
    fn load_users(&self) -> Result<HashMap<UserId, User>, TaskFlowError>;

    fn save_tasks(&self, tasks: &HashMap<Uuid, Task>) -> Result<(), TaskFlowError>;
    fn save_projects(&self, projects: &HashMap<ProjectId, Project>) -> Result<(), TaskFlowError>;
    fn save_users(&self, users: &HashMap<UserId, User>) -> Result<(), TaskFlowError>;
}
