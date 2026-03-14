use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::user::{ProjectId, UserId};

/// A task within a project.
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

/// Criteria for filtering tasks. All fields are optional; `None` means "no constraint."
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
