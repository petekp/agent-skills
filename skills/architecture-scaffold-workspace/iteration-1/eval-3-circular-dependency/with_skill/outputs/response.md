# Circular Dependency Detection and Resolution

## What was found

The assessment (ASSESSMENT_CIRCULAR.md) contains a circular dependency between `task_service` and `project_service`:

- `task_service` depends on `project_service` because it needs `ProjectService::get_project` to validate that a project exists and check user membership before creating/modifying tasks.
- `project_service` depends on `task_service` because it needs `TaskService::filter_by_project` to compute project stats (task counts, completion rates).

The assessment even acknowledges this in a note: "task_service and project_service depend on each other because tasks need project validation and projects need task queries for stats."

This is not a valid architecture. In Rust, circular `use crate::` imports between sibling modules create a compilation failure. Even in languages where circular imports compile (e.g., via forward declarations or runtime resolution), a cycle between two service modules is an architectural smell indicating the boundaries are drawn incorrectly.

## How it was detected

The skill's Phase 1 (Extract the Target Architecture) requires producing a module map with explicit dependency rules per layer. When writing the dependency graph:

```
task_service -> domain, project_service
project_service -> domain, task_service
```

The cycle is immediately visible. The skill specifically calls this out: "Circular dependency between modules means the module boundaries are drawn wrong. Stop. Go back to the module map. Restructure until the dependency graph is a DAG."

This detection happened before any code was written, during the module map phase. The skill's emphasis on writing down dependency rules as an explicit step makes the cycle impossible to miss.

## How it would be resolved

### Root cause

Both services need read-only access to the other's data. task_service needs to validate project existence/membership (a read of project data). project_service needs to query tasks for statistics (a read of task data). Neither actually needs the other's *mutation* capabilities.

### Resolution: Extract `task_query` (Layer 2)

The read-only task query capability was extracted into a new `task_query` module at Layer 2 (below both services). This module provides:

- `TaskQuery::filter_tasks()` -- general filtering
- `TaskQuery::filter_by_project()` -- project-scoped task listing
- `TaskQuery::get_project_stats()` -- aggregate statistics

This is the method `project_service` actually needed from `task_service`. By putting it at Layer 2, `project_service` depends on `task_query` (downward), not `task_service` (sideways/circular).

For the other direction (task_service needing project validation), a `ProjectValidator` trait was defined in task_service itself. This is a dependency inversion: task_service declares what it needs (the trait), and project_service implements it. At runtime, a ProjectServiceImpl is passed as the ProjectValidator to TaskServiceImpl. No compile-time dependency from task_service to project_service exists.

### Corrected dependency graph

```
domain (Layer 1)           -> nothing
storage (Layer 2)          -> domain
task_query (Layer 2)       -> domain
task_service (Layer 3)     -> domain, task_query
project_service (Layer 3)  -> domain, task_query
notification (Layer 3)     -> domain
```

No cycles. The graph is a DAG. This was verified mechanically by grepping all `use crate::` statements in the skeleton and confirming no module imports from a peer or higher layer.

## Compilation proof

The skeleton compiles cleanly with `cargo check --bin scaffold-check`:
- 0 errors
- 23 warnings (all dead_code, expected for a type skeleton)
- 6 modules, 6 traits, 5 stub implementations
- Original `src/lib.rs` was not modified

## Artifacts produced

1. **module-map.md** -- The corrected module map with the circular dependency analysis and resolution
2. **skeleton-verification.md** -- Compilation and dependency verification results
3. **migration-manifest.md** -- Per-stub mapping from old god object to new modules with action/confidence ratings
4. **Skeleton code** in the test project:
   - `src/scaffold.rs` -- Binary entry point declaring all skeleton modules
   - `src/domain/mod.rs` -- Layer 1 domain types (pre-existing)
   - `src/storage/mod.rs` -- Layer 2 persistence trait
   - `src/task_query/mod.rs` -- Layer 2 query trait (the cycle-breaker)
   - `src/task_service/mod.rs` -- Layer 3 task operations with ProjectValidator trait
   - `src/project_service/mod.rs` -- Layer 3 project operations
   - `src/notification/mod.rs` -- Layer 3 notification dispatch
