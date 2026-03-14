# Architecture Scaffold Response

## What I'd Ask You First (If This Were Interactive)

The assessment says "separate concerns" and "extract some of the functionality into separate modules" but leaves a lot of design decisions open. Before committing to an architecture, here's what I'd want to clarify:

1. **Storage backend flexibility**: The review says "persistence logic shouldn't be mixed in with business logic." Do you want a trait-based storage abstraction so you can swap JSON files for SQLite/Postgres later, or is file-based storage the permanent plan?
   - **My assumption**: You want a `Storage` trait. The current JSON-file approach becomes one implementation.

2. **Notification dispatch model**: The review mentions "maybe add some abstractions for the external-facing parts." Are notifications going to stay as in-process callback hooks, or do you plan to add email, Slack, webhooks, etc.?
   - **My assumption**: Introduce a `NotificationSink` trait so you can register multiple backends.

3. **Analytics destination**: The analytics buffer currently has a broken `&self` mutation issue. Should analytics be fire-and-forget (channel-based), or do you need synchronous buffering?
   - **My assumption**: Move analytics behind a trait as well, fix the mutability bug.

4. **Public API surface**: Should the refactored crate expose a single facade (like a cleaned-up `TaskFlowEngine`), or should consumers be able to use individual services (e.g., use `TaskService` directly without `ProjectService`)?
   - **My assumption**: Keep a thin facade/coordinator, but make the individual services independently testable.

5. **Error granularity**: Should each module have its own error type, or keep a single unified error enum?
   - **My assumption**: Keep a single `TaskFlowError` for now since the crate is small, but organize variants by domain.

## What I Built

Given those assumptions, I created a modular architecture that breaks the god object into focused modules. See the scaffolded files in this directory:

- `architecture.md` -- Full architecture plan with module map, dependency diagram, and migration strategy
- `src/` directory stubs showing the target file structure with type signatures and doc comments

The existing `src/lib.rs` is **not modified**. The new files represent the target state after refactoring.

## Key Design Decisions

- **5 modules**: `models`, `storage`, `services`, `notifications`, `analytics`
- **Trait-based boundaries** for storage, notifications, and analytics (testable, swappable)
- **Services own business logic** but not persistence or side effects -- those are injected
- **The `track_event(&self)` mutability bug** is fixed by moving analytics behind an `AnalyticsCollector` trait (implementations can use channels or `Mutex` internally)
- **Filter cache** moves into `TaskService` where it belongs, scoped to task operations only
