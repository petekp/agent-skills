# TaskFlow Architecture Assessment

## Overview

TaskFlow is a task management library implemented as a single god object (`TaskFlowEngine`) in `src/lib.rs`. The engine mixes six distinct responsibilities: task CRUD, project management, user management, search/filtering, persistence, notifications, and analytics.

## Problems

### God Object: TaskFlowEngine

`TaskFlowEngine` (~300 lines of implementation) owns everything:
- Task creation, updates, deletion, and status transitions (`create_task`, `update_task_status`, `assign_task`, `delete_task`)
- Project lifecycle and membership (`create_project`, `add_project_member`)
- User registration and lookup (`register_user`, `get_user`)
- Filtering and search with caching (`filter_tasks`)
- Disk persistence (`save_to_disk`, `load_from_disk`, `auto_save`)
- Notification dispatch (`send_notification`, `register_notification_hook`)
- Analytics event tracking and stats (`track_event`, `flush_analytics`, `get_project_stats`)

This causes:
- Borrow checker friction (had to work around `&self` vs `&mut self` conflicts between notification dispatch and task mutation)
- Every change touches the same file
- Impossible to test one concern in isolation
- Permission logic is duplicated across methods

### Mixed Concern Types

Domain types (`Task`, `Project`, `User`), persistence types, API types (`TaskFilter`), notification types, and analytics types all live in the same module. There's no layering.

### Permission Logic is Scattered

`is_member()` and `is_project_owner()` checks are manually called in each method. There's no centralized authorization policy.

## Recommended Architecture

Split into these modules:

### 1. `domain` — Core types and business rules
- `Task`, `Project`, `User`, `TaskStatus`, `Priority`
- Status transition validation (`is_valid_transition`)
- The types should be pure data + invariant enforcement

### 2. `task_service` — Task use cases
- `TaskService` trait with: `create_task`, `update_status`, `assign`, `delete`, `get`
- Depends on: `domain`, `auth` (for permission checks), `notification_service` (for dispatch)

### 3. `project_service` — Project use cases
- `ProjectService` trait with: `create_project`, `add_member`, `get_stats`
- Depends on: `domain`, `auth`

### 4. `user_service` — User use cases
- `UserService` trait with: `register`, `get`
- Depends on: `domain`

### 5. `auth` — Authorization policy
- Centralized permission checks: `can_create_task`, `can_delete_task`, `can_add_member`, etc.
- Depends on: `domain` only

### 6. `storage` — Persistence adapter
- `StorageBackend` trait with: `save_tasks`, `load_tasks`, `save_projects`, etc.
- `JsonFileStorage` implementation
- Depends on: `domain` only

### 7. `notification` — Notification dispatch
- `NotificationService` trait with: `notify`, `register_hook`
- Depends on: `domain` only (notification types)

### 8. `query` — Filtering and search
- `TaskQuery` trait with: `filter_tasks`, `search`
- Owns the filter cache
- Depends on: `domain` only

### 9. `analytics` — Event tracking and stats
- `AnalyticsCollector` trait with: `track_event`, `flush`, `get_project_stats`
- Depends on: `domain` only

## Dependency Rules

```
domain (Layer 1)         → nothing
auth, storage, notification, query, analytics (Layer 2) → domain only
task_service, project_service, user_service (Layer 3) → domain + Layer 2 traits
```

No Layer 2 module may depend on another Layer 2 module. Layer 3 modules may depend on Layer 2 traits (not implementations).
