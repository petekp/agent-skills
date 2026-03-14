# Phase 1 Gap Analysis: ASSESSMENT_VAGUE.md

Source assessment: `/Users/petepetrash/Code/agent-skills/skills/architecture-scaffold-workspace/test-project/ASSESSMENT_VAGUE.md`
Existing code: `/Users/petepetrash/Code/agent-skills/skills/architecture-scaffold-workspace/test-project/src/lib.rs`
Date: 2026-03-05

## Assessment Coverage Audit

The skill requires the assessment to provide enough information to identify: modules/components, responsibilities, types, dependencies, and boundaries. Here is what the assessment covers vs. what it omits.

### Provided by Assessment

| Requirement | Status | Evidence |
|---|---|---|
| Problem identification | PARTIAL | "main engine class does too much" -- names the god object but not its specific responsibilities |
| Module extraction targets | VAGUE | "the data layer" and "maybe the notification stuff" -- two of ~7 concerns, with hedging |
| Type organization | VAGUE | "types could probably be organized better" -- no specifics |
| Dependency direction | MISSING | Not mentioned at all |
| Boundary definitions | MISSING | Not mentioned at all |
| Abstraction strategy | VAGUE | "maybe add some abstractions for the external-facing parts" |
| Specific file/line references | MISSING | No code references whatsoever |
| Error type design | MISSING | Not mentioned |
| Target module count/granularity | MISSING | Not mentioned |

### Critical Gaps (Blocking -- Cannot Proceed to Skeleton)

1. **No module list.** The assessment names two extraction candidates ("data layer", "notification stuff") out of at least seven distinct concerns in the god object. The other five (task CRUD, project management, user management, analytics, filtering/search, permission checks) are not addressed.

2. **No dependency rules.** The assessment does not specify which modules should depend on which. This is the most important architectural decision -- it determines compilation order, testability, and whether the architecture can actually enforce separation.

3. **No abstraction decisions.** "Maybe add some abstractions" is not actionable. Trait-based abstraction vs. concrete module extraction produces fundamentally different skeletons with different type signatures.

4. **No type restructuring scope.** Moving types to a new file is trivial. Splitting `TaskFlowError` into per-module errors, converting `UserId`/`ProjectId` to newtypes, or removing `Serialize`/`Deserialize` from domain types are structural changes that affect every signature in the skeleton.

### Non-Blocking Gaps (Can Be Inferred from Code)

1. **Error types per module** -- Can be inferred from existing error variants and which methods produce which errors.
2. **Permission model placement** -- Can be inferred (it's a cross-cutting concern that probably stays with services or becomes its own policy module).
3. **Filter cache ownership** -- Implementation detail, can be decided during skeleton.

## Existing Codebase Inventory

The assessment provides no file/line references, so here is the inventory the skill would normally extract from a good assessment.

### Types (lines 13-132 of lib.rs)

| Type | Kind | Lines | Used By |
|---|---|---|---|
| `Task` | struct | 13-28 | Task CRUD, filtering, persistence, analytics |
| `TaskStatus` | enum | 30-38 | Task CRUD, filtering, transition validation |
| `Priority` | enum | 40-47 | Task CRUD, filtering, sorting |
| `UserId` | type alias (String) | 49 | Everything |
| `ProjectId` | type alias (String) | 50 | Everything |
| `Project` | struct | 52-60 | Project mgmt, permission checks, persistence |
| `User` | struct | 62-68 | User mgmt, persistence |
| `Notification` | struct | 86-92 | Notification dispatch |
| `NotificationKind` | enum | 94-101 | Notification dispatch |
| `AnalyticsEvent` | struct | 103-110 | Analytics tracking |
| `TaskFilter` | struct | 112-122 | Filtering/search |
| `TaskFlowError` | enum | 124-132 | Everything |
| `ProjectStats` | struct | 685-691 | Analytics |

### God Object Responsibilities (TaskFlowEngine, lines 76-683)

| Responsibility | Methods | Lines |
|---|---|---|
| Construction & persistence | `new`, `load_from_disk`, `save_to_disk` | 156-218 |
| Task CRUD | `create_task`, `update_task_status`, `assign_task`, `get_task`, `delete_task` | 223-404 |
| Filtering & search | `filter_tasks` | 408-477 |
| Project management | `create_project`, `add_project_member` | 481-534 |
| User management | `register_user`, `get_user` | 538-564 |
| Analytics | `track_event`, `flush_analytics`, `get_project_stats` | 568-628 |
| Notifications | `register_notification_hook`, `send_notification` | 632-648 |
| Permission checks | `is_member`, `is_project_owner`, `is_valid_transition` | 652-674 |
| Cache management | `invalidate_filter_cache` | 676-678 |
| Auto-save trigger | `auto_save` | 680-682 |

### Known Code Issues

1. **`track_event` takes `&self` but needs to mutate `analytics_buffer`** (line 568-585). Comment in code acknowledges this bug -- real code uses `RefCell`. This means the analytics extraction needs to decide on interior mutability vs. `&mut self`.

2. **`auto_save` is a no-op** (line 680-682). The skeleton needs to decide if persistence is caller-driven or automatic.

3. **`filter_cache` is invalidated on every write** (line 676-678). The cache strategy would need to live wherever filtering lives.

## Conclusion

Phase 1 cannot produce a confident module map from this assessment. The assessment identifies symptoms but prescribes no architecture. Proceeding to Phase 2 (skeleton) without resolving the blocking gaps would mean the agent is designing the architecture, not scaffolding the user's architecture. That violates the skill's core principle.

**Recommended next step:** Return to the user with the specific questions listed in `response.md` and get answers before proceeding.
