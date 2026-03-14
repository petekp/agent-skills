use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::user::{ProjectId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_type: String,
    pub user_id: Option<UserId>,
    pub project_id: Option<ProjectId>,
    pub metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}
