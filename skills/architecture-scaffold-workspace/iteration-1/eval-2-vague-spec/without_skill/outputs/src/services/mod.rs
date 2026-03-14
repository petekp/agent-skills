//! Business logic services.
//!
//! Each service owns one domain area and depends on trait abstractions
//! for storage, notifications, and analytics -- never on concrete types.

pub mod task_service;
pub mod project_service;
pub mod user_service;

pub use task_service::TaskService;
pub use project_service::ProjectService;
pub use user_service::UserService;
