//! Notification dispatch.
//! Layer 2: Depends on domain only.

use crate::domain::Notification;

/// Notification dispatch trait.
pub trait NotificationService {
    fn notify(&self, notification: &Notification);
    fn register_hook(&mut self, hook: Box<dyn Fn(&Notification) + Send + Sync>);
}

/// Default hook-based notification dispatcher.
pub struct DefaultNotificationService {
    hooks: Vec<Box<dyn Fn(&Notification) + Send + Sync>>,
}

impl DefaultNotificationService {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }
}

impl NotificationService for DefaultNotificationService {
    fn notify(&self, _notification: &Notification) {
        todo!("DefaultNotificationService::notify")
    }
    fn register_hook(&mut self, _hook: Box<dyn Fn(&Notification) + Send + Sync>) {
        todo!("DefaultNotificationService::register_hook")
    }
}
