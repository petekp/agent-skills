use crate::domain::{User, UserId};
#[derive(Debug)]
pub enum UserServiceError { NotFound(String), DuplicateEmail(String) }
impl std::fmt::Display for UserServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "not found: {}", msg),
            Self::DuplicateEmail(email) => write!(f, "duplicate email: {}", email),
        }
    }
}
impl std::error::Error for UserServiceError {}
pub trait UserService {
    fn register(&mut self, name: String, email: String) -> Result<UserId, UserServiceError>;
    fn get(&self, user_id: &UserId) -> Result<User, UserServiceError>;
}
pub struct UserServiceImpl;
impl UserService for UserServiceImpl {
    fn register(&mut self, _name: String, _email: String) -> Result<UserId, UserServiceError> { todo!() }
    fn get(&self, _user_id: &UserId) -> Result<User, UserServiceError> { todo!() }
}
