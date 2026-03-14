//! Analytics collection abstraction.
//!
//! The `AnalyticsCollector` trait fixes the original `&self` mutation bug by
//! letting implementations choose their own interior mutability strategy.

mod buffer;

pub use buffer::BufferedCollector;

use crate::models::{AnalyticsEvent, ProjectId, UserId};

/// Trait for collecting analytics events.
///
/// Implementations use interior mutability (Mutex, channel, etc.) so that
/// `track()` can be called from `&self` contexts without borrow conflicts.
pub trait AnalyticsCollector: Send + Sync {
    fn track(&self, event_type: &str, user_id: Option<&UserId>, project_id: Option<&ProjectId>);
    fn flush(&self) -> Vec<AnalyticsEvent>;
}

/// A no-op collector useful for testing.
pub struct NullCollector;

impl AnalyticsCollector for NullCollector {
    fn track(&self, _event_type: &str, _user_id: Option<&UserId>, _project_id: Option<&ProjectId>) {}
    fn flush(&self) -> Vec<AnalyticsEvent> {
        Vec::new()
    }
}
