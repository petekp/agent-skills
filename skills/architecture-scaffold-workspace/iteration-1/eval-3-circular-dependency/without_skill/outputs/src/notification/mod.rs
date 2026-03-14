//! Notification dispatch module.
//! Layer 3: Depends on `domain`, `task_service`, `project_service`.
//!
//! Needs task details from `task_service` for notification messages
//! (e.g., task title when notifying about assignment).
//! Needs project details from `project_service` for invite notifications
//! (e.g., project name when notifying about project invite).

use crate::domain::{Notification, TaskFlowError};
use crate::task_service::TaskService;
use crate::project_service::ProjectService;
use uuid::Uuid;

/// Trait for notification dispatch.
pub trait NotificationService {
    /// Send a notification through all registered hooks.
    fn notify(&self, notification: &Notification);

    /// Register a notification hook/callback.
    fn register_hook(&mut self, hook: Box<dyn Fn(&Notification) + Send + Sync>);

    /// Send a task-assignment notification, looking up task details from TaskService.
    fn notify_task_assigned(
        &self,
        task_id: Uuid,
        assignee: &str,
        task_svc: &dyn TaskService,
    ) -> Result<(), TaskFlowError>;

    /// Send a project-invite notification, looking up project details from ProjectService.
    fn notify_project_invite(
        &self,
        project_id: &str,
        invitee: &str,
        project_svc: &dyn ProjectService,
    ) -> Result<(), TaskFlowError>;
}

/// Default in-process notification dispatcher using callback hooks.
pub struct HookNotificationService {
    hooks: Vec<Box<dyn Fn(&Notification) + Send + Sync>>,
}

impl HookNotificationService {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }
}

impl NotificationService for HookNotificationService {
    fn notify(&self, _notification: &Notification) {
        todo!("migrate from TaskFlowEngine::send_notification")
    }

    fn register_hook(&mut self, _hook: Box<dyn Fn(&Notification) + Send + Sync>) {
        todo!("migrate from TaskFlowEngine::register_notification_hook")
    }

    fn notify_task_assigned(
        &self,
        _task_id: Uuid,
        _assignee: &str,
        _task_svc: &dyn TaskService,
    ) -> Result<(), TaskFlowError> {
        todo!("look up task via task_svc.get_task() and send assignment notification")
    }

    fn notify_project_invite(
        &self,
        _project_id: &str,
        _invitee: &str,
        _project_svc: &dyn ProjectService,
    ) -> Result<(), TaskFlowError> {
        todo!("look up project via project_svc.get_project() and send invite notification")
    }
}
