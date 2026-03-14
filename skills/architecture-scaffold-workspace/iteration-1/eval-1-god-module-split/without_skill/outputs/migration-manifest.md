# Migration Manifest: TaskFlowEngine God Object Split

## Overview

This manifest maps every type, method, and block of logic in the original `src/lib.rs`
god object (`TaskFlowEngine`) to its destination in the new modular architecture.

The new skeleton lives alongside the original code (compiled via `scaffold-check` binary)
and `src/lib.rs` was not modified.

## Architecture Layers

```
Layer 1: domain           -> no dependencies
Layer 2: auth, storage, notification, task_query, analytics -> domain only
Layer 3: task_service, project_service, user_service        -> domain + Layer 2 traits
```

## Module File Map

| New Module         | File                          | Layer |
|--------------------|-------------------------------|-------|
| domain             | src/domain/mod.rs             | 1     |
| auth               | src/auth/mod.rs               | 2     |
| storage            | src/storage/mod.rs            | 2     |
| notification       | src/notification/mod.rs       | 2     |
| task_query         | src/task_query/mod.rs         | 2     |
| analytics          | src/analytics/mod.rs          | 2     |
| task_service       | src/task_service/mod.rs       | 3     |
| project_service    | src/project_service/mod.rs    | 3     |
| user_service       | src/user_service/mod.rs       | 3     |

Entry point: `src/scaffold.rs` (binary target `scaffold-check`)

---

## Type Migration

| Original (lib.rs)         | Destination                   | Status    |
|---------------------------|-------------------------------|-----------|
| `Task`                    | `domain::Task`                | Skeleton  |
| `TaskStatus`              | `domain::TaskStatus`          | Skeleton  |
| `Priority`                | `domain::Priority`            | Skeleton  |
| `Project`                 | `domain::Project`             | Skeleton  |
| `User`                    | `domain::User`                | Skeleton  |
| `UserId` (type alias)     | `domain::UserId`              | Skeleton  |
| `ProjectId` (type alias)  | `domain::ProjectId`           | Skeleton  |
| `Notification`            | `domain::Notification`        | Skeleton  |
| `NotificationKind`        | `domain::NotificationKind`    | Skeleton  |
| `AnalyticsEvent`          | `domain::AnalyticsEvent`      | Skeleton  |
| `TaskFilter`              | `domain::TaskFilter`          | Skeleton  |
| `ProjectStats`            | `domain::ProjectStats`        | Skeleton  |
| `TaskFlowError`           | `domain::TaskFlowError`       | Skeleton  |
| `TaskFlowEngine` (struct) | REMOVED -- replaced by traits | -         |

---

## Method Migration

### TaskFlowEngine -> task_service::TaskService (+ TaskServiceImpl)

| Original Method               | New Trait Method                | Notes                                           |
|-------------------------------|--------------------------------|-------------------------------------------------|
| `create_task()`               | `TaskService::create_task()`   | Permission check delegates to ProjectValidator  |
| `update_task_status()`        | `TaskService::update_status()` | Uses domain::is_valid_transition                |
| `assign_task()`               | `TaskService::assign()`        | Permission check delegates to ProjectValidator  |
| `delete_task()`               | `TaskService::delete()`        | Recursive subtask deletion preserved            |
| `get_task()`                  | `TaskService::get_task()`      | Returns owned Task (not &Task)                  |

### TaskFlowEngine -> project_service::ProjectService (+ ProjectServiceImpl)

| Original Method               | New Trait Method                    | Notes                                      |
|-------------------------------|-------------------------------------|--------------------------------------------|
| `create_project()`            | `ProjectService::create_project()`  | --                                         |
| `add_project_member()`        | `ProjectService::add_member()`      | --                                         |
| `get_project_stats()`         | `ProjectService::get_stats()`       | Delegates to task_query for task counts    |
| (implicit)                    | `ProjectService::get_project()`     | New: explicit project lookup               |
| `is_member()` (helper)        | `ProjectService::is_member()`       | Exposed as trait method                    |
| `is_project_owner()` (helper) | `ProjectService::is_owner()`        | Exposed as trait method                    |

### TaskFlowEngine -> user_service::UserService (+ UserServiceImpl)

| Original Method               | New Trait Method              | Notes                           |
|-------------------------------|-------------------------------|---------------------------------|
| `register_user()`             | `UserService::register()`     | Duplicate email check preserved |
| `get_user()`                  | `UserService::get()`          | Returns owned User              |

### TaskFlowEngine -> storage::StorageBackend (+ JsonFileStorage)

| Original Method               | New Trait Method                    | Notes                                 |
|-------------------------------|-------------------------------------|---------------------------------------|
| `save_to_disk()` (tasks)      | `StorageBackend::save_tasks()`      | Split into per-entity methods         |
| `save_to_disk()` (projects)   | `StorageBackend::save_projects()`   |                                       |
| `save_to_disk()` (users)      | `StorageBackend::save_users()`      |                                       |
| `load_from_disk()` (tasks)    | `StorageBackend::load_tasks()`      |                                       |
| `load_from_disk()` (projects) | `StorageBackend::load_projects()`   |                                       |
| `load_from_disk()` (users)    | `StorageBackend::load_users()`      |                                       |
| `auto_save()` (helper)        | Caller responsibility               | Not in trait; orchestrator handles it  |

### TaskFlowEngine -> notification::NotificationService (+ NotificationServiceImpl)

| Original Method                  | New Trait Method                          | Notes                    |
|----------------------------------|-------------------------------------------|--------------------------|
| `send_notification()`            | `NotificationService::notify()`           | Takes Notification value |
| `register_notification_hook()`   | `NotificationService::register_hook()`    | --                       |

### TaskFlowEngine -> task_query::TaskQuery (+ InMemoryTaskQuery)

| Original Method               | New Trait Method                  | Notes                                 |
|-------------------------------|-----------------------------------|---------------------------------------|
| `filter_tasks()`              | `TaskQuery::filter_tasks()`       | Takes TaskFilter, returns Vec<Task>   |
| (implicit)                    | `TaskQuery::filter_by_project()`  | New: extracted from get_project_stats |
| `get_project_stats()`         | `TaskQuery::get_project_stats()`  | Stats computation moved to query layer|
| `invalidate_filter_cache()`   | `CachedTaskQuery::invalidate()`   | Method on impl, not trait             |

### TaskFlowEngine -> analytics::AnalyticsCollector (+ BufferedAnalyticsCollector)

| Original Method               | New Trait Method                            | Notes                              |
|-------------------------------|---------------------------------------------|------------------------------------|
| `track_event()`               | `AnalyticsCollector::track_event()`         | Now takes &mut self (fixes bug)    |
| `flush_analytics()`           | `AnalyticsCollector::flush()`               | --                                 |
| `get_project_stats()`         | `AnalyticsCollector::get_project_stats()`   | Takes tasks HashMap as parameter   |

### TaskFlowEngine -> auth::AuthPolicy (+ DefaultAuthPolicy)

| Original Method               | New Trait Method                    | Notes                              |
|-------------------------------|-------------------------------------|------------------------------------|
| `is_member()` (helper)        | `AuthPolicy::can_create_task()`     | Consolidated into policy methods   |
| `is_member()` (helper)        | `AuthPolicy::can_update_task()`     |                                    |
| `is_member()` (helper)        | `AuthPolicy::can_assign_task()`     |                                    |
| `is_project_owner()` (helper) | `AuthPolicy::can_delete_task()`     | Owner-only check                   |
| `is_project_owner()` (helper) | `AuthPolicy::can_add_member()`      | Owner-only check                   |

### Business Rules -> domain (free functions)

| Original Method               | New Location                        | Notes                    |
|-------------------------------|-------------------------------------|--------------------------|
| `is_valid_transition()`       | `domain::is_valid_transition()`     | Pure function, no self   |

---

## Dependency Validation

### Circular Dependency Resolution

The original assessment proposed task_service depending on project_service and vice versa.
The skeleton resolves this:

- `task_service` defines a `ProjectValidator` trait for project existence/membership checks.
  At runtime, an adapter backed by project_service implements this trait.
- `project_service` delegates stats computation to `task_query` (Layer 2), not task_service.
- Neither Layer 3 module imports the other directly.

### Actual Dependencies (verified by cargo check)

```
domain          -> (nothing)
auth            -> domain
storage         -> domain
notification    -> domain
task_query      -> domain
analytics       -> domain
task_service    -> domain
project_service -> domain
user_service    -> domain
```

No Layer 2 module depends on another Layer 2 module. No circular dependencies.

---

## Bug Fixes Captured in Skeleton

1. **track_event &self mutation bug** (lib.rs:574): The original `track_event` took `&self`
   but needed to mutate `analytics_buffer`. The new `AnalyticsCollector::track_event` takes
   `&mut self`, fixing the borrow checker issue.

2. **Scattered permission checks**: All authorization now goes through `AuthPolicy` trait
   methods instead of ad-hoc `is_member()`/`is_project_owner()` calls in each method.

---

## Compilation Status

- `cargo check --bin scaffold-check`: PASS (warnings only: dead_code, expected for stubs)
- `cargo check --lib`: PASS (original lib.rs unchanged)
- `cargo test`: PASS (all 3 original tests pass)
