use std::collections::HashMap;
use uuid::Uuid;
use crate::domain::{AnalyticsEvent, ProjectId, ProjectStats, Task, UserId};
pub trait AnalyticsCollector {
    fn track_event(&mut self, event_type: &str, user_id: Option<&UserId>, project_id: Option<&ProjectId>);
    fn flush(&mut self) -> Vec<AnalyticsEvent>;
    fn get_project_stats(&self, tasks: &HashMap<Uuid, Task>, project_id: &ProjectId) -> ProjectStats;
}
pub struct BufferedAnalyticsCollector { buffer: Vec<AnalyticsEvent> }
impl BufferedAnalyticsCollector {
    pub fn new() -> Self { Self { buffer: Vec::new() } }
}
impl AnalyticsCollector for BufferedAnalyticsCollector {
    fn track_event(&mut self, _event_type: &str, _user_id: Option<&UserId>, _project_id: Option<&ProjectId>) { todo!() }
    fn flush(&mut self) -> Vec<AnalyticsEvent> { todo!() }
    fn get_project_stats(&self, _tasks: &HashMap<Uuid, Task>, _project_id: &ProjectId) -> ProjectStats { todo!() }
}
