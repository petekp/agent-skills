//! Scaffold verification binary.
//! Declares and compiles the new architecture skeleton without modifying lib.rs.
//!
//! Architecture (from ASSESSMENT_CIRCULAR.md):
//!   Layer 1: domain           -> nothing
//!   Layer 2: storage          -> domain
//!   Layer 3: task_service     -> domain, project_service  (CIRCULAR)
//!   Layer 3: project_service  -> domain, task_service      (CIRCULAR)
//!   Layer 3: notification     -> domain, task_service, project_service

// Layer 1: Domain types and business rules
mod domain;

// Layer 2: Persistence
mod storage;

// Layer 3: Services (task_service <-> project_service form a circular dependency)
mod task_service;
mod project_service;
mod notification;

fn main() {
    println!("Scaffold compiles successfully.");
}
