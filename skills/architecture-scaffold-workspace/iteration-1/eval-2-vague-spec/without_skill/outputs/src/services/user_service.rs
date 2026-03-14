use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::analytics::AnalyticsCollector;
use crate::models::*;

/// Manages user registration and lookup.
pub struct UserService {
    users: HashMap<UserId, User>,
    analytics: Arc<dyn AnalyticsCollector>,
}

impl UserService {
    pub fn new(
        users: HashMap<UserId, User>,
        analytics: Arc<dyn AnalyticsCollector>,
    ) -> Self {
        Self { users, analytics }
    }

    pub fn users(&self) -> &HashMap<UserId, User> {
        &self.users
    }

    pub fn register(&mut self, name: String, email: String) -> Result<UserId, TaskFlowError> {
        if self.users.values().any(|u| u.email == email) {
            return Err(TaskFlowError::ValidationError(format!(
                "email already registered: {}", email
            )));
        }

        let id = Uuid::new_v4().to_string();
        let user = User {
            id: id.clone(),
            name,
            email,
            avatar_url: None,
        };
        self.users.insert(id.clone(), user);
        self.analytics.track("user_registered", Some(&id), None);
        Ok(id)
    }

    pub fn get(&self, user_id: &UserId) -> Result<&User, TaskFlowError> {
        self.users.get(user_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("user: {}", user_id)))
    }
}
