//! Notification dispatch abstraction.
//!
//! The `NotificationSink` trait allows business logic to send notifications
//! without knowing how they're delivered. The default `CallbackSink` preserves
//! the existing hook-based behavior.

mod hook;

pub use hook::CallbackSink;

use crate::models::Notification;

/// Trait for delivering notifications.
///
/// Implementations might log to stdout, call webhooks, send emails, etc.
/// Must be Send + Sync for shared ownership across services.
pub trait NotificationSink: Send + Sync {
    fn send(&self, notification: &Notification);
}

/// A no-op sink useful for testing.
pub struct NullSink;

impl NotificationSink for NullSink {
    fn send(&self, _notification: &Notification) {}
}
