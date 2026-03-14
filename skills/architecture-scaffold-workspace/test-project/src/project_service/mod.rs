//! Project use-case module.
//! Layer 3: Depends on domain + Layer 2 traits (auth, notification).

use crate::domain::{ProjectId, UserId};

/// Project service errors.
#[derive(Debug)]
pub enum ProjectServiceError {
    NotFound(String),
    PermissionDenied { user: UserId, action: String },
    ValidationError(String),
}

impl std::fmt::Display for ProjectServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "not found: {}", msg),
            Self::PermissionDenied { user, action } => {
                write!(f, "permission denied: {} cannot {}", user, action)
            }
            Self::ValidationError(msg) => write!(f, "validation error: {}", msg),
        }
    }
}

impl std::error::Error for ProjectServiceError {}

/// Project use-case trait.
pub trait ProjectService {
    fn create_project(
        &mut self,
        name: String,
        owner: UserId,
    ) -> Result<ProjectId, ProjectServiceError>;

    fn add_member(
        &mut self,
        project_id: &ProjectId,
        user_id: UserId,
        actor: &UserId,
    ) -> Result<(), ProjectServiceError>;
}

/// Stub implementation with injected Layer 2 dependencies.
pub struct ProjectServiceImpl<
    A: crate::auth::AuthPolicy,
    N: crate::notification::NotificationService,
> {
    _auth: A,
    _notifications: N,
}

impl<A: crate::auth::AuthPolicy, N: crate::notification::NotificationService>
    ProjectServiceImpl<A, N>
{
    pub fn new(auth: A, notifications: N) -> Self {
        Self {
            _auth: auth,
            _notifications: notifications,
        }
    }
}

impl<A: crate::auth::AuthPolicy, N: crate::notification::NotificationService> ProjectService
    for ProjectServiceImpl<A, N>
{
    fn create_project(
        &mut self,
        _name: String,
        _owner: UserId,
    ) -> Result<ProjectId, ProjectServiceError> {
        todo!("ProjectServiceImpl::create_project")
    }

    fn add_member(
        &mut self,
        _project_id: &ProjectId,
        _user_id: UserId,
        _actor: &UserId,
    ) -> Result<(), ProjectServiceError> {
        todo!("ProjectServiceImpl::add_member")
    }
}
