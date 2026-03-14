# Module Map

Source assessment: ASSESSMENT_CIRCULAR.md

## CIRCULAR DEPENDENCY DETECTED -- ASSESSMENT REQUIRES CORRECTION

The assessment declares:
- `task_service` depends on `project_service` (needs `ProjectService::get_project` to validate project exists and check membership)
- `project_service` depends on `task_service` (needs `TaskService::filter_by_project` to compute project stats)

This is a circular dependency. Both modules are at "Layer 3" in the assessment, but they form a cycle: task_service -> project_service -> task_service. This cannot compile in Rust (no circular `use crate::` references between sibling modules that depend on each other's traits). Even if it could be made to compile via trait objects or generics, it is an architectural smell -- it means the module boundaries are drawn incorrectly.

### Root cause analysis

The cycle exists because `project_service` needs task data (to compute stats like task counts and completion rates), and `task_service` needs project data (to validate that a project exists and that a user is a member). These are two distinct concerns:

1. **Task operations need project validation** -- "does this project exist? is this user a member?" This is a read-only query against project data.
2. **Project stats need task queries** -- "how many tasks are in this project? what's the completion rate?" This is a read-only query against task data.

### Resolution: Extract a `task_query` module

Break the cycle by extracting the read-only task query capability into its own module at Layer 2. `project_service` depends on `task_query` (not `task_service`) for stats. `task_service` depends on project data via a trait, also at Layer 2.

Alternatively, `project_service::get_stats` could accept task data as a parameter rather than reaching into another service -- but extracting `task_query` is cleaner and matches the existing `filter_tasks` functionality in the god object.

---

## Corrected Module Map

### Layer 1: Domain Types (no dependencies on other project modules)

#### module: domain
Location: `src/domain/mod.rs`
Responsibility: Shared value types, identities, enums, error types, and business rules used across all modules.
Types:
  - `Task` { id, title, description, status, priority, assignee, project_id, labels, created_at, updated_at, due_date, parent_id, subtask_ids }
  - `TaskStatus` (enum: Backlog, Todo, InProgress, InReview, Done, Cancelled)
  - `Priority` (enum: None, Low, Medium, High, Urgent)
  - `Project` { id, name, description, owner, members, created_at }
  - `User` { id, name, email, avatar_url }
  - `Notification` { kind, user_id, message, timestamp }
  - `NotificationKind` (enum: TaskAssigned, TaskStatusChanged, TaskDueSoon, MentionedInComment, ProjectInvite)
  - `TaskFilter` { status, priority, assignee, project_id, labels, due_before, due_after, search_text }
  - `ProjectStats` { total_tasks, completed_tasks, overdue_tasks, completion_rate }
  - `TaskFlowError` (enum: NotFound, DuplicateId, InvalidTransition, PermissionDenied, StorageError, ValidationError)
  - `UserId` (type alias = String)
  - `ProjectId` (type alias = String)
  - `is_valid_transition(from, to) -> bool` (pure business rule)
Dependency rule: Imports nothing from the project.

### Layer 2: Storage and Query Contracts (depend only on domain)

#### module: storage
Location: `src/storage/mod.rs`
Responsibility: Persistence -- save and load domain objects to/from disk.
Dependencies: `domain` only
Exposes:
  - `StorageBackend` (trait)
    - `fn save_tasks(&self, tasks: &HashMap<Uuid, Task>) -> Result<(), TaskFlowError>`
    - `fn load_tasks(&self) -> Result<HashMap<Uuid, Task>, TaskFlowError>`
    - `fn save_projects(&self, projects: &HashMap<ProjectId, Project>) -> Result<(), TaskFlowError>`
    - `fn load_projects(&self) -> Result<HashMap<ProjectId, Project>, TaskFlowError>`
    - `fn save_users(&self, users: &HashMap<UserId, User>) -> Result<(), TaskFlowError>`
    - `fn load_users(&self) -> Result<HashMap<UserId, User>, TaskFlowError>`

#### module: task_query
Location: `src/task_query/mod.rs`
Responsibility: Read-only task filtering and querying. This is the module that breaks the circular dependency -- it provides task query capabilities that both `task_service` and `project_service` can depend on without depending on each other.
Dependencies: `domain` only
Exposes:
  - `TaskQuery` (trait)
    - `fn filter_tasks(&self, filter: &TaskFilter) -> Vec<Task>`
    - `fn filter_by_project(&self, project_id: &ProjectId) -> Vec<Task>`
    - `fn get_project_stats(&self, project_id: &ProjectId) -> Result<ProjectStats, TaskFlowError>`

### Layer 3: Service Contracts (depend on domain + Layer 2)

#### module: task_service
Location: `src/task_service/mod.rs`
Responsibility: Task CRUD operations with validation and authorization.
Dependencies: `domain`, `task_query` (for query delegation)
Exposes:
  - `TaskService` (trait)
    - `fn create_task(&mut self, title: String, project_id: ProjectId, creator: &UserId) -> Result<Uuid, TaskFlowError>`
    - `fn update_status(&mut self, task_id: Uuid, new_status: TaskStatus, actor: &UserId) -> Result<(), TaskFlowError>`
    - `fn assign(&mut self, task_id: Uuid, assignee: UserId, actor: &UserId) -> Result<(), TaskFlowError>`
    - `fn delete(&mut self, task_id: Uuid, actor: &UserId) -> Result<(), TaskFlowError>`
    - `fn get_task(&self, task_id: Uuid) -> Result<Task, TaskFlowError>`
Note: Task creation needs to validate project existence and membership. Rather than depending on `project_service`, this is accomplished by accepting a `ProjectValidator` trait (defined in domain or here) or by having the caller pass validated project data.

#### module: project_service
Location: `src/project_service/mod.rs`
Responsibility: Project CRUD and membership management.
Dependencies: `domain`, `task_query` (for stats computation)
Exposes:
  - `ProjectService` (trait)
    - `fn create_project(&mut self, name: String, owner: UserId) -> Result<ProjectId, TaskFlowError>`
    - `fn add_member(&mut self, project_id: &ProjectId, user_id: UserId, actor: &UserId) -> Result<(), TaskFlowError>`
    - `fn get_project(&self, project_id: &ProjectId) -> Result<Project, TaskFlowError>`
    - `fn get_stats(&self, project_id: &ProjectId) -> Result<ProjectStats, TaskFlowError>`
    - `fn is_member(&self, user_id: &UserId, project_id: &ProjectId) -> bool`
    - `fn is_owner(&self, user_id: &UserId, project_id: &ProjectId) -> bool`

#### module: notification
Location: `src/notification/mod.rs`
Responsibility: Notification dispatch to registered hooks.
Dependencies: `domain`
Exposes:
  - `NotificationService` (trait)
    - `fn notify(&self, notification: Notification)`
    - `fn register_hook(&mut self, hook: Box<dyn Fn(&Notification) + Send + Sync>)`

## Dependency Rules (corrected)

```
Layer 1: domain           -> nothing
Layer 2: storage           -> domain
Layer 2: task_query        -> domain
Layer 3: task_service      -> domain, task_query
Layer 3: project_service   -> domain, task_query
Layer 3: notification      -> domain
```

No cycles. The dependency graph is a DAG.

## What changed from the assessment

1. **Added `task_query` module** (Layer 2) -- extracts the read-only query/filter/stats logic that both task_service and project_service need.
2. **Removed the `task_service <-> project_service` cycle** -- project_service gets stats via task_query, not task_service.
3. **Project validation for task operations** -- task_service validates project existence/membership via a `ProjectValidator` trait or by accepting validated data, rather than importing project_service directly.
4. **Moved storage to Layer 2** -- it only depends on domain types, so it belongs alongside task_query.
