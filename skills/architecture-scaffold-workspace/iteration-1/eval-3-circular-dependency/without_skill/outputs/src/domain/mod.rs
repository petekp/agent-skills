use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ---- Identity aliases ----

pub type UserId = String;
pub type ProjectId = String;

// ---- Core domain types ----

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub overdue_tasks: usize,
    pub completion_rate: f64,
}

// ---- Error type ----

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

// ---- Business rules ----

pub fn is_valid_transition(from: TaskStatus, to: TaskStatus) -> bool {
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
