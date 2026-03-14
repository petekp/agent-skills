# Migration Manifest

Source assessment: ASSESSMENT.md
Skeleton binary: scaffold-check (src/scaffold.rs)
Generated: 2026-03-05

---

## DefaultAuthPolicy

### can_create_task / can_update_task / can_assign_task
- **Source:** `TaskFlowEngine::is_member()` in `src/lib.rs:652-656`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Currently checks `project.members.contains(user_id)`. Straightforward extraction. Each service method currently calls `is_member` inline; the new AuthPolicy centralizes this.

### can_delete_task
- **Source:** `TaskFlowEngine::is_project_owner()` in `src/lib.rs:658-662`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Checks `project.owner == user_id`. Delete requires ownership, not just membership.

### can_add_member
- **Source:** Inline check in `TaskFlowEngine::add_project_member()` at `src/lib.rs:512-517`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Currently `&project.owner != actor` check. Move to AuthPolicy.

---

## JsonFileStorage (StorageBackend)

### save_tasks / save_projects / save_users
- **Source:** `TaskFlowEngine::save_to_disk()` in `src/lib.rs:199-219`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Split the monolithic save into per-collection methods. Each serializes with `serde_json::to_string_pretty` and writes to `{storage_path}/{collection}.json`.

### load_tasks / load_projects / load_users
- **Source:** `TaskFlowEngine::load_from_disk()` in `src/lib.rs:173-197`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Split the monolithic load. Each reads and deserializes one JSON file.

---

## DefaultNotificationService

### notify
- **Source:** `TaskFlowEngine::send_notification()` in `src/lib.rs:639-648`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Iterates hooks and calls each with the notification. Straightforward extraction. Remove the analytics tracking call (that moves to the caller/service layer).

### register_hook
- **Source:** `TaskFlowEngine::register_notification_hook()` in `src/lib.rs:632-637`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Direct push to hooks vec. Trivial.

---

## CachedTaskQuery (TaskQuery)

### filter_tasks
- **Source:** `TaskFlowEngine::filter_tasks()` in `src/lib.rs:408-477`
- **Action:** ADAPT
- **Confidence:** MEDIUM
- **Notes:** Logic is sound but currently takes `&mut self` because it writes to the cache. The new signature also takes `&mut self` (cache is owned by CachedTaskQuery). The filter logic itself is a direct port. The sort-by-priority-then-date behavior should be preserved. Cache invalidation is now handled by `CachedTaskQuery::invalidate()` instead of `TaskFlowEngine::invalidate_filter_cache()`.

---

## BufferedAnalyticsCollector (AnalyticsCollector)

### track_event
- **Source:** `TaskFlowEngine::track_event()` in `src/lib.rs:568-585`
- **Action:** REWRITE
- **Confidence:** MEDIUM
- **Notes:** The existing implementation is broken -- it takes `&self` but needs to mutate `analytics_buffer`. The comment says the real code uses `RefCell` interior mutability. The new implementation takes `&mut self`, which fixes the borrow issue cleanly. Write fresh with proper `self.buffer.push(event)`.

### flush
- **Source:** `TaskFlowEngine::flush_analytics()` in `src/lib.rs:587-589`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** `std::mem::take(&mut self.buffer)` -- trivial extraction.

### get_project_stats
- **Source:** `TaskFlowEngine::get_project_stats()` in `src/lib.rs:591-628`
- **Action:** ADAPT
- **Confidence:** HIGH
- **Notes:** Logic is self-contained. The new signature takes `&HashMap<Uuid, Task>` instead of reading from `self.tasks`. The computation (total, done, overdue, completion_rate) ports directly. Remove the `self.projects.contains_key` check -- that validation moves to the caller.

---

## TaskServiceImpl (TaskService)

### create_task
- **Source:** `TaskFlowEngine::create_task()` in `src/lib.rs:223-265`
- **Action:** ADAPT
- **Confidence:** HIGH
- **Notes:** Replace inline `is_member` check with `AuthPolicy::can_create_task`. Remove direct notification/analytics/auto_save calls -- use injected services instead. The task construction logic itself is a direct port.

### update_status
- **Source:** `TaskFlowEngine::update_task_status()` in `src/lib.rs:267-316`
- **Action:** ADAPT
- **Confidence:** MEDIUM
- **Notes:** Replace inline permission check with `AuthPolicy::can_update_task`. Replace `is_valid_transition` with `domain::is_valid_transition`. Notification dispatch moves to injected `NotificationService`. The double-borrow workaround (get immutable then get mutable) should be simplified in the new architecture since the task store is local.

### assign
- **Source:** `TaskFlowEngine::assign_task()` in `src/lib.rs:318-362`
- **Action:** ADAPT
- **Confidence:** HIGH
- **Notes:** Replace inline permission checks with AuthPolicy. Notification dispatch via injected service. The membership validation for the assignee (`is_member(&assignee, ...)`) becomes `AuthPolicy::can_create_task` or a dedicated check.

### delete
- **Source:** `TaskFlowEngine::delete_task()` in `src/lib.rs:370-404`
- **Action:** ADAPT
- **Confidence:** MEDIUM
- **Notes:** Replace `is_project_owner` with `AuthPolicy::can_delete_task`. Recursive subtask deletion logic ports directly. Remove analytics/auto_save calls.

### get
- **Source:** `TaskFlowEngine::get_task()` in `src/lib.rs:364-368`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Simple HashMap lookup. Returns cloned Task instead of reference (new signature returns `Task` not `&Task`).

---

## ProjectServiceImpl (ProjectService)

### create_project
- **Source:** `TaskFlowEngine::create_project()` in `src/lib.rs:481-499`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Project construction is straightforward. Remove analytics/auto_save calls.

### add_member
- **Source:** `TaskFlowEngine::add_project_member()` in `src/lib.rs:501-534`
- **Action:** ADAPT
- **Confidence:** HIGH
- **Notes:** Replace inline ownership check with `AuthPolicy::can_add_member`. Notification dispatch via injected service. Duplicate-member guard ports directly.

---

## UserServiceImpl (UserService)

### register
- **Source:** `TaskFlowEngine::register_user()` in `src/lib.rs:538-558`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Duplicate email check and user construction. Remove analytics/auto_save calls.

### get
- **Source:** `TaskFlowEngine::get_user()` in `src/lib.rs:560-564`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Simple HashMap lookup. Returns cloned User.

---

## Domain (is_valid_transition)

### is_valid_transition
- **Source:** `TaskFlowEngine::is_valid_transition()` in `src/lib.rs:664-674`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Already extracted as a free function in the skeleton. The `matches!` macro logic is identical. No longer takes `&self`.

---

## Items with no new home (DELETE candidates)

### TaskFlowEngine::auto_save
- **Source:** `src/lib.rs:680-682`
- **Action:** DELETE
- **Confidence:** HIGH
- **Notes:** No-op in the test fixture. In the new architecture, persistence is explicit via `StorageBackend` calls at the service layer, not implicit auto-save.

### TaskFlowEngine::invalidate_filter_cache
- **Source:** `src/lib.rs:676-678`
- **Action:** DELETE
- **Confidence:** HIGH
- **Notes:** Responsibility moves to `CachedTaskQuery::invalidate()`. The method itself was just `self.filter_cache.clear()`.

### TaskFlowEngine struct
- **Source:** `src/lib.rs:76-84`
- **Action:** DELETE
- **Confidence:** HIGH
- **Notes:** The god object itself. All its fields are distributed across the new services: tasks -> TaskServiceImpl, projects -> ProjectServiceImpl, users -> UserServiceImpl, storage_path -> JsonFileStorage, notification_hooks -> DefaultNotificationService, analytics_buffer -> BufferedAnalyticsCollector, filter_cache -> CachedTaskQuery.
