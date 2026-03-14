# Scaffold Response for ASSESSMENT_CIRCULAR.md

## What Was Done

Scaffolded the new architecture as specified in ASSESSMENT_CIRCULAR.md. Created 5 modules with trait definitions and type skeletons:

### Module Summary

1. **`domain/mod.rs`** (Layer 1) -- Core types with no dependencies on other modules.
   - Types: `Task`, `TaskStatus`, `Priority`, `Project`, `User`, `Notification`, `NotificationKind`, `AnalyticsEvent`, `TaskFilter`, `ProjectStats`
   - Type aliases: `UserId`, `ProjectId`
   - Error type: `TaskFlowError` with Display and Error impls
   - Business rule: `is_valid_transition()` for task status transitions

2. **`storage/mod.rs`** (Layer 2) -- Persistence adapter, depends on `domain` only.
   - Trait: `StorageBackend` with `save`/`load` methods for tasks, projects, and users
   - Stub: `JsonFileStorage` struct with `todo!()` implementations

3. **`task_service/mod.rs`** (Layer 3) -- Task operations, depends on `domain` and `project_service`.
   - Trait: `TaskService` with `create_task`, `update_status`, `assign`, `delete`, `get_task`, `filter_by_project`
   - Methods that need project validation accept `&dyn ProjectService` as a parameter
   - `filter_by_project` is the method `ProjectService::get_stats` calls back into

4. **`project_service/mod.rs`** (Layer 3) -- Project operations, depends on `domain` and `task_service`.
   - Trait: `ProjectService` with `create_project`, `add_member`, `get_project`, `get_stats`
   - `get_stats` accepts `&dyn TaskService` to call `filter_by_project` for computing stats
   - `get_project` is the method `TaskService` methods call back into for validation

5. **`notification/mod.rs`** (Layer 3) -- Notification dispatch, depends on `domain`, `task_service`, `project_service`.
   - Trait: `NotificationService` with `notify`, `register_hook`, `notify_task_assigned`, `notify_project_invite`
   - Stub: `HookNotificationService` with `todo!()` implementations
   - Uses `TaskService` to look up task details for notification messages
   - Uses `ProjectService` to look up project details for invite notifications

### Compilation

The scaffold compiles successfully (warnings only -- all "never used" warnings expected for a type skeleton).

## Issues Found

### CRITICAL: Circular Dependency Between task_service and project_service

The assessment specifies that:
- `task_service` depends on `project_service` (needs `ProjectService::get_project` to validate project exists and check membership)
- `project_service` depends on `task_service` (needs `TaskService::filter_by_project` to compute project stats)

This is a **circular dependency**. While Rust allows mutual `use` imports between modules in the same crate (since they compile as a single unit), this creates several problems:

1. **Tight coupling**: The two services cannot be tested, reasoned about, or evolved independently. Every change to one requires understanding the other.

2. **API ergonomics**: To avoid Rust borrow checker issues, the circular dependency forces methods to accept the other service as a parameter (e.g., `create_task(..., project_svc: &dyn ProjectService)`). This pushes the coupling burden onto every caller.

3. **Cannot split into separate crates**: If you ever wanted to extract these into separate crates (e.g., for a workspace), the circular dependency would be a hard compile error since Cargo does not allow circular crate dependencies.

4. **Ownership conflicts at runtime**: If both services hold mutable state, a method on `TaskService` that calls `ProjectService` (or vice versa) will hit borrow checker issues unless the services are behind `Rc<RefCell<>>` or similar.

### Recommended Fix

Extract a `TaskQuery` trait at Layer 2 (depends on `domain` only) that provides read-only query operations like `filter_by_project` and `get_project_stats`. Then:
- `project_service` depends on `domain` + `TaskQuery` (not `task_service`)
- `task_service` depends on `domain` + `project_service`
- No cycle

This is a standard "extract interface to lower layer" pattern for breaking circular dependencies.

### Minor: Assessment Missing User Type from Assessment Modules

The assessment lists `User` as a domain type but no module is responsible for user operations (register, lookup). The original `TaskFlowEngine` had `register_user` and `get_user` methods. These would need a home -- either in a new `user_service` module or folded into `project_service`.
