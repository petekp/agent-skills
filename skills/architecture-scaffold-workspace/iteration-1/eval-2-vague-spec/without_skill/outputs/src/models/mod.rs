//! Domain model types for TaskFlow.
//!
//! This module contains all data structures and enums shared across the crate.
//! No business logic lives here -- only data definitions and trait implementations
//! needed for serialization, display, and comparison.

mod task;
mod project;
mod user;
mod notification;
mod analytics;
mod error;

pub use task::*;
pub use project::*;
pub use user::*;
pub use notification::*;
pub use analytics::*;
pub use error::*;
