use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user::UserId;

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
