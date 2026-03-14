use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;

use super::AnalyticsCollector;
use crate::models::{AnalyticsEvent, ProjectId, UserId};

/// Buffers analytics events in memory behind a Mutex.
///
/// This fixes the original bug where `track_event(&self)` could not
/// mutate the buffer. The Mutex provides safe interior mutability.
pub struct BufferedCollector {
    buffer: Mutex<Vec<AnalyticsEvent>>,
}

impl BufferedCollector {
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new(Vec::new()),
        }
    }
}

impl AnalyticsCollector for BufferedCollector {
    fn track(&self, event_type: &str, user_id: Option<&UserId>, project_id: Option<&ProjectId>) {
        let event = AnalyticsEvent {
            event_type: event_type.to_string(),
            user_id: user_id.cloned(),
            project_id: project_id.cloned(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        };
        self.buffer.lock().unwrap().push(event);
    }

    fn flush(&self) -> Vec<AnalyticsEvent> {
        let mut buffer = self.buffer.lock().unwrap();
        std::mem::take(&mut *buffer)
    }
}
