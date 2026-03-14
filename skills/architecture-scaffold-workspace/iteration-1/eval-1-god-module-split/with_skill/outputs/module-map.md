# Module Map

Source assessment: ASSESSMENT.md
Project: taskflow (Rust)

## Dependency Rules

```
Layer 1 (domain)                                        -> imports nothing from the project
Layer 2 (auth, storage, notification, query, analytics) -> imports domain only
Layer 3 (task_service, project_service, user_service)   -> imports domain + Layer 2 traits
```

No Layer 2 module may depend on another Layer 2 module.
Layer 3 modules depend on Layer 2 traits (not implementations).

---

## Layer 1: Domain Types (innermost -- no dependencies)

### module: domain
Location: src/domain/mod.rs
Responsibility: Shared value types, identities, enums, and business rules used across all modules
Types:
  - Task { id, title, description, status, priority, assignee, project_id, labels, created_at, updated_at, due_date, parent_id, subtask_ids }
  - TaskStatus (enum: Backlog, Todo, InProgress, InReview, Done, Cancelled)
  - Priority (enum: None, Low, Medium, High, Urgent)
  - Project { id, name, description, owner, members, created_at }
  - User { id, name, email, avatar_url }
  - Notification { kind, user_id, message, timestamp }
  - NotificationKind (enum: TaskAssigned, TaskStatusChanged, TaskDueSoon, MentionedInComment, ProjectInvite)
  - AnalyticsEvent { event_type, user_id, project_id, metadata, timestamp }
  - TaskFilter { status, priority, assignee, project_id, labels, due_before, due_after, search_text }
  - ProjectStats { total_tasks, completed_tasks, overdue_tasks, completion_rate }
  - UserId (type alias = String)
  - ProjectId (type alias = String)
Functions:
  - is_valid_transition(from: TaskStatus, to: TaskStatus) -> bool
Dependency rule: This module imports nothing from the project. Everything else may import this.

---

## Layer 2: Service Contracts (depend only on domain types)

### module: auth
Location: src/auth/mod.rs
Responsibility: Centralized authorization policy
Dependencies: domain only
Exposes:
  - AuthPolicy (trait)
    - can_create_task(user: &UserId, project: &Project) -> bool
    - can_update_task(user: &UserId, project: &Project) -> bool
    - can_delete_task(user: &UserId, project: &Project) -> bool
    - can_assign_task(user: &UserId, project: &Project) -> bool
    - can_add_member(user: &UserId, project: &Project) -> bool
  - DefaultAuthPolicy (stub impl)

### module: storage
Location: src/storage/mod.rs
Responsibility: Persistence adapter
Dependencies: domain only
Exposes:
  - StorageBackend (trait)
    - save_tasks(tasks: &HashMap<Uuid, Task>) -> Result<(), StorageError>
    - load_tasks() -> Result<HashMap<Uuid, Task>, StorageError>
    - save_projects(projects: &HashMap<ProjectId, Project>) -> Result<(), StorageError>
    - load_projects() -> Result<HashMap<ProjectId, Project>, StorageError>
    - save_users(users: &HashMap<UserId, User>) -> Result<(), StorageError>
    - load_users() -> Result<HashMap<UserId, User>, StorageError>
  - JsonFileStorage (stub impl)
  - StorageError { message: String }

### module: notification
Location: src/notification/mod.rs
Responsibility: Notification dispatch
Dependencies: domain only
Exposes:
  - NotificationService (trait)
    - notify(notification: &Notification)
    - register_hook(hook: Box<dyn Fn(&Notification) + Send + Sync>)
  - DefaultNotificationService (stub impl)

### module: query
Location: src/query/mod.rs
Responsibility: Filtering and search with caching
Dependencies: domain only
Exposes:
  - TaskQuery (trait)
    - filter_tasks(tasks: &HashMap<Uuid, Task>, filter: &TaskFilter) -> Vec<Uuid>
  - CachedTaskQuery (stub impl)

### module: analytics
Location: src/analytics/mod.rs
Responsibility: Event tracking and stats
Dependencies: domain only
Exposes:
  - AnalyticsCollector (trait)
    - track_event(event_type: &str, user_id: Option<&UserId>, project_id: Option<&ProjectId>)
    - flush() -> Vec<AnalyticsEvent>
    - get_project_stats(tasks: &HashMap<Uuid, Task>, project_id: &ProjectId) -> ProjectStats
  - BufferedAnalyticsCollector (stub impl)

---

## Layer 3: Use-Case Services (depend on domain + Layer 2 traits)

### module: task_service
Location: src/task_service/mod.rs
Responsibility: Task use cases (CRUD, status transitions, assignment)
Dependencies: domain, auth (trait), notification (trait), analytics (trait), query (trait)
Exposes:
  - TaskService (trait)
    - create_task(title: String, project_id: ProjectId, creator: &UserId) -> Result<Uuid, TaskServiceError>
    - update_status(task_id: Uuid, new_status: TaskStatus, actor: &UserId) -> Result<(), TaskServiceError>
    - assign(task_id: Uuid, assignee: UserId, actor: &UserId) -> Result<(), TaskServiceError>
    - delete(task_id: Uuid, actor: &UserId) -> Result<(), TaskServiceError>
    - get(task_id: Uuid) -> Result<Task, TaskServiceError>
  - TaskServiceImpl (stub impl)
  - TaskServiceError (enum: NotFound, DuplicateId, InvalidTransition, PermissionDenied, ValidationError)

### module: project_service
Location: src/project_service/mod.rs
Responsibility: Project lifecycle and membership
Dependencies: domain, auth (trait), notification (trait), analytics (trait)
Exposes:
  - ProjectService (trait)
    - create_project(name: String, owner: UserId) -> Result<ProjectId, ProjectServiceError>
    - add_member(project_id: &ProjectId, user_id: UserId, actor: &UserId) -> Result<(), ProjectServiceError>
  - ProjectServiceImpl (stub impl)
  - ProjectServiceError (enum: NotFound, PermissionDenied, ValidationError)

### module: user_service
Location: src/user_service/mod.rs
Responsibility: User registration and lookup
Dependencies: domain
Exposes:
  - UserService (trait)
    - register(name: String, email: String) -> Result<UserId, UserServiceError>
    - get(user_id: &UserId) -> Result<User, UserServiceError>
  - UserServiceImpl (stub impl)
  - UserServiceError (enum: NotFound, DuplicateEmail)
