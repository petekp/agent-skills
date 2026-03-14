use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

// ============================================================
// Types — a mix of domain types, API types, and persistence types
// all living in one file
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub assignee: Option<UserId>,
    pub project_id: ProjectId,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub parent_id: Option<Uuid>,
    pub subtask_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    None,
    Low,
    Medium,
    High,
    Urgent,
}

pub type UserId = String;
pub type ProjectId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub owner: UserId,
    pub members: Vec<UserId>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
}

// ============================================================
// The God Object — TaskFlowEngine
// Handles: task CRUD, project management, user management,
// search/filtering, persistence, notifications, and analytics
// ============================================================

pub struct TaskFlowEngine {
    tasks: HashMap<Uuid, Task>,
    projects: HashMap<ProjectId, Project>,
    users: HashMap<UserId, User>,
    storage_path: PathBuf,
    notification_hooks: Vec<Box<dyn Fn(&Notification) + Send + Sync>>,
    analytics_buffer: Vec<AnalyticsEvent>,
    filter_cache: HashMap<String, Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub kind: NotificationKind,
    pub user_id: UserId,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationKind {
    TaskAssigned,
    TaskStatusChanged,
    TaskDueSoon,
    MentionedInComment,
    ProjectInvite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_type: String,
    pub user_id: Option<UserId>,
    pub project_id: Option<ProjectId>,
    pub metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilter {
    pub status: Option<Vec<TaskStatus>>,
    pub priority: Option<Vec<Priority>>,
    pub assignee: Option<UserId>,
    pub project_id: Option<ProjectId>,
    pub labels: Option<Vec<String>>,
    pub due_before: Option<DateTime<Utc>>,
    pub due_after: Option<DateTime<Utc>>,
    pub search_text: Option<String>,
}

#[derive(Debug)]
pub enum TaskFlowError {
    NotFound(String),
    DuplicateId(Uuid),
    InvalidTransition { from: TaskStatus, to: TaskStatus },
    PermissionDenied { user: UserId, action: String },
    StorageError(String),
    ValidationError(String),
}

impl std::fmt::Display for TaskFlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "not found: {}", msg),
            Self::DuplicateId(id) => write!(f, "duplicate id: {}", id),
            Self::InvalidTransition { from, to } => {
                write!(f, "invalid transition: {:?} -> {:?}", from, to)
            }
            Self::PermissionDenied { user, action } => {
                write!(f, "permission denied: {} cannot {}", user, action)
            }
            Self::StorageError(msg) => write!(f, "storage error: {}", msg),
            Self::ValidationError(msg) => write!(f, "validation error: {}", msg),
        }
    }
}

impl std::error::Error for TaskFlowError {}

impl TaskFlowEngine {
    // --- Construction & Persistence ---

    pub fn new(storage_path: PathBuf) -> Result<Self, TaskFlowError> {
        let engine = Self {
            tasks: HashMap::new(),
            projects: HashMap::new(),
            users: HashMap::new(),
            storage_path: storage_path.clone(),
            notification_hooks: Vec::new(),
            analytics_buffer: Vec::new(),
            filter_cache: HashMap::new(),
        };
        // Auto-load if data exists
        if storage_path.join("tasks.json").exists() {
            return Self::load_from_disk(storage_path);
        }
        Ok(engine)
    }

    pub fn load_from_disk(path: PathBuf) -> Result<Self, TaskFlowError> {
        let tasks_data = fs::read_to_string(path.join("tasks.json"))
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        let projects_data = fs::read_to_string(path.join("projects.json"))
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        let users_data = fs::read_to_string(path.join("users.json"))
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;

        let tasks: HashMap<Uuid, Task> = serde_json::from_str(&tasks_data)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        let projects: HashMap<ProjectId, Project> = serde_json::from_str(&projects_data)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        let users: HashMap<UserId, User> = serde_json::from_str(&users_data)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;

        Ok(Self {
            tasks,
            projects,
            users,
            storage_path: path,
            notification_hooks: Vec::new(),
            analytics_buffer: Vec::new(),
            filter_cache: HashMap::new(),
        })
    }

    pub fn save_to_disk(&self) -> Result<(), TaskFlowError> {
        fs::create_dir_all(&self.storage_path)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;

        let tasks_json = serde_json::to_string_pretty(&self.tasks)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        let projects_json = serde_json::to_string_pretty(&self.projects)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        let users_json = serde_json::to_string_pretty(&self.users)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;

        fs::write(self.storage_path.join("tasks.json"), tasks_json)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        fs::write(self.storage_path.join("projects.json"), projects_json)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;
        fs::write(self.storage_path.join("users.json"), users_json)
            .map_err(|e| TaskFlowError::StorageError(e.to_string()))?;

        self.track_event("data_saved", None, None);
        Ok(())
    }

    // --- Task CRUD ---

    pub fn create_task(
        &mut self,
        title: String,
        project_id: ProjectId,
        creator: &UserId,
    ) -> Result<Uuid, TaskFlowError> {
        if !self.projects.contains_key(&project_id) {
            return Err(TaskFlowError::NotFound(format!(
                "project: {}",
                project_id
            )));
        }
        if !self.is_member(creator, &project_id) {
            return Err(TaskFlowError::PermissionDenied {
                user: creator.clone(),
                action: "create task".to_string(),
            });
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        let task = Task {
            id,
            title: title.clone(),
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
        self.invalidate_filter_cache(&project_id);
        self.track_event("task_created", Some(creator), Some(&project_id));
        self.auto_save();
        Ok(id)
    }

    pub fn update_task_status(
        &mut self,
        task_id: Uuid,
        new_status: TaskStatus,
        actor: &UserId,
    ) -> Result<(), TaskFlowError> {
        let task = self
            .tasks
            .get(&task_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("task: {}", task_id)))?;

        if !self.is_valid_transition(task.status, new_status) {
            return Err(TaskFlowError::InvalidTransition {
                from: task.status,
                to: new_status,
            });
        }

        if !self.is_member(actor, &task.project_id) {
            return Err(TaskFlowError::PermissionDenied {
                user: actor.clone(),
                action: "update task status".to_string(),
            });
        }

        let project_id = task.project_id.clone();
        let assignee = task.assignee.clone();

        let task = self.tasks.get_mut(&task_id).unwrap();
        task.status = new_status;
        task.updated_at = Utc::now();

        // Notify assignee if status changed
        if let Some(ref assignee_id) = assignee {
            if assignee_id != actor {
                let notification = Notification {
                    kind: NotificationKind::TaskStatusChanged,
                    user_id: assignee_id.clone(),
                    message: format!("Task status changed to {:?}", new_status),
                    timestamp: Utc::now(),
                };
                self.send_notification(notification);
            }
        }

        self.invalidate_filter_cache(&project_id);
        self.track_event("task_status_changed", Some(actor), Some(&project_id));
        self.auto_save();
        Ok(())
    }

    pub fn assign_task(
        &mut self,
        task_id: Uuid,
        assignee: UserId,
        actor: &UserId,
    ) -> Result<(), TaskFlowError> {
        let task = self
            .tasks
            .get(&task_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("task: {}", task_id)))?;

        if !self.is_member(actor, &task.project_id) {
            return Err(TaskFlowError::PermissionDenied {
                user: actor.clone(),
                action: "assign task".to_string(),
            });
        }
        if !self.is_member(&assignee, &task.project_id) {
            return Err(TaskFlowError::ValidationError(format!(
                "{} is not a member of project {}",
                assignee, task.project_id
            )));
        }

        let project_id = task.project_id.clone();
        let title = task.title.clone();

        let task = self.tasks.get_mut(&task_id).unwrap();
        task.assignee = Some(assignee.clone());
        task.updated_at = Utc::now();

        // Notify the assignee
        let notification = Notification {
            kind: NotificationKind::TaskAssigned,
            user_id: assignee.clone(),
            message: format!("You were assigned to task: {}", title),
            timestamp: Utc::now(),
        };
        self.send_notification(notification);

        self.invalidate_filter_cache(&project_id);
        self.track_event("task_assigned", Some(actor), Some(&project_id));
        self.auto_save();
        Ok(())
    }

    pub fn get_task(&self, task_id: Uuid) -> Result<&Task, TaskFlowError> {
        self.tasks
            .get(&task_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("task: {}", task_id)))
    }

    pub fn delete_task(&mut self, task_id: Uuid, actor: &UserId) -> Result<(), TaskFlowError> {
        let task = self
            .tasks
            .get(&task_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("task: {}", task_id)))?;

        let project_id = task.project_id.clone();
        let parent_id = task.parent_id;
        let subtask_ids = task.subtask_ids.clone();

        if !self.is_project_owner(actor, &project_id) {
            return Err(TaskFlowError::PermissionDenied {
                user: actor.clone(),
                action: "delete task".to_string(),
            });
        }

        // Remove from parent's subtask list
        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.tasks.get_mut(&parent_id) {
                parent.subtask_ids.retain(|id| *id != task_id);
            }
        }

        // Recursively delete subtasks
        self.tasks.remove(&task_id);
        for sub_id in subtask_ids {
            let _ = self.delete_task(sub_id, actor);
        }

        self.invalidate_filter_cache(&project_id);
        self.track_event("task_deleted", Some(actor), Some(&project_id));
        self.auto_save();
        Ok(())
    }

    // --- Filtering & Search ---

    pub fn filter_tasks(&mut self, filter: &TaskFilter) -> Vec<&Task> {
        let cache_key = format!("{:?}", filter);
        if let Some(cached_ids) = self.filter_cache.get(&cache_key) {
            return cached_ids
                .iter()
                .filter_map(|id| self.tasks.get(id))
                .collect();
        }

        let mut results: Vec<&Task> = self
            .tasks
            .values()
            .filter(|task| {
                if let Some(ref statuses) = filter.status {
                    if !statuses.contains(&task.status) {
                        return false;
                    }
                }
                if let Some(ref priorities) = filter.priority {
                    if !priorities.contains(&task.priority) {
                        return false;
                    }
                }
                if let Some(ref assignee) = filter.assignee {
                    if task.assignee.as_ref() != Some(assignee) {
                        return false;
                    }
                }
                if let Some(ref project_id) = filter.project_id {
                    if &task.project_id != project_id {
                        return false;
                    }
                }
                if let Some(ref labels) = filter.labels {
                    if !labels.iter().any(|l| task.labels.contains(l)) {
                        return false;
                    }
                }
                if let Some(ref due_before) = filter.due_before {
                    if task.due_date.as_ref().map_or(true, |d| d > due_before) {
                        return false;
                    }
                }
                if let Some(ref due_after) = filter.due_after {
                    if task.due_date.as_ref().map_or(true, |d| d < due_after) {
                        return false;
                    }
                }
                if let Some(ref text) = filter.search_text {
                    let lower = text.to_lowercase();
                    let title_match = task.title.to_lowercase().contains(&lower);
                    let desc_match = task
                        .description
                        .as_ref()
                        .map_or(false, |d| d.to_lowercase().contains(&lower));
                    if !title_match && !desc_match {
                        return false;
                    }
                }
                true
            })
            .collect();

        results.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.created_at.cmp(&b.created_at)));

        let ids: Vec<Uuid> = results.iter().map(|t| t.id).collect();
        self.filter_cache.insert(cache_key, ids);

        results
    }

    // --- Project Management ---

    pub fn create_project(
        &mut self,
        name: String,
        owner: UserId,
    ) -> Result<ProjectId, TaskFlowError> {
        let id = Uuid::new_v4().to_string();
        let project = Project {
            id: id.clone(),
            name,
            description: None,
            owner: owner.clone(),
            members: vec![owner.clone()],
            created_at: Utc::now(),
        };
        self.projects.insert(id.clone(), project);
        self.track_event("project_created", Some(&owner), Some(&id));
        self.auto_save();
        Ok(id)
    }

    pub fn add_project_member(
        &mut self,
        project_id: &ProjectId,
        user_id: UserId,
        actor: &UserId,
    ) -> Result<(), TaskFlowError> {
        let project = self
            .projects
            .get_mut(project_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("project: {}", project_id)))?;

        if &project.owner != actor {
            return Err(TaskFlowError::PermissionDenied {
                user: actor.clone(),
                action: "add member".to_string(),
            });
        }

        if !project.members.contains(&user_id) {
            project.members.push(user_id.clone());
            let name = project.name.clone();
            let notification = Notification {
                kind: NotificationKind::ProjectInvite,
                user_id: user_id.clone(),
                message: format!("You were added to project: {}", name),
                timestamp: Utc::now(),
            };
            self.send_notification(notification);
        }

        self.track_event("member_added", Some(actor), Some(project_id));
        self.auto_save();
        Ok(())
    }

    // --- User Management ---

    pub fn register_user(&mut self, name: String, email: String) -> Result<UserId, TaskFlowError> {
        // Check for duplicate email
        if self.users.values().any(|u| u.email == email) {
            return Err(TaskFlowError::ValidationError(format!(
                "email already registered: {}",
                email
            )));
        }

        let id = Uuid::new_v4().to_string();
        let user = User {
            id: id.clone(),
            name,
            email,
            avatar_url: None,
        };
        self.users.insert(id.clone(), user);
        self.track_event("user_registered", Some(&id), None);
        self.auto_save();
        Ok(id)
    }

    pub fn get_user(&self, user_id: &UserId) -> Result<&User, TaskFlowError> {
        self.users
            .get(user_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("user: {}", user_id)))
    }

    // --- Analytics ---

    fn track_event(
        &self,
        event_type: &str,
        user_id: Option<&UserId>,
        project_id: Option<&ProjectId>,
    ) {
        // NOTE: This is broken — it takes &self but tries to mutate analytics_buffer.
        // In the real codebase this uses interior mutability via RefCell, but the
        // god-object pattern makes it hard to tell what's actually mutating what.
        // For the purposes of this test fixture, we just skip the push.
        let _event = AnalyticsEvent {
            event_type: event_type.to_string(),
            user_id: user_id.cloned(),
            project_id: project_id.cloned(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        };
    }

    pub fn flush_analytics(&mut self) -> Vec<AnalyticsEvent> {
        std::mem::take(&mut self.analytics_buffer)
    }

    pub fn get_project_stats(&self, project_id: &ProjectId) -> Result<ProjectStats, TaskFlowError> {
        if !self.projects.contains_key(project_id) {
            return Err(TaskFlowError::NotFound(format!(
                "project: {}",
                project_id
            )));
        }

        let project_tasks: Vec<&Task> = self
            .tasks
            .values()
            .filter(|t| &t.project_id == project_id)
            .collect();

        let total = project_tasks.len();
        let done = project_tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .count();
        let overdue = project_tasks
            .iter()
            .filter(|t| {
                t.due_date
                    .map_or(false, |d| d < Utc::now() && t.status != TaskStatus::Done)
            })
            .count();

        Ok(ProjectStats {
            total_tasks: total,
            completed_tasks: done,
            overdue_tasks: overdue,
            completion_rate: if total > 0 {
                done as f64 / total as f64
            } else {
                0.0
            },
        })
    }

    // --- Notifications ---

    pub fn register_notification_hook(
        &mut self,
        hook: Box<dyn Fn(&Notification) + Send + Sync>,
    ) {
        self.notification_hooks.push(hook);
    }

    fn send_notification(&self, notification: Notification) {
        for hook in &self.notification_hooks {
            hook(&notification);
        }
        self.track_event(
            &format!("notification_{:?}", notification.kind),
            Some(&notification.user_id),
            None,
        );
    }

    // --- Helpers (permission checks, validation, etc.) ---

    fn is_member(&self, user_id: &UserId, project_id: &ProjectId) -> bool {
        self.projects
            .get(project_id)
            .map_or(false, |p| p.members.contains(user_id))
    }

    fn is_project_owner(&self, user_id: &UserId, project_id: &ProjectId) -> bool {
        self.projects
            .get(project_id)
            .map_or(false, |p| &p.owner == user_id)
    }

    fn is_valid_transition(&self, from: TaskStatus, to: TaskStatus) -> bool {
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

    fn invalidate_filter_cache(&mut self, _project_id: &ProjectId) {
        self.filter_cache.clear();
    }

    fn auto_save(&self) {
        // In the real codebase this debounces and saves. Here it's a no-op.
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub overdue_tasks: usize,
    pub completion_rate: f64,
}

// ============================================================
// Tests — basic coverage of the god object
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_engine() -> TaskFlowEngine {
        TaskFlowEngine {
            tasks: HashMap::new(),
            projects: HashMap::new(),
            users: HashMap::new(),
            storage_path: PathBuf::from("/tmp/taskflow-test"),
            notification_hooks: Vec::new(),
            analytics_buffer: Vec::new(),
            filter_cache: HashMap::new(),
        }
    }

    #[test]
    fn test_create_task_requires_project_membership() {
        let mut engine = test_engine();
        let owner = "user-1".to_string();
        engine.users.insert(
            owner.clone(),
            User {
                id: owner.clone(),
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                avatar_url: None,
            },
        );
        let project_id = engine.create_project("Test".to_string(), owner.clone()).unwrap();
        let result = engine.create_task("My Task".to_string(), project_id, &owner);
        assert!(result.is_ok());

        let outsider = "user-2".to_string();
        let result = engine.create_task(
            "Sneaky Task".to_string(),
            engine.projects.keys().next().unwrap().clone(),
            &outsider,
        );
        assert!(matches!(result, Err(TaskFlowError::PermissionDenied { .. })));
    }

    #[test]
    fn test_status_transitions() {
        let mut engine = test_engine();
        let owner = "user-1".to_string();
        let project_id = engine.create_project("Test".to_string(), owner.clone()).unwrap();
        let task_id = engine
            .create_task("Task".to_string(), project_id, &owner)
            .unwrap();

        // Valid: Backlog -> Todo
        assert!(engine
            .update_task_status(task_id, TaskStatus::Todo, &owner)
            .is_ok());
        // Invalid: Todo -> Done (must go through InProgress and InReview)
        assert!(matches!(
            engine.update_task_status(task_id, TaskStatus::Done, &owner),
            Err(TaskFlowError::InvalidTransition { .. })
        ));
        // Valid: any -> Cancelled
        assert!(engine
            .update_task_status(task_id, TaskStatus::Cancelled, &owner)
            .is_ok());
    }

    #[test]
    fn test_duplicate_email_rejected() {
        let mut engine = test_engine();
        engine
            .register_user("Alice".to_string(), "alice@example.com".to_string())
            .unwrap();
        let result = engine.register_user("Bob".to_string(), "alice@example.com".to_string());
        assert!(matches!(result, Err(TaskFlowError::ValidationError(_))));
    }
}
