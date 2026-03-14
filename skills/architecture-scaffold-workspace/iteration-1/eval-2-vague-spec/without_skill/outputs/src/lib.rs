//! TaskFlow -- refactored architecture.
//!
//! This is the TARGET lib.rs after migration. The original src/lib.rs is
//! intentionally left unmodified. This file shows how the facade would
//! wire together the extracted modules.

pub mod models;
pub mod storage;
pub mod services;
pub mod notifications;
pub mod analytics;

// Re-export commonly used types at crate root for convenience.
pub use models::*;
pub use storage::Storage;
pub use notifications::NotificationSink;
pub use analytics::AnalyticsCollector;

use std::path::PathBuf;
use std::sync::Arc;

use services::{TaskService, ProjectService, UserService};
use storage::JsonFileStorage;
use notifications::CallbackSink;
use analytics::BufferedCollector;

/// Thin facade that coordinates the extracted services.
///
/// Public API surface is intentionally similar to the original `TaskFlowEngine`
/// for backward compatibility during migration.
pub struct TaskFlowEngine {
    pub task_service: TaskService,
    pub project_service: ProjectService,
    pub user_service: UserService,
    storage: Arc<JsonFileStorage>,
    analytics: Arc<BufferedCollector>,
    notifications: Arc<CallbackSink>,
}

impl TaskFlowEngine {
    pub fn new(storage_path: PathBuf) -> Result<Self, TaskFlowError> {
        let storage = Arc::new(JsonFileStorage::new(storage_path));
        let analytics = Arc::new(BufferedCollector::new());
        let notifications = Arc::new(CallbackSink::new());

        // Try loading existing data; fall back to empty collections.
        let tasks = storage.load_tasks().unwrap_or_default();
        let projects = storage.load_projects().unwrap_or_default();
        let users = storage.load_users().unwrap_or_default();

        Ok(Self {
            task_service: TaskService::new(tasks, notifications.clone(), analytics.clone()),
            project_service: ProjectService::new(projects, notifications.clone(), analytics.clone()),
            user_service: UserService::new(users, analytics.clone()),
            storage,
            analytics,
            notifications,
        })
    }

    /// Persist all current state to disk.
    pub fn save(&self) -> Result<(), TaskFlowError> {
        self.storage.save_tasks(self.task_service.tasks())?;
        self.storage.save_projects(self.project_service.projects())?;
        self.storage.save_users(self.user_service.users())?;
        self.analytics.track("data_saved", None, None);
        Ok(())
    }

    /// Register a notification callback hook.
    pub fn register_notification_hook(
        &self,
        hook: Box<dyn Fn(&Notification) + Send + Sync>,
    ) {
        self.notifications.register(hook);
    }

    /// Drain buffered analytics events.
    pub fn flush_analytics(&self) -> Vec<AnalyticsEvent> {
        self.analytics.flush()
    }
}
