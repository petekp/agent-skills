use uuid::Uuid;

use super::user::UserId;
use super::task::TaskStatus;

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
