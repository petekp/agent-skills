# TaskFlow Architecture Plan

## Current State

Single file (`src/lib.rs`, ~770 lines) containing a god object `TaskFlowEngine` that handles:
- Domain types (Task, Project, User, etc.)
- Task CRUD + status workflow
- Project management
- User management
- Filtering/search with caching
- JSON file persistence
- Notification dispatch
- Analytics event tracking

Key bugs: `track_event` takes `&self` but needs to mutate `analytics_buffer`.

## Target Module Map

```
src/
  lib.rs              -- Re-exports, facade (TaskFlowEngine becomes a thin coordinator)
  models/
    mod.rs            -- Re-exports all model types
    task.rs           -- Task, TaskStatus, Priority, TaskFilter
    project.rs        -- Project, ProjectStats
    user.rs           -- User, UserId, ProjectId type aliases
    notification.rs   -- Notification, NotificationKind
    analytics.rs      -- AnalyticsEvent
    error.rs          -- TaskFlowError
  storage/
    mod.rs            -- Storage trait + re-exports
    json_file.rs      -- JsonFileStorage implementation
  services/
    mod.rs            -- Re-exports
    task_service.rs   -- Task CRUD, status transitions, filtering, cache
    project_service.rs -- Project CRUD, membership
    user_service.rs   -- User registration, lookup
  notifications/
    mod.rs            -- NotificationSink trait + re-exports
    hook.rs           -- CallbackNotificationSink (current behavior)
  analytics/
    mod.rs            -- AnalyticsCollector trait + re-exports
    buffer.rs         -- BufferedCollector (fixes the &self mutation bug)
```

## Dependency Diagram

```
                  TaskFlowEngine (facade)
                 /        |        \
          TaskService  ProjectService  UserService
              |            |              |
              +-----+------+------+-------+
                    |             |
               Storage trait   NotificationSink trait
                    |             |
            JsonFileStorage   CallbackNotificationSink

          All services also use --> AnalyticsCollector trait
                                        |
                                   BufferedCollector
```

**Rules:**
- Services depend on traits, never on concrete implementations
- Models are shared across all layers (no mapping needed yet -- single crate)
- The facade coordinates services but adds no business logic of its own

## Migration Strategy

### Phase 1: Extract models (no behavior change)
Move all type definitions (Task, Project, User, enums, error type) into `src/models/`. Update `lib.rs` to `pub mod models; pub use models::*;`. Existing tests pass without changes.

### Phase 2: Extract storage trait + implementation
Define `Storage` trait. Move `load_from_disk` / `save_to_disk` into `JsonFileStorage`. Services receive `&dyn Storage` or `Arc<dyn Storage>`.

### Phase 3: Extract notification + analytics traits
Define `NotificationSink` and `AnalyticsCollector` traits. This fixes the `&self` mutation bug: `BufferedCollector` can use `Mutex<Vec<AnalyticsEvent>>` internally.

### Phase 4: Extract services
Split `TaskFlowEngine` methods into `TaskService`, `ProjectService`, `UserService`. Each service takes references to storage + notification + analytics traits.

### Phase 5: Thin facade
`TaskFlowEngine` becomes a coordinator that owns the services and wires dependencies. Public API surface stays the same for backward compatibility.

## Key Trait Definitions

```rust
pub trait Storage: Send + Sync {
    fn load_tasks(&self) -> Result<HashMap<Uuid, Task>, TaskFlowError>;
    fn load_projects(&self) -> Result<HashMap<ProjectId, Project>, TaskFlowError>;
    fn load_users(&self) -> Result<HashMap<UserId, User>, TaskFlowError>;
    fn save_tasks(&self, tasks: &HashMap<Uuid, Task>) -> Result<(), TaskFlowError>;
    fn save_projects(&self, projects: &HashMap<ProjectId, Project>) -> Result<(), TaskFlowError>;
    fn save_users(&self, users: &HashMap<UserId, User>) -> Result<(), TaskFlowError>;
}

pub trait NotificationSink: Send + Sync {
    fn send(&self, notification: &Notification);
}

pub trait AnalyticsCollector: Send + Sync {
    fn track(&self, event_type: &str, user_id: Option<&UserId>, project_id: Option<&ProjectId>);
    fn flush(&self) -> Vec<AnalyticsEvent>;
}
```
