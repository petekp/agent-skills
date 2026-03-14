//! Task use-case module.
//! Layer 3: Depends on domain + Layer 2 traits (auth, notification).

use uuid::Uuid;

use crate::domain::{ProjectId, Task, TaskStatus, UserId};

/// Task service errors.
#[derive(Debug)]
pub enum TaskServiceError {
    NotFound(String),
    DuplicateId(Uuid),
    InvalidTransition { from: TaskStatus, to: TaskStatus },
    PermissionDenied { user: UserId, action: String },
    ValidationError(String),
}

impl std::fmt::Display for TaskServiceError {
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
            Self::ValidationError(msg) => write!(f, "validation error: {}", msg),
        }
    }
}

impl std::error::Error for TaskServiceError {}

/// Task use-case trait.
pub trait TaskService {
    fn create_task(
        &mut self,
        title: String,
        project_id: ProjectId,
        creator: &UserId,
    ) -> Result<Uuid, TaskServiceError>;

    fn update_status(
        &mut self,
        task_id: Uuid,
        new_status: TaskStatus,
        actor: &UserId,
    ) -> Result<(), TaskServiceError>;

    fn assign(
        &mut self,
        task_id: Uuid,
        assignee: UserId,
        actor: &UserId,
    ) -> Result<(), TaskServiceError>;

    fn delete(&mut self, task_id: Uuid, actor: &UserId) -> Result<(), TaskServiceError>;

    fn get(&self, task_id: Uuid) -> Result<Task, TaskServiceError>;
}

/// Stub implementation with injected Layer 2 dependencies.
pub struct TaskServiceImpl<A: crate::auth::AuthPolicy, N: crate::notification::NotificationService>
{
    _auth: A,
    _notifications: N,
}

impl<A: crate::auth::AuthPolicy, N: crate::notification::NotificationService> TaskServiceImpl<A, N>
{
    pub fn new(auth: A, notifications: N) -> Self {
        Self {
            _auth: auth,
            _notifications: notifications,
        }
    }
}

impl<A: crate::auth::AuthPolicy, N: crate::notification::NotificationService> TaskService
    for TaskServiceImpl<A, N>
{
    fn create_task(
        &mut self,
        _title: String,
        _project_id: ProjectId,
        _creator: &UserId,
    ) -> Result<Uuid, TaskServiceError> {
        todo!("TaskServiceImpl::create_task")
    }

    fn update_status(
        &mut self,
        _task_id: Uuid,
        _new_status: TaskStatus,
        _actor: &UserId,
    ) -> Result<(), TaskServiceError> {
        todo!("TaskServiceImpl::update_status")
    }

    fn assign(
        &mut self,
        _task_id: Uuid,
        _assignee: UserId,
        _actor: &UserId,
    ) -> Result<(), TaskServiceError> {
        todo!("TaskServiceImpl::assign")
    }

    fn delete(&mut self, _task_id: Uuid, _actor: &UserId) -> Result<(), TaskServiceError> {
        todo!("TaskServiceImpl::delete")
    }

    fn get(&self, _task_id: Uuid) -> Result<Task, TaskServiceError> {
        todo!("TaskServiceImpl::get")
    }
}
