//! Task operations module.
//! Layer 3: Depends on `domain`, `project_service`.
//!
//! CIRCULAR DEPENDENCY: This module depends on `project_service` (to validate
//! that a project exists and check membership via `ProjectService::get_project`),
//! while `project_service` depends back on this module (to query tasks for
//! project stats via `TaskService::filter_by_project`).

use uuid::Uuid;

use crate::domain::{Task, TaskFlowError, TaskStatus, UserId, ProjectId};
use crate::project_service::ProjectService;

/// Trait for task operations.
///
/// Methods that need project validation accept a `&dyn ProjectService` parameter
/// to look up the project and check membership. This creates the circular
/// dependency described in the assessment.
pub trait TaskService {
    /// Create a new task in a project.
    /// Uses `ProjectService::get_project` to validate project exists and check membership.
    fn create_task(
        &mut self,
        title: String,
        project_id: ProjectId,
        creator: &UserId,
        project_svc: &dyn ProjectService,
    ) -> Result<Uuid, TaskFlowError>;

    /// Update a task's status, enforcing valid transitions.
    /// Uses `ProjectService::get_project` to check actor membership.
    fn update_status(
        &mut self,
        task_id: Uuid,
        new_status: TaskStatus,
        actor: &UserId,
        project_svc: &dyn ProjectService,
    ) -> Result<(), TaskFlowError>;

    /// Assign a task to a user.
    /// Uses `ProjectService::get_project` to check both actor and assignee membership.
    fn assign(
        &mut self,
        task_id: Uuid,
        assignee: UserId,
        actor: &UserId,
        project_svc: &dyn ProjectService,
    ) -> Result<(), TaskFlowError>;

    /// Delete a task (and its subtasks). Requires project owner.
    /// Uses `ProjectService::get_project` to verify ownership.
    fn delete(
        &mut self,
        task_id: Uuid,
        actor: &UserId,
        project_svc: &dyn ProjectService,
    ) -> Result<(), TaskFlowError>;

    /// Get a task by ID.
    fn get_task(&self, task_id: Uuid) -> Result<&Task, TaskFlowError>;

    /// Filter tasks belonging to a specific project.
    /// Called by `ProjectService::get_stats` to compute task counts and completion rates.
    fn filter_by_project(&self, project_id: &ProjectId) -> Vec<&Task>;
}
