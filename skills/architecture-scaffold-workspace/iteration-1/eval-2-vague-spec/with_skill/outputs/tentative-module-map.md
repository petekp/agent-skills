# Tentative Module Map (UNCONFIRMED -- Needs User Sign-Off)

This module map is inferred from the existing code, NOT from the assessment (which is too vague to derive a module map from). It represents one reasonable decomposition but there are many valid alternatives. The user must confirm or revise this before any skeleton code is written.

## Status: BLOCKED -- Awaiting user confirmation on questions in response.md

---

## Layer 1: Domain Types (innermost -- no dependencies on other project modules)

### module: domain
Location: `src/domain/mod.rs`
Responsibility: Core value types, identities, and enums shared across all modules
Types:
  - `Task { id, title, description, status, priority, assignee, project_id, labels, created_at, updated_at, due_date, parent_id, subtask_ids }`
  - `TaskStatus` (enum: Backlog, Todo, InProgress, InReview, Done, Cancelled)
  - `Priority` (enum: None, Low, Medium, High, Urgent)
  - `Project { id, name, description, owner, members, created_at }`
  - `User { id, name, email, avatar_url }`
  - `UserId` (type alias or newtype -- NEEDS DECISION)
  - `ProjectId` (type alias or newtype -- NEEDS DECISION)
  - `TaskFilter { status, priority, assignee, project_id, labels, due_before, due_after, search_text }`
  - `Notification { kind, user_id, message, timestamp }`
  - `NotificationKind` (enum: TaskAssigned, TaskStatusChanged, TaskDueSoon, MentionedInComment, ProjectInvite)
  - `AnalyticsEvent { event_type, user_id, project_id, metadata, timestamp }`
  - `ProjectStats { total_tasks, completed_tasks, overdue_tasks, completion_rate }`
Dependency rule: This module imports nothing from the project. All other modules may import this.

**OPEN QUESTION:** Should `Notification`, `NotificationKind`, `AnalyticsEvent`, and `ProjectStats` live in domain, or in their respective service modules? Domain is cleaner if multiple modules need these types. Service-local is cleaner if only one module uses them.

---

## Layer 2: Service Traits / Contracts (depend only on domain types)

### module: storage
Location: `src/storage/mod.rs`
Responsibility: Persistence abstraction -- load and save application state
Dependencies: domain (inward only)
Exposes:
  - `StorageService` (trait -- NEEDS CONFIRMATION whether trait-based or concrete)
    - `load() -> Result<StorageSnapshot, StorageError>`
    - `save(snapshot: &StorageSnapshot) -> Result<(), StorageError>`
  - OR concrete functions: `load_from_disk(path) -> Result<...>`, `save_to_disk(path, ...) -> Result<...>`
Types:
  - `StorageSnapshot { tasks, projects, users }` (or just pass individual collections)
  - `StorageError` (enum: IoError, SerializationError)
Source in current code: `TaskFlowEngine::new` (lines 156-171), `load_from_disk` (lines 173-197), `save_to_disk` (lines 199-218)

**OPEN QUESTION:** Should storage be a trait (enabling swap to SQLite, etc.) or a concrete JSON implementation? This drastically changes the skeleton.

### module: notification
Location: `src/notification/mod.rs`
Responsibility: Notification dispatch via registered hooks
Dependencies: domain (inward only)
Exposes:
  - `NotificationService` (trait or struct)
    - `register_hook(hook: Box<dyn Fn(&Notification) + Send + Sync>)`
    - `send(notification: Notification)`
Types:
  - (Uses `Notification` and `NotificationKind` from domain)
Source in current code: `register_notification_hook` (lines 632-637), `send_notification` (lines 639-648)

### module: analytics
Location: `src/analytics/mod.rs`
Responsibility: Event tracking and project statistics
Dependencies: domain (inward only)
Exposes:
  - `AnalyticsService` (trait or struct)
    - `track_event(event_type: &str, user_id: Option<&UserId>, project_id: Option<&ProjectId>)`
    - `flush() -> Vec<AnalyticsEvent>`
    - `get_project_stats(project_id: &ProjectId, tasks: &[&Task]) -> ProjectStats`
Types:
  - (Uses `AnalyticsEvent`, `ProjectStats` from domain)
Source in current code: `track_event` (lines 568-585), `flush_analytics` (lines 587-589), `get_project_stats` (lines 591-628)

**NOTE:** `track_event` currently has a `&self` vs mutation bug. The new design must resolve this -- either `&mut self` or interior mutability.

---

## Layer 3: Service Implementations (depend on Layer 1 + Layer 2 traits)

### module: task_service
Location: `src/task_service/mod.rs`
Responsibility: Task CRUD, status transitions, assignment, filtering
Dependencies: domain, storage (trait), notification (trait), analytics (trait)
Exposes:
  - `TaskService` (struct)
    - `create_task(title, project_id, creator) -> Result<Uuid, TaskServiceError>`
    - `update_task_status(task_id, new_status, actor) -> Result<(), TaskServiceError>`
    - `assign_task(task_id, assignee, actor) -> Result<(), TaskServiceError>`
    - `get_task(task_id) -> Result<&Task, TaskServiceError>`
    - `delete_task(task_id, actor) -> Result<(), TaskServiceError>`
    - `filter_tasks(filter) -> Vec<&Task>`
Types:
  - `TaskServiceError` (subset of current TaskFlowError: NotFound, DuplicateId, InvalidTransition, PermissionDenied, ValidationError)
Source in current code: lines 223-477

**OPEN QUESTION:** Should permission checking be inline in this service or injected as a separate policy module?

### module: project_service
Location: `src/project_service/mod.rs`
Responsibility: Project CRUD, member management
Dependencies: domain, storage (trait), notification (trait), analytics (trait)
Exposes:
  - `ProjectService` (struct)
    - `create_project(name, owner) -> Result<ProjectId, ProjectServiceError>`
    - `add_member(project_id, user_id, actor) -> Result<(), ProjectServiceError>`
Types:
  - `ProjectServiceError` (subset: NotFound, PermissionDenied)
Source in current code: lines 481-534

### module: user_service
Location: `src/user_service/mod.rs`
Responsibility: User registration and lookup
Dependencies: domain, storage (trait), analytics (trait)
Exposes:
  - `UserService` (struct)
    - `register_user(name, email) -> Result<UserId, UserServiceError>`
    - `get_user(user_id) -> Result<&User, UserServiceError>`
Types:
  - `UserServiceError` (subset: NotFound, ValidationError)
Source in current code: lines 538-564

---

## Dependency Rules (if confirmed)

```
Layer 1 (domain)                -> imports nothing from the project
Layer 2 (storage, notification, analytics) -> imports only Layer 1
Layer 3 (task_service, project_service, user_service) -> imports Layers 1 and 2
```

No module in Layer 2 may import from Layer 3. No module in Layer 1 may import from Layers 2 or 3. These rules would be mechanically verified via `use crate::` grep checks after the skeleton compiles.

---

## Unresolved Design Decisions

1. **Trait vs. concrete for Layer 2 services** -- affects every signature in Layer 3
2. **Newtype vs. alias for UserId/ProjectId** -- affects every function signature
3. **Error type strategy** -- one unified error or per-module errors
4. **Facade/coordinator** -- does something replace `TaskFlowEngine` as a single entry point, or do callers use services directly?
5. **Where does `is_valid_transition` live?** -- Domain type method? Task service? Separate policy module?
6. **Filter cache ownership** -- Task service internal detail or separate caching layer?
7. **`auto_save` behavior** -- Caller-driven persistence or automatic? If automatic, who owns the trigger?
