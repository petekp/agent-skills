use std::sync::Mutex;

use super::NotificationSink;
use crate::models::Notification;

/// Delivers notifications by calling registered callback functions.
/// This preserves the existing `notification_hooks` behavior from the original engine.
pub struct CallbackSink {
    hooks: Mutex<Vec<Box<dyn Fn(&Notification) + Send + Sync>>>,
}

impl CallbackSink {
    pub fn new() -> Self {
        Self {
            hooks: Mutex::new(Vec::new()),
        }
    }

    pub fn register(&self, hook: Box<dyn Fn(&Notification) + Send + Sync>) {
        self.hooks.lock().unwrap().push(hook);
    }
}

impl NotificationSink for CallbackSink {
    fn send(&self, notification: &Notification) {
        let hooks = self.hooks.lock().unwrap();
        for hook in hooks.iter() {
            hook(notification);
        }
    }
}
