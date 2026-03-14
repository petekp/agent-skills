use crate::domain::{Task, TaskFilter, ProjectStats, TaskFlowError, ProjectId};

/// Read-only task query trait. Breaks the circular dependency between
/// task_service and project_service.
pub trait TaskQuery {
    fn filter_tasks(&self, filter: &TaskFilter) -> Vec<Task>;
    fn filter_by_project(&self, project_id: &ProjectId) -> Vec<Task>;
    fn get_project_stats(&self, project_id: &ProjectId) -> Result<ProjectStats, TaskFlowError>;
}

/// Stub implementation backed by an in-memory task store.
pub struct InMemoryTaskQuery;

impl TaskQuery for InMemoryTaskQuery {
    fn filter_tasks(&self, _filter: &TaskFilter) -> Vec<Task> {
        todo!("TaskQuery::filter_tasks")
    }
    fn filter_by_project(&self, _project_id: &ProjectId) -> Vec<Task> {
        todo!("TaskQuery::filter_by_project")
    }
    fn get_project_stats(&self, _project_id: &ProjectId) -> Result<ProjectStats, TaskFlowError> {
        todo!("TaskQuery::get_project_stats")
    }
}
