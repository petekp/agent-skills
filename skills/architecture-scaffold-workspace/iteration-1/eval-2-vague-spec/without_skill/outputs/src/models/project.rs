use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user::{ProjectId, UserId};

/// A project that contains tasks and has members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub owner: UserId,
    pub members: Vec<UserId>,
    pub created_at: DateTime<Utc>,
}

/// Summary statistics for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub overdue_tasks: usize,
    pub completion_rate: f64,
}
