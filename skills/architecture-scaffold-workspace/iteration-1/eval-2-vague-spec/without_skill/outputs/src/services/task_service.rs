use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::analytics::AnalyticsCollector;
use crate::models::*;
use crate::notifications::NotificationSink;

/// Manages task CRUD, status transitions, assignment, and filtering.
///
/// Does NOT own persistence -- the facade coordinates saving.
/// Receives notification and analytics dependencies via constructor injection.
pub struct TaskService {
    tasks: HashMap<Uuid, Task>,
    notifications: Arc<dyn NotificationSink>,
    analytics: Arc<dyn AnalyticsCollector>,
    filter_cache: HashMap<String, Vec<Uuid>>,
}

impl TaskService {
    pub fn new(
        tasks: HashMap<Uuid, Task>,
        notifications: Arc<dyn NotificationSink>,
        analytics: Arc<dyn AnalyticsCollector>,
    ) -> Self {
        Self {
            tasks,
            notifications,
            analytics,
            filter_cache: HashMap::new(),
        }
    }

    pub fn tasks(&self) -> &HashMap<Uuid, Task> {
        &self.tasks
    }

    /// Create a new task in the given project.
    ///
    /// Caller must verify project exists and creator is a member before calling.
    pub fn create_task(
        &mut self,
        title: String,
        project_id: ProjectId,
        creator: &UserId,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let task = Task {
            id,
            title,
            description: None,
            status: TaskStatus::Backlog,
            priority: Priority::None,
            assignee: None,
            project_id: project_id.clone(),
            labels: Vec::new(),
            created_at: now,
            updated_at: now,
            due_date: None,
            parent_id: None,
            subtask_ids: Vec::new(),
        };
        self.tasks.insert(id, task);
        self.invalidate_cache(&project_id);
        self.analytics.track("task_created", Some(creator), Some(&project_id));
        id
    }

    /// Transition a task's status, enforcing the allowed workflow.
    pub fn update_status(
        &mut self,
        task_id: Uuid,
        new_status: TaskStatus,
        actor: &UserId,
    ) -> Result<(), TaskFlowError> {
        let task = self.tasks.get(&task_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("task: {}", task_id)))?;

        if !Self::is_valid_transition(task.status, new_status) {
            return Err(TaskFlowError::InvalidTransition {
                from: task.status,
                to: new_status,
            });
        }

        let project_id = task.project_id.clone();
        let assignee = task.assignee.clone();

        let task = self.tasks.get_mut(&task_id).unwrap();
        task.status = new_status;
        task.updated_at = Utc::now();

        if let Some(ref assignee_id) = assignee {
            if assignee_id != actor {
                self.notifications.send(&Notification {
                    kind: NotificationKind::TaskStatusChanged,
                    user_id: assignee_id.clone(),
                    message: format!("Task status changed to {:?}", new_status),
                    timestamp: Utc::now(),
                });
            }
        }

        self.invalidate_cache(&project_id);
        self.analytics.track("task_status_changed", Some(actor), Some(&project_id));
        Ok(())
    }

    /// Assign a task to a user. Caller must verify membership.
    pub fn assign(
        &mut self,
        task_id: Uuid,
        assignee: UserId,
        actor: &UserId,
    ) -> Result<(), TaskFlowError> {
        let task = self.tasks.get(&task_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("task: {}", task_id)))?;

        let project_id = task.project_id.clone();
        let title = task.title.clone();

        let task = self.tasks.get_mut(&task_id).unwrap();
        task.assignee = Some(assignee.clone());
        task.updated_at = Utc::now();

        self.notifications.send(&Notification {
            kind: NotificationKind::TaskAssigned,
            user_id: assignee.clone(),
            message: format!("You were assigned to task: {}", title),
            timestamp: Utc::now(),
        });

        self.invalidate_cache(&project_id);
        self.analytics.track("task_assigned", Some(actor), Some(&project_id));
        Ok(())
    }

    pub fn get(&self, task_id: Uuid) -> Result<&Task, TaskFlowError> {
        self.tasks.get(&task_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("task: {}", task_id)))
    }

    pub fn delete(&mut self, task_id: Uuid, actor: &UserId) -> Result<(), TaskFlowError> {
        let task = self.tasks.get(&task_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("task: {}", task_id)))?;

        let project_id = task.project_id.clone();
        let parent_id = task.parent_id;
        let subtask_ids = task.subtask_ids.clone();

        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.tasks.get_mut(&parent_id) {
                parent.subtask_ids.retain(|id| *id != task_id);
            }
        }

        self.tasks.remove(&task_id);
        for sub_id in subtask_ids {
            let _ = self.delete(sub_id, actor);
        }

        self.invalidate_cache(&project_id);
        self.analytics.track("task_deleted", Some(actor), Some(&project_id));
        Ok(())
    }

    /// Filter tasks by the given criteria. Results are cached until invalidation.
    pub fn filter(&mut self, filter: &TaskFilter) -> Vec<&Task> {
        let cache_key = format!("{:?}", filter);
        if let Some(cached_ids) = self.filter_cache.get(&cache_key) {
            return cached_ids.iter().filter_map(|id| self.tasks.get(id)).collect();
        }

        let mut results: Vec<&Task> = self.tasks.values()
            .filter(|task| Self::matches_filter(task, filter))
            .collect();

        results.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.created_at.cmp(&b.created_at)));

        let ids: Vec<Uuid> = results.iter().map(|t| t.id).collect();
        self.filter_cache.insert(cache_key, ids);
        results
    }

    // --- Private helpers ---

    fn is_valid_transition(from: TaskStatus, to: TaskStatus) -> bool {
        matches!(
            (from, to),
            (TaskStatus::Backlog, TaskStatus::Todo)
                | (TaskStatus::Todo, TaskStatus::InProgress)
                | (TaskStatus::InProgress, TaskStatus::InReview)
                | (TaskStatus::InReview, TaskStatus::Done)
                | (TaskStatus::InReview, TaskStatus::InProgress)
                | (_, TaskStatus::Cancelled)
        )
    }

    fn matches_filter(task: &Task, filter: &TaskFilter) -> bool {
        if let Some(ref statuses) = filter.status {
            if !statuses.contains(&task.status) { return false; }
        }
        if let Some(ref priorities) = filter.priority {
            if !priorities.contains(&task.priority) { return false; }
        }
        if let Some(ref assignee) = filter.assignee {
            if task.assignee.as_ref() != Some(assignee) { return false; }
        }
        if let Some(ref project_id) = filter.project_id {
            if &task.project_id != project_id { return false; }
        }
        if let Some(ref labels) = filter.labels {
            if !labels.iter().any(|l| task.labels.contains(l)) { return false; }
        }
        if let Some(ref due_before) = filter.due_before {
            if task.due_date.as_ref().map_or(true, |d| d > due_before) { return false; }
        }
        if let Some(ref due_after) = filter.due_after {
            if task.due_date.as_ref().map_or(true, |d| d < due_after) { return false; }
        }
        if let Some(ref text) = filter.search_text {
            let lower = text.to_lowercase();
            let title_match = task.title.to_lowercase().contains(&lower);
            let desc_match = task.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&lower));
            if !title_match && !desc_match { return false; }
        }
        true
    }

    fn invalidate_cache(&mut self, _project_id: &ProjectId) {
        self.filter_cache.clear();
    }
}
