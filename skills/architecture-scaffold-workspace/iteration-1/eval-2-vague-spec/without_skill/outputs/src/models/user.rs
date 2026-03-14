use serde::{Deserialize, Serialize};

pub type UserId = String;
pub type ProjectId = String;

/// A registered user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
}
