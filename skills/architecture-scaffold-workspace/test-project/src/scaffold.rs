//! Scaffold verification binary.
//! Declares and compiles the new architecture skeleton without modifying lib.rs.
//!
//! Architecture layers:
//!   Layer 1: domain                                         -> nothing
//!   Layer 2: auth, storage, notification, query, analytics  -> domain only
//!   Layer 3: task_service, project_service, user_service    -> domain + Layer 2 traits

// Layer 1: Domain types and business rules
mod domain;

// Layer 2: Service contracts (depend on domain only)
mod auth;
mod storage;
mod notification;
mod query;
mod analytics;

// Layer 3: Use-case services (depend on domain + Layer 2 traits)
mod task_service;
mod project_service;
mod user_service;

fn main() {
    println!("Scaffold compiles successfully.");
}
