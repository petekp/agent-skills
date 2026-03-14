# Migration Manifest

Source assessment: ASSESSMENT_CIRCULAR.md
Skeleton binary: src/scaffold.rs (compiled as scaffold-check)
Generated: 2026-03-05

## domain (Layer 1)

Already scaffolded. Types extracted from the god object in lib.rs.

### Task, TaskStatus, Priority, Project, User
- **Source:** `src/lib.rs:13-68` (type definitions at the top of the god object file)
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Straightforward extraction -- these are pure data types with derives.

### Notification, NotificationKind
- **Source:** `src/lib.rs:86-101`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Pure data types.

### TaskFilter, ProjectStats, AnalyticsEvent
- **Source:** `src/lib.rs:103-122`, `src/lib.rs:685-691`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Pure data types.

### TaskFlowError
- **Source:** `src/lib.rs:124-151`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Error enum + Display + Error impls. Ported as-is.

### is_valid_transition()
- **Source:** `src/lib.rs:664-674` (method `TaskFlowEngine::is_valid_transition`)
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Extracted from instance method to free function. Pure logic, no dependencies.

---

## StorageBackend / JsonFileStorage (Layer 2 -- storage)

### save_tasks / load_tasks / save_projects / load_projects / save_users / load_users
- **Source:** `TaskFlowEngine::save_to_disk()` at `src/lib.rs:199-218` and `TaskFlowEngine::load_from_disk()` at `src/lib.rs:173-197`
- **Action:** ADAPT
- **Confidence:** HIGH
- **Notes:** The god object saves all three entity types in one method. The new trait splits this into per-entity methods. Logic is straightforward (serde_json read/write) but needs restructuring.

---

## TaskQuery / InMemoryTaskQuery (Layer 2 -- task_query)

### filter_tasks()
- **Source:** `TaskFlowEngine::filter_tasks()` at `src/lib.rs:408-477`
- **Action:** ADAPT
- **Confidence:** MEDIUM
- **Notes:** The existing implementation mutates a filter_cache via &mut self. The new trait returns owned Vec<Task> and doesn't own the cache. The filter logic itself is sound but the caching strategy needs rethinking. Also, the existing method returns Vec<&Task> (borrowed) while the new trait returns Vec<Task> (owned) -- this is a deliberate design choice to avoid lifetime entanglement across module boundaries.

### filter_by_project()
- **Source:** `TaskFlowEngine::get_project_stats()` at `src/lib.rs:591-628` (the inline filter)
- **Action:** ADAPT
- **Confidence:** HIGH
- **Notes:** The filter-by-project logic is inline in get_project_stats. Extract the filtering part as its own method.

### get_project_stats()
- **Source:** `TaskFlowEngine::get_project_stats()` at `src/lib.rs:591-628`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** This is the method that caused the circular dependency in the assessment. By placing it in task_query (Layer 2), both task_service and project_service can use it without depending on each other.

---

## TaskService / TaskServiceImpl (Layer 3 -- task_service)

### create_task()
- **Source:** `TaskFlowEngine::create_task()` at `src/lib.rs:223-265`
- **Action:** ADAPT
- **Confidence:** MEDIUM
- **Notes:** Currently validates project existence and membership by directly accessing self.projects. In the new architecture, this validation is done via the ProjectValidator trait, injected at construction time. The notification and analytics side effects also need to be wired differently.

### update_status()
- **Source:** `TaskFlowEngine::update_task_status()` at `src/lib.rs:267-316`
- **Action:** ADAPT
- **Confidence:** MEDIUM
- **Notes:** Uses is_valid_transition (now a domain function) and sends notifications inline. The notification dispatch needs to be delegated to NotificationService.

### assign()
- **Source:** `TaskFlowEngine::assign_task()` at `src/lib.rs:318-362`
- **Action:** ADAPT
- **Confidence:** MEDIUM
- **Notes:** Same pattern -- validates membership, mutates task, sends notification. Notification dispatch moves to NotificationService.

### delete()
- **Source:** `TaskFlowEngine::delete_task()` at `src/lib.rs:370-404`
- **Action:** ADAPT
- **Confidence:** MEDIUM
- **Notes:** Recursive subtask deletion. Permission check (is_project_owner) moves to ProjectValidator trait.

### get_task()
- **Source:** `TaskFlowEngine::get_task()` at `src/lib.rs:364-368`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Trivial lookup. Returns owned Task instead of reference.

---

## ProjectService / ProjectServiceImpl (Layer 3 -- project_service)

### create_project()
- **Source:** `TaskFlowEngine::create_project()` at `src/lib.rs:481-499`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Straightforward extraction.

### add_member()
- **Source:** `TaskFlowEngine::add_project_member()` at `src/lib.rs:501-534`
- **Action:** ADAPT
- **Confidence:** MEDIUM
- **Notes:** Sends a ProjectInvite notification inline. Notification dispatch moves to NotificationService.

### get_project()
- **Source:** No dedicated method in god object, but `self.projects.get(project_id)` is used throughout.
- **Action:** NEW
- **Confidence:** HIGH
- **Notes:** Simple lookup, but didn't exist as a standalone method.

### get_stats()
- **Source:** `TaskFlowEngine::get_project_stats()` at `src/lib.rs:591-628`
- **Action:** ADAPT
- **Confidence:** HIGH
- **Notes:** Delegates to TaskQuery::get_project_stats. This is the method that required the circular dependency in the original assessment.

### is_member() / is_owner()
- **Source:** `TaskFlowEngine::is_member()` at `src/lib.rs:652-656`, `TaskFlowEngine::is_project_owner()` at `src/lib.rs:658-661`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Simple lookups. Also implement the ProjectValidator trait from task_service.

---

## NotificationService / NotificationServiceImpl (Layer 3 -- notification)

### notify()
- **Source:** `TaskFlowEngine::send_notification()` at `src/lib.rs:639-648`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Iterates over hooks and calls each one. Currently also tracks analytics; that side effect can be removed or wired separately.

### register_hook()
- **Source:** `TaskFlowEngine::register_notification_hook()` at `src/lib.rs:632-637`
- **Action:** PORT
- **Confidence:** HIGH
- **Notes:** Simple push to a Vec.

---

## Items NOT mapped (intentionally omitted from the new architecture)

### Analytics (track_event, flush_analytics, AnalyticsEvent)
- **Source:** `src/lib.rs:568-589`
- **Action:** DELETE (from active architecture) or defer to a future module
- **Confidence:** MEDIUM
- **Notes:** The assessment does not mention analytics as a target module. The existing track_event is broken (takes &self but needs &mut self). Recommend deferring analytics to a separate future effort. AnalyticsEvent is in domain types for now but has no service consumer.

### User management (register_user, get_user)
- **Source:** `src/lib.rs:538-564`
- **Action:** DELETE (from this scaffolding pass) or defer
- **Confidence:** LOW
- **Notes:** The assessment does not mention a user_service module. User management exists in the god object but isn't called out in the target architecture. This is a gap in the assessment that should be clarified with the user.

### Filter cache (invalidate_filter_cache, filter_cache field)
- **Source:** `src/lib.rs:676-678`
- **Action:** DELETE
- **Confidence:** HIGH
- **Notes:** The caching strategy was tightly coupled to the god object's mutable state. The new task_query module should implement its own caching strategy (or none) as an implementation detail.
