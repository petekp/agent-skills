# TaskFlow Architecture Assessment — Circular Variant

## Overview

TaskFlow needs to be split from a single god object into focused modules.

## Recommended Architecture

### 1. `domain` — Core types
- `Task`, `Project`, `User`, `TaskStatus`, `Priority`
- Pure data types, no logic

### 2. `task_service` — Task operations
- `TaskService` trait: `create_task`, `update_status`, `assign`, `delete`
- Depends on: `domain`, `project_service` (needs `ProjectService::get_project` to validate project exists and check membership)

### 3. `project_service` — Project operations
- `ProjectService` trait: `create_project`, `add_member`, `get_stats`
- Depends on: `domain`, `task_service` (needs `TaskService::filter_by_project` to compute project stats like task counts and completion rates)

### 4. `storage` — Persistence
- `StorageBackend` trait: `save`, `load`
- Depends on: `domain`

### 5. `notification` — Notification dispatch
- `NotificationService` trait: `notify`, `register_hook`
- Depends on: `domain`, `task_service` (needs task details for notification messages), `project_service` (needs project details for invite notifications)

## Dependency Rules

```
domain (Layer 1)           → nothing
storage (Layer 2)          → domain
task_service (Layer 3)     → domain, project_service
project_service (Layer 3)  → domain, task_service
notification (Layer 3)     → domain, task_service, project_service
```

Note: task_service and project_service depend on each other because tasks need project validation and projects need task queries for stats.
