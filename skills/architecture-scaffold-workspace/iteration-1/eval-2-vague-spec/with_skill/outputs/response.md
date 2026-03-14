# Architecture Scaffold: Phase 1 -- Clarification Required

I read through the assessment (ASSESSMENT_VAGUE.md) and the existing codebase (`src/lib.rs`), and I need to stop here before producing a module map or writing any code. The assessment is too underspecified to scaffold a sound architecture from. The skill's guiding principle is "ask early, ask often" -- every ambiguity resolved now prevents an order of magnitude of rework later.

Here is what the assessment gives us, what it leaves out, and the specific questions I need answered before proceeding.

---

## What the Assessment Actually Says

1. "The main engine class does too much" -- identifies `TaskFlowEngine` as a god object.
2. "Consider extracting some of the functionality into separate modules -- especially the data layer and maybe the notification stuff."
3. "The types could probably be organized better too."
4. "Think about separating concerns."
5. "The persistence logic shouldn't be mixed in with business logic."
6. "Maybe add some abstractions for the external-facing parts."

That is six sentences, three of which contain hedging language ("some", "maybe", "probably", "consider"). None of them define boundaries, specify types, or name concrete modules.

---

## What Is Missing (and What I Would Ask)

### 1. Module Boundaries -- What gets extracted, and what stays?

The assessment says to extract "the data layer" and "the notification stuff." But `TaskFlowEngine` currently handles at least **six distinct concerns**:

- Task CRUD (create, read, update, delete, status transitions)
- Project management (create project, add members)
- User management (register, lookup)
- Persistence (load/save JSON to disk)
- Notifications (hooks, dispatch)
- Analytics (event tracking, stats, buffer flushing)
- Filtering/Search (filter_tasks with caching)

**Questions I would ask:**

- Do you want each of these to become its own module, or should some stay grouped? For example, should Task CRUD and Project management live in the same "core" service, or be separate?
- "The data layer" -- does this mean just the JSON persistence (load_from_disk / save_to_disk), or does it also include the in-memory HashMap stores? Should there be a `Repository` trait abstracting storage?
- "Maybe the notification stuff" -- is this a definite yes or a "nice to have"? Does analytics extraction come along with it, or is analytics a separate concern?

### 2. Abstraction Strategy -- Traits or Concrete Types?

The assessment says "maybe add some abstractions for the external-facing parts." This is critical for the skeleton because it determines whether we write traits (dependency inversion) or just move concrete code into separate modules.

**Questions I would ask:**

- What are the "external-facing parts"? The persistence layer (so you can swap JSON for a real database)? The notification system (so you can swap hooks for a message queue)? The analytics buffer?
- Do you want trait-based abstractions (e.g., `trait TaskRepository`, `trait NotificationSender`) so implementations can be swapped? Or just module separation without abstraction boundaries?
- Is there a planned future integration (API layer, CLI, web framework) that would inform what the public API surface should look like?

### 3. Type Organization -- What Changes?

"The types could probably be organized better" -- but the current types are actually already fairly clean structs/enums. The real problem is they all live in one file alongside the god object.

**Questions I would ask:**

- Is the goal simply to move types into a dedicated `domain` module (pure extraction, no structural changes to the types themselves)?
- Or are there specific type changes you want -- e.g., splitting `TaskFlowError` into per-module error types (`StorageError`, `NotificationError`, etc.)?
- Should `UserId` and `ProjectId` become newtypes instead of bare `String` aliases? That would be a meaningful type-level change, not just reorganization.

### 4. Dependency Direction -- What Depends on What?

The assessment doesn't specify dependency rules. This is the single most important architectural decision because it determines what can compile independently.

**Questions I would ask:**

- Should persistence depend on domain types (repository knows about `Task`), or should domain types be persistence-agnostic (no `Serialize`/`Deserialize` on domain types, with separate DTO types for storage)?
- Should notification and analytics be injected into the task service, or should the task service emit events that notification/analytics subscribe to? (Push vs. pull model -- very different dependency graphs.)
- Is there a specific layering you have in mind? For example:
  - Layer 1: Domain types (no deps)
  - Layer 2: Service traits (depend on domain)
  - Layer 3: Concrete implementations (depend on traits + domain)
  - Or something flatter?

### 5. Scope of Change

**Questions I would ask:**

- Is this a full decomposition of `TaskFlowEngine` (it ceases to exist and is replaced by multiple services), or should there be a facade/coordinator that delegates to the extracted modules?
- What about the permission checking logic (`is_member`, `is_project_owner`, `is_valid_transition`)? Does that become its own module, stay with the task service, or get expressed as a separate policy layer?
- Should the filter cache remain an implementation detail of whatever module handles filtering, or does it need its own abstraction?

---

## What I Can Infer (But Need Confirmed)

Based on reading the code, here is my **best guess** at what a reasonable module map would look like. But I emphasize: this is inference from the code, not from the assessment. I would need the user to confirm this before writing any skeleton code.

**Likely modules:**

| Module | Responsibility | Source in Current Code |
|---|---|---|
| `domain` | Task, Project, User, TaskStatus, Priority, UserId, ProjectId, TaskFilter | Types at top of lib.rs (lines 13-122) |
| `storage` | Load/save to disk, storage error types | `load_from_disk`, `save_to_disk` (lines 156-218) |
| `task_service` | Task CRUD, status transitions, assignment, filtering | Lines 223-477 |
| `project_service` | Project CRUD, member management | Lines 481-534 |
| `user_service` | User registration, lookup | Lines 538-564 |
| `notification` | Notification types, hook registration, dispatch | Lines 86-101, 632-648 |
| `analytics` | Event tracking, stats, buffer | Lines 103-110, 568-628 |
| `auth` (or `policy`) | Permission checks, transition validation | `is_member`, `is_project_owner`, `is_valid_transition` (lines 652-674) |

**Likely dependency direction:**

```
domain (Layer 1) -- no project imports
storage, notification, analytics (Layer 2) -- import domain only
task_service, project_service, user_service (Layer 3) -- import domain + Layer 2 traits
```

But this is all speculation without confirmation. For example, maybe the user wants `task_service` and `project_service` merged. Maybe they want `auth` folded into the services. Maybe they want a coordinator/facade on top. I do not know, because the assessment does not say.

---

## Bottom Line

The assessment identifies a real problem (god object, mixed concerns) but provides no architectural target. It is a diagnosis without a prescription. To scaffold a compilable type skeleton, I need:

1. **An explicit list of target modules** (not "some functionality")
2. **Dependency rules between those modules** (what imports what)
3. **A decision on abstraction level** (traits vs. concrete modules)
4. **Confirmation on type restructuring scope** (move only, or also refactor types?)
5. **Clarification on the facade question** (does TaskFlowEngine survive as a coordinator, or does it get fully dissolved?)

Once I have answers to these questions, I can produce a precise module map (Phase 1), build a compilable skeleton (Phase 2), and map every stub back to existing code with confidence ratings (Phase 3). Without them, any skeleton I build would be my architectural opinion, not a scaffold of the user's intent -- and that defeats the purpose of this skill.
