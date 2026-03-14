//! Project operations module.
//! Layer 3: Depends on `domain`, `task_service`.
//!
//! CIRCULAR DEPENDENCY: This module depends on `task_service` (to query tasks
//! for computing project stats via `TaskService::filter_by_project`), while
//! `task_service` depends back on this module (to validate project existence
//! and membership via `ProjectService::get_project`).

use crate::domain::{Project, ProjectStats, TaskFlowError, UserId, ProjectId};
use crate::task_service::TaskService;

/// Trait for project operations.
pub trait ProjectService {
    /// Create a new project.
    fn create_project(
        &mut self,
        name: String,
        owner: UserId,
    ) -> Result<ProjectId, TaskFlowError>;

    /// Add a member to a project.
    fn add_member(
        &mut self,
        project_id: &ProjectId,
        user_id: UserId,
        actor: &UserId,
    ) -> Result<(), TaskFlowError>;

    /// Get a project by ID.
    /// Called by `TaskService` methods to validate project exists and check membership.
    fn get_project(&self, project_id: &ProjectId) -> Result<&Project, TaskFlowError>;

    /// Get project statistics (task counts, completion rates).
    /// Uses `TaskService::filter_by_project` to query tasks belonging to this project.
    fn get_stats(
        &self,
        project_id: &ProjectId,
        task_svc: &dyn TaskService,
    ) -> Result<ProjectStats, TaskFlowError>;
}
