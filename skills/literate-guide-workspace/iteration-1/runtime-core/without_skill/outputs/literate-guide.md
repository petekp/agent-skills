# The Capacitor Runtime Core: A Literate Walkthrough

*A narrative guide to `capacitor-core`, the Rust crate that gives Capacitor its mind.*

---

## Prologue: What Is This Thing?

Capacitor is a desktop HUD (heads-up display) for Claude Code. It sits alongside your terminal, watching Claude sessions start and stop, tracking which projects are active, displaying token usage statistics, and routing focus to the right window. The beating heart of all of this is `capacitor-core` -- a single Rust crate that owns every domain decision.

The crate compiles three ways: as a normal Rust library, as a C-compatible static library, and as a C-compatible dynamic library. This triple-target is not vanity -- it exists because of UniFFI, Mozilla's foreign function interface generator. Every public type in the crate is annotated with `#[uniffi::Record]` or `#[uniffi::Enum]`, and the main `CoreRuntime` object is annotated with `#[uniffi::Object]`. UniFFI reads these annotations and generates Swift, Kotlin, and Python bindings automatically. This means the macOS app, a potential mobile client, and a potential TUI all share the exact same domain logic. There is no "Swift version of the truth" -- there is only the Rust version.

```rust
// core/capacitor-core/Cargo.toml
[lib]
crate-type = ["lib", "staticlib", "cdylib"]
name = "capacitor_core"
```

```rust
// src/lib.rs -- the first line that matters
uniffi::setup_scaffolding!();
```

With that single macro invocation, UniFFI generates all the FFI scaffolding: the C functions, the Swift wrapper classes, the type converters. Every `pub` function annotated with `#[uniffi::export]` becomes callable from Swift as if it were native.

---

## Part I: The Storage Foundation

Before we can understand what the runtime *does*, we need to understand where it *keeps things*. Capacitor maintains two separate directory trees on disk:

1. **`~/.capacitor/`** -- Capacitor's own data: pinned projects, idea files, stats caches, agent registries.
2. **`~/.claude/`** -- Claude Code's data, which Capacitor reads but never writes: JSONL session transcripts, plugin registries, settings.

The `StorageConfig` struct in `runtime_storage.rs` is the single source of truth for all paths. Production code uses `StorageConfig::default()`, which points to the real home directory. Tests use `StorageConfig::with_root(temp_dir)` to get a completely isolated filesystem:

```rust
pub struct StorageConfig {
    root: PathBuf,       // ~/.capacitor
    claude_root: PathBuf, // ~/.claude
}

impl Default for StorageConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
        Self {
            root: home.join(".capacitor"),
            claude_root: home.join(".claude"),
        }
    }
}
```

This design has a subtle but critical consequence: the entire crate is testable without touching the real filesystem. Every function that needs a path takes a `&StorageConfig` parameter rather than computing paths itself. The dependency is injected, not assumed.

### Path Encoding

Capacitor needs to store per-project data inside `~/.capacitor/projects/`, but filesystem paths contain slashes, which cannot appear in directory names. The solution is a versioned percent-encoding scheme:

```rust
// /Users/pete/Code/my-project --> p2_%2FUsers%2Fpete%2FCode%2Fmy-project
const ENCODED_PREFIX: &'static str = "p2_";
```

The `p2_` prefix is a version marker. If the encoding scheme ever changes, old directories can be detected and migrated. The encoding is lossless -- round-tripping through `encode_path` and `decode_path` always recovers the original. This matters because paths with hyphens (like `/Users/pete/my-project`) must not be confused with the separator character.

---

## Part II: The Domain Model

The `domain/` module contains the core types that describe the world as Capacitor sees it. These types are reused everywhere: in the reducer, in the FFI exports, in the service layer. They are the shared vocabulary.

### Session State

A Claude Code session is always in one of five states:

```rust
pub enum SessionState {
    Working,     // Claude is generating a response
    Ready,       // Claude has finished and is waiting for input
    Idle,        // No recent activity
    Compacting,  // Claude is compacting its context window
    Waiting,     // Claude is waiting for user approval (e.g., permission request)
}
```

Each state has a priority (used to determine which session "represents" a project) and an `is_active` predicate. `Waiting` is the highest priority because it demands human attention; `Idle` is the lowest because nothing is happening:

```rust
impl SessionState {
    pub fn priority(self) -> u8 {
        match self {
            Self::Waiting => 4,
            Self::Working => 3,
            Self::Compacting => 2,
            Self::Ready => 1,
            Self::Idle => 0,
        }
    }
}
```

### The AppSnapshot

The `AppSnapshot` is the runtime's complete view of the world at a point in time. It is the single document that the UI reads to render itself:

```rust
pub struct AppSnapshot {
    pub projects: Vec<ProjectSummary>,
    pub sessions: Vec<SessionSummary>,
    pub shells: Vec<ShellSignal>,
    pub routing: Vec<RoutingView>,
    pub diagnostics: DiagnosticsSummary,
    pub generated_at: String,
}
```

This is an event-sourced architecture in miniature. The snapshot is not the source of truth -- the `ReducerState` is. The snapshot is a *projection* of that state, serialized for consumption by clients and for persistence to disk. When the runtime restarts, it rehydrates its `ReducerState` from the last persisted snapshot.

### Project Identity

How does Capacitor know that two different file paths belong to the same project? The `identity.rs` module solves this with a boundary-detection algorithm. Starting from a file path, it walks up the directory tree looking for project markers:

```rust
const PROJECT_MARKERS: &[(&str, u8)] = &[
    ("CLAUDE.md", 1),      // Explicit project marker, highest priority
    ("package.json", 2),   // Package manifest
    ("Cargo.toml", 2),     // Package manifest
    // ... more markers ...
    (".git", 3),           // Repository root
    ("Makefile", 4),       // Build system, lowest priority
];
```

Priority 1 (`CLAUDE.md`) is special: if found, it wins unless a *nearer* package marker was already discovered. This handles the monorepo case where the repo root has a `CLAUDE.md` but you're editing a file inside `packages/auth/`, which has its own `package.json`. In that case, `packages/auth/` is the correct project boundary.

Once the project boundary is found, Capacitor resolves git worktree relationships. Two git worktrees that share the same `.git/commondir` will produce the same `project_id`. This means switching between worktrees doesn't create a duplicate project in the HUD. The `workspace_id` is derived from an MD5 hash of the project identity and the relative worktree path:

```rust
pub fn workspace_id(project_id: &str, project_path: &str) -> String {
    let relative = workspace_relative_path(&project_id, &project_path);
    let source = format!("{}|{}", project_id, relative);
    format!("{:x}", md5::compute(source))
}
```

### Hook Events and Shell Signals

The runtime has two input channels. **Hook events** come from Claude Code itself, via hooks that fire at various lifecycle points (session start, tool use, prompt submission, etc.). **Shell signals** come from a shell integration that reports which terminal or IDE is hosting each shell process.

```rust
pub enum HookEventType {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionRequest,
    PreCompact,
    Notification,
    SubagentStart,
    SubagentStop,
    Stop,
    // ... and more
}
```

Each hook event carries a `session_id`, `project_path`, and a timestamp. Shell signals carry a process ID, working directory, TTY, and the name of the parent application (Ghostty, iTerm2, VS Code, etc.).

---

## Part III: The Data Pipeline

The runtime processes incoming data through a four-stage pipeline: **Ingest**, **Observation**, **Reduce**, **Projection**. This is a simplified event-sourcing architecture.

### Stage 1: Ingest

The `ingest/` module is pure normalization. It takes raw hook events and shell signals and cleans them up: trimming whitespace, normalizing paths (trailing slashes, case-folding on macOS), and discarding empty optional fields:

```rust
pub fn normalize_hook_event(command: IngestHookEventCommand) -> IngestHookEventCommand {
    IngestHookEventCommand {
        event_id: command.event_id.trim().to_string(),
        project_path: normalize_required_path(&command.project_path),
        cwd: normalize_optional_path(command.cwd),
        workspace_id: normalize_optional_text(command.workspace_id),
        // ... etc
    }
}
```

This is a classic data-pipeline pattern: validate at the boundary, normalize immediately, and pass clean data downstream. No other module needs to worry about whitespace or trailing slashes.

### Stage 2: Observation

The `observation/` module wraps normalized commands into `ObservationRecord` structs. An observation is a timestamped, classified input with an idempotency key:

```rust
pub struct ObservationRecord {
    pub source_kind: ObservationSourceKind,  // ClaudeHook or ShellSignal
    pub recorded_at: String,
    pub idempotency_key: String,             // "hook:evt-1:session-1:2026-..."
    pub payload: ObservationPayload,
}
```

The idempotency key is composed from the event's natural identifiers. If the same event arrives twice (network retry, duplicate webhook), the key lets the system detect and skip the duplicate. This is the write-ahead log of the event-sourced system, though in the current implementation it primarily serves as a structural boundary between ingestion and reduction.

### Stage 3: Reduce

The `reduce/` module is where the real work happens. It maintains a `ReducerState` -- the authoritative in-memory model of all tracked projects, sessions, shells, and routing decisions:

```rust
pub struct ReducerState {
    pub projects: BTreeMap<String, ProjectSummary>,
    pub sessions: HashMap<String, SessionSummary>,
    pub shells: HashMap<u32, ShellSignal>,
    pub routing: BTreeMap<String, RoutingView>,
    pub events_ingested: u64,
    // ... diagnostics counters
}
```

When a hook event arrives, the reducer follows a strict sequence:

1. **Validate**: reject events with missing `event_id` or `session_id`.
2. **Record heartbeat**: update `last_hook_event_at` for health monitoring.
3. **Stale check**: if the event's timestamp is older than the session's current state, skip it. A 5-second grace period prevents clock-skew rejections.
4. **Session reduction**: compute the new session state based on the event type.
5. **Project recomputation**: recalculate every project's aggregate state from its sessions.
6. **Routing recomputation**: determine which terminal/tmux-pane each project should route to.

The session reducer is a state machine. Each event type maps to a new state:

- `SessionStart` or `UserPromptSubmit` --> `Working`
- `Stop` or `TaskCompleted` --> `Ready` (with a `ready_reason`)
- `PreCompact` --> `Compacting`
- `PermissionRequest` --> `Waiting`
- `PreToolUse` --> `Working`, incrementing `tools_in_flight`
- `PostToolUse` --> `Working`, decrementing `tools_in_flight`
- `SessionEnd` --> session is deleted entirely
- `Notification`, `SubagentStart`, `SubagentStop` --> informational, skipped by the reducer

After the session state is updated, `recompute_projects()` walks all sessions, groups them by project path, and determines each project's aggregate state. The project takes the state of its highest-priority session (by `SessionState::priority()`). If multiple sessions share the highest priority, the most recently updated one becomes the "representative session."

### Stage 4: Projection and Query

The `projection/` module defines a `SnapshotReadModelProjector` -- a component that takes a snapshot and a list of observations and produces a read model. In the current implementation this is lightweight (the snapshot is the read model), but the abstraction exists to support future projectors that might compute different views of the same observation stream.

The `query/` module is a single function:

```rust
pub fn app_snapshot(state: &ReducerState) -> AppSnapshot {
    state.snapshot()
}
```

This is deliberately thin. The query layer is the seam where you'd add filtering, pagination, or access control in a larger system. Today it's just a pass-through.

---

## Part IV: The CoreRuntime Object

The `CoreRuntime` struct in `lib.rs` is the god object -- the single entry point that FFI clients use for everything. It owns the reducer state behind a mutex, a snapshot storage backend, and the app's storage configuration:

```rust
#[derive(uniffi::Object)]
pub struct CoreRuntime {
    state: std::sync::Mutex<reduce::ReducerState>,
    snapshot_storage: Arc<dyn SnapshotStorage>,
    app_storage: StorageConfig,
}
```

Every mutation follows the same pattern: lock the state, apply the change, take a snapshot, unlock, persist the snapshot. This is a coarse-grained locking strategy -- only one mutation can proceed at a time -- but it guarantees consistency and keeps the code simple.

```rust
pub fn ingest_hook_event(
    &self,
    command: IngestHookEventCommand,
) -> Result<MutationOutcome, CoreRuntimeError> {
    let normalized = ingest::normalize_hook_event(command);
    let mut state = self.lock_state()?;
    let outcome = state.apply_hook_event(normalized);
    let snapshot = state.snapshot();
    drop(state);                    // release the lock before I/O
    self.persist_snapshot(&snapshot)?;
    Ok(outcome)
}
```

Note the careful ordering: the lock is dropped *before* persisting to disk. This ensures that the mutex is not held during potentially slow I/O, keeping the system responsive to concurrent requests.

### Constructors

The runtime has two constructors:

```rust
// In-memory snapshot storage (for embedded/testing use)
pub fn new() -> Result<Arc<Self>, CoreRuntimeError>

// File-backed snapshot storage (for production)
pub fn new_with_snapshot_file(snapshot_file: String) -> Result<Arc<Self>, CoreRuntimeError>
```

Both constructors attempt to load a persisted snapshot on startup. If one exists, the reducer state is rehydrated from it. If not, the reducer starts empty. This means the runtime survives process restarts -- session state is preserved across app relaunches.

---

## Part V: The Hook System

Capacitor observes Claude Code by injecting hooks into Claude's lifecycle. Claude Code supports hooks that fire shell commands or HTTP requests at specific points during a session. Capacitor registers its own hooks in Claude's `settings.json`, and those hooks forward events to the Capacitor runtime.

### Hook Contracts

The `runtime_contracts/claude_hooks.rs` module defines every hook event that Claude Code supports and which transport mechanisms are allowed for each:

```rust
pub struct ClaudeHookEventContract {
    pub event_name: &'static str,
    pub allowed_transports: &'static [HookTransport],
    pub managed_transport: Option<HookTransport>,
    pub needs_matcher: bool,
}
```

The `managed_transport` field indicates how Capacitor should configure this hook. `SessionStart` uses `Command` transport (a shell command), while `UserPromptSubmit` uses `Http` transport (a POST to `http://127.0.0.1:7474/hook`). Some events like `InstructionsLoaded` have no managed transport -- Capacitor doesn't need to observe them.

Events that `needs_matcher` is true for (like `PreToolUse` and `PostToolUse`) require a tool-name matcher pattern in the hook configuration, allowing selective observation of specific tools.

### Hook Setup

The `runtime_setup.rs` module handles installing, validating, and removing hooks. The `SetupChecker` reads Claude's `settings.json`, checks for a `hud-hook` binary on the PATH, and determines whether hooks are properly configured. It follows the "sidecar principle" -- it only mutates Capacitor-managed hook entries while preserving all other user settings.

The installer uses atomic writes (write to temp file, then rename) to prevent corrupting `settings.json` if the process is interrupted mid-write.

Hook status is modeled as an enum that captures all the failure modes:

```rust
pub enum HookStatus {
    NotInstalled,
    Installed { version: String },
    PolicyBlocked { reason: String },
    BinaryBroken { reason: String },
    SymlinkBroken { target: String, reason: String },
}
```

Policy blocking occurs when Claude's settings contain `disableAllHooks` or `allowManagedHooksOnly` flags. These are hard failures that Capacitor cannot auto-fix.

### Hook Health Monitoring

Even when hooks are installed, they might stop firing. The runtime monitors hook health by tracking the age of the most recent hook event. If no event has been seen for more than 60 seconds, hooks are considered "stale." A grace period of 300 seconds is applied when an active session is running, since some operations (like long tool executions) produce no hook events for extended periods.

```rust
fn heartbeat_status(
    age_secs: u64,
    threshold_secs: u64,
    grace_secs: u64,
    has_active_session: bool,
) -> HookHealthStatus {
    if age_secs <= threshold_secs {
        return HookHealthStatus::Healthy;
    }
    if has_active_session && age_secs <= grace_secs {
        return HookHealthStatus::Healthy;
    }
    HookHealthStatus::Stale { last_seen_secs: age_secs }
}
```

The `HookDiagnosticReport` combines installation status and runtime health into a single report for the UI. It determines the `primary_issue` (the most critical problem to display) and whether `can_auto_fix` is possible:

```rust
pub struct HookDiagnosticReport {
    pub is_healthy: bool,
    pub primary_issue: Option<HookIssue>,
    pub can_auto_fix: bool,
    pub is_first_run: bool,
    pub binary_ok: bool,
    pub config_ok: bool,
    pub firing_ok: bool,
    // ...
}
```

---

## Part VI: The Runtime Service

The runtime can operate in two modes: embedded (the `CoreRuntime` struct is instantiated directly by the host process) or as a local HTTP service. The `runtime_service/` module provides the client side of this service protocol.

### Service Discovery

The service listens on `127.0.0.1:7474` by default. Clients discover it through a cascade:

1. Environment variables `CAPACITOR_RUNTIME_SERVICE_PORT` and `CAPACITOR_RUNTIME_SERVICE_TOKEN`
2. A connection file at `~/.capacitor/runtime/runtime-service.json`
3. A token file at `~/.capacitor/runtime/runtime-service-7474.token`

All communication is authenticated with a Bearer token. The service exposes four endpoints:

- `GET /health` -- returns service metadata (PID, version, protocol version)
- `GET /runtime/snapshot` -- returns the full `AppSnapshot`
- `POST /runtime/ingest/hook-event` -- ingests a hook event
- `POST /runtime/ingest/shell-signal` -- ingests a shell signal

The client implementation in `RuntimeServiceEndpoint` uses raw TCP sockets (no HTTP library dependency) with a 2-second read timeout:

```rust
fn request_json<Request, Response>(
    &self, method: &str, path: &str, payload: Option<&Request>,
) -> Result<Response, String> {
    let mut stream = TcpStream::connect((self.host.as_str(), self.port))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    // ... hand-crafted HTTP/1.1 request
}
```

This is a deliberate choice to minimize dependencies. The runtime service protocol is simple enough that a full HTTP client would be overkill.

### Snapshot Liveness

The `runtime_state/snapshot.rs` module provides a bridge between the service and the rest of the crate. When the UI needs to check session liveness (is a session still running?), it calls `sessions_snapshot()`, which fetches the current snapshot from the runtime service and converts the session records into `RuntimeSessionRecord` structs.

Importantly, the `is_alive` field from the service is *not* blindly trusted. The snapshot module sets `is_alive: None` and lets the health-checking logic make its own determination based on timestamps and state:

```rust
RuntimeSessionRecord {
    // ... fields copied from snapshot ...
    is_alive: None,  // Not assumed from service
}
```

---

## Part VII: Project and Dashboard Management

The HUD's primary view is a dashboard showing pinned projects, their activity status, and global Claude Code configuration.

### Project Loading

When the dashboard loads, `load_projects_with_storage()` reads the `HudConfig` (which contains the list of pinned project paths), then builds a `Project` struct for each:

```rust
pub fn load_projects_with_storage(storage: &StorageConfig) -> Result<Vec<Project>, String> {
    let config = load_hud_config_with_storage(storage);
    let mut stats_cache = load_stats_cache_with_storage(storage);

    let mut projects: Vec<(Project, SystemTime)> = Vec::new();
    for path in &config.pinned_projects {
        let project = build_project_from_path(path, claude_dir, &mut stats_cache)
            .unwrap_or_else(|| build_missing_project(path));
        // ... sort by latest session activity
    }
    // save updated stats cache
    Ok(projects)
}
```

The function is resilient to missing directories: if a pinned project's path no longer exists, it creates a minimal `Project` with `is_missing: true` so the UI can show a warning rather than crashing.

Projects are sorted by most recent session activity, determined by the modification time of JSONL files in Claude's projects directory. Agent transcript files (those starting with `agent-`) are explicitly excluded from this calculation so that background agent activity doesn't push a project to the top of the list.

### Statistics

Token usage statistics are parsed from Claude's JSONL session files. The `runtime_stats.rs` module uses lazy regex patterns (compiled once, reused everywhere) to extract token counts, model names, and timestamps:

```rust
pub static RE_INPUT_TOKENS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#""input_tokens":(\d+)"#).unwrap());
pub static RE_OUTPUT_TOKENS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#""output_tokens":(\d+)"#).unwrap());
```

These regexes scan the raw JSONL content without fully parsing it -- a pragmatic tradeoff between correctness and performance. Full JSON parsing of potentially gigabyte-scale session files would be prohibitively slow.

The stats system uses an intelligent mtime-based cache. Each file's size and modification time are stored in a `StatsCache`. On the next load, only files that have changed are re-parsed:

```rust
let is_new_or_modified =
    cached_file.map_or(true, |cf| cf.size != size || cf.mtime != mtime);
if is_new_or_modified {
    needs_recompute = true;
}
```

### Project Validation

When a user adds a project to the HUD, the path goes through validation in `runtime_validation.rs`. The validator checks for several conditions:

- **Path doesn't exist**: `PathNotFound`
- **Path is too broad** (`/`, `~`, `/Users`): `DangerousPath`
- **Path is inside a project** (e.g., `~/project/src/`): `SuggestParent`, recommending the project root
- **Path has project markers but no CLAUDE.md**: `MissingClaudeMd`, offering to create one
- **Path has no markers at all**: `NotAProject`
- **Path is already pinned**: `AlreadyTracked`

If the user accepts, the validator can auto-generate a `CLAUDE.md` file by extracting metadata from `package.json` or `Cargo.toml`.

---

## Part VIII: The Routing System

One of Capacitor's most distinctive features is its ability to route focus to the terminal window where a Claude session is running. The routing system, implemented in the latter half of `reduce/mod.rs`, determines which terminal app, tmux session, or tmux pane hosts each project.

### Shell Signals

The routing system relies on shell signals -- periodic reports from a shell integration that runs in the user's shell (bash/zsh). Each signal reports:

```rust
pub struct ShellSignal {
    pub pid: u32,             // Shell process ID
    pub cwd: String,          // Current working directory
    pub tty: String,          // TTY device (e.g., /dev/ttys001)
    pub parent_app: String,   // Terminal app (e.g., "ghostty")
    pub tmux_session: Option<String>,
    pub tmux_client_tty: Option<String>,
    pub tmux_pane: Option<String>,
    pub updated_at: String,
}
```

### Routing Resolution

For each project with active sessions, the routing system selects the best matching shell signal:

1. **PID match** (rank 2): the shell's PID matches a session's PID. This is the strongest signal.
2. **CWD match** (rank 1): the shell's working directory matches the project path. Weaker, but useful when PID information is unavailable.

Among matching shells, the system prefers those with more routing precision:

- Rank 3: tmux pane (most specific)
- Rank 2: tmux session
- Rank 1: terminal app (least specific, only "detached" activation possible)
- Rank 0: unknown app

The routing view for each project includes a `RoutingStatus`:

```rust
pub enum RoutingStatus {
    Attached,    // tmux session has a client TTY connected
    Detached,    // tmux session exists but no client is attached
    Unavailable, // no routing evidence
}
```

And a `RoutingTarget` that specifies how to activate the session:

```rust
pub struct RoutingTarget {
    pub kind: RoutingTargetKind,  // TmuxPane, TmuxSession, TerminalApp, None
    pub terminal_app: Option<String>,
    pub session_name: Option<String>,
    pub pane_id: Option<String>,
    pub host_tty: Option<String>,
}
```

The `ParentApp` enum provides type-safe classification of terminal applications, distinguishing between terminals (Ghostty, iTerm, Alacritty), IDEs (VS Code, Cursor, Zed), and multiplexers (tmux):

```rust
pub enum ParentApp {
    Ghostty, ITerm, Terminal, Alacritty, Kitty, Warp,  // terminals
    Cursor, VSCode, VSCodeInsiders, Zed,                // IDEs
    Tmux,                                               // multiplexer
    Unknown,
}
```

---

## Part IX: Idea Capture

Capacitor includes a lightweight idea-capture system that stores notes in markdown files. Ideas live in `~/.capacitor/projects/{encoded-path}/ideas.md`, where they can be read and edited by both the HUD and by Claude directly.

Each idea has a ULID identifier (26 characters, sortable, generated with the `ulid` crate), metadata fields (effort, status, triage), and a free-form description. The markdown format is designed to be human-editable while still being machine-parseable:

```markdown
### [#idea-01JQXYZ8K6TQFH2M5NWQR9SV7X] Add retry logic
- **Added:** 2026-01-14T15:23:42Z
- **Effort:** small
- **Status:** open
- **Triage:** pending
- **Related:** None

The API client should retry transient failures with exponential backoff.

---
```

The parser is careful about metadata injection: it only parses `- **Key:** value` lines in the contiguous block immediately after the heading. Once a blank line is encountered, subsequent lines are treated as description content. This prevents a description containing `- **Status:** done` from accidentally marking an idea as complete.

All writes use atomic file operations (write to temp file, then rename). The display order is stored separately in `ideas-order.json` to avoid churning the markdown file on every drag-and-drop reorder in the UI.

---

## Part X: Storage Backends

The `storage/` module defines two abstractions: `SnapshotStorage` and `ObservationJournalStore`.

### Snapshot Storage

```rust
pub trait SnapshotStorage: Send + Sync {
    fn load_snapshot(&self) -> Result<Option<AppSnapshot>, String>;
    fn save_snapshot(&self, snapshot: &AppSnapshot) -> Result<(), String>;
}
```

Two implementations exist:

- **`InMemorySnapshotStorage`**: uses a `Mutex<Option<AppSnapshot>>`. Used for testing and for embedded runtimes where persistence isn't needed.
- **`JsonFileSnapshotStorage`**: writes the snapshot to a JSON file on disk using atomic temp-file-then-rename. Used in production to survive process restarts.

The file storage uses its own internal `Mutex<()>` to serialize I/O operations, ensuring that concurrent snapshot writes don't interleave and corrupt the file.

### Observation Journal

```rust
pub trait ObservationJournalStore: Send + Sync {
    fn append(&self, observation: ObservationRecord) -> Result<(), String>;
    fn list(&self) -> Result<Vec<ObservationRecord>, String>;
}
```

Currently only `InMemoryObservationJournalStore` is implemented. This provides the foundation for future event replay and audit logging, but the current system doesn't persist the observation journal -- it persists snapshots instead.

---

## Part XI: Error Handling

The crate uses a two-tier error strategy:

**`HudError`** is the rich internal error type, with variants for every failure mode (config not found, JSON parse error, I/O failure, etc.). It uses `thiserror` for ergonomic error messages and proper `Display`/`Error` implementations.

**`HudFfiError`** is the FFI-safe error type, with a single `General { message: String }` variant. Every `HudError` can be converted into an `HudFfiError` by calling `.to_string()`. This flattening is intentional -- FFI boundaries cannot carry Rust's rich error hierarchy, so errors are serialized as human-readable strings before crossing the boundary.

At the `CoreRuntime` level, a third error type `CoreRuntimeError` serves the same purpose for UniFFI-exported methods. The pattern is consistent: rich errors internally, flat strings at the FFI boundary.

---

## Part XII: Configuration

The `runtime_config.rs` module handles loading and saving two kinds of configuration:

1. **HudConfig**: the list of pinned projects and the preferred terminal app. Stored at `~/.capacitor/projects.json`.
2. **StatsCache**: cached token usage statistics for each project. Stored at `~/.capacitor/stats-cache.json`.

Both use the same defensive pattern:

```rust
pub fn load_hud_config_with_storage(storage: &StorageConfig) -> HudConfig {
    let path = storage.projects_file();
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}
```

If the file doesn't exist, use defaults. If the file is corrupted, use defaults. The app should always launch, even if its configuration is damaged. This is a deliberate UX decision: a partially broken HUD is better than a HUD that won't start.

Writes use the atomic temp-plus-rename pattern:

```rust
let tmp_path = path.with_extension("tmp");
let mut file = fs::File::create(&tmp_path)?;
file.write_all(content.as_bytes())?;
file.sync_all()?;
fs::rename(&tmp_path, &path)?;
```

The `sync_all()` call ensures the data is flushed to disk before the rename. Without it, a power failure between the write and the rename could leave the temp file empty. With it, the worst case is that the rename doesn't happen and the old file is preserved.

---

## Epilogue: The Architecture in Perspective

Capacitor's runtime core follows a pattern that might be called **"event sourcing lite."** It has the shape of an event-sourced system -- ingest, observe, reduce, project -- but without the full operational complexity of event replay or CQRS. The observation journal exists structurally but isn't persisted. The snapshot *is* the durable state, not a derived view.

This is a pragmatic choice for a desktop application. The benefits of event sourcing (deterministic state reconstruction, audit logging, temporal queries) are less valuable when your state is a few hundred sessions at most and your persistence model is a single JSON file. But the architectural seams are in place: if the system needed to replay events or maintain multiple read models in the future, the modules are already separated along those boundaries.

The crate's most distinctive quality is its commitment to the FFI boundary as a first-class design constraint. Every type is serializable. Every function that needs file paths takes them as parameters rather than computing them internally. Every error is convertible to a flat string. This discipline makes the crate usable from Swift, Kotlin, and Python without any wrapper code -- the UniFFI bindings are generated directly from the Rust source.

The result is a system where the macOS desktop app, a potential TUI, and the local runtime service all share the same business logic, the same state machine, and the same persistence format. There is one source of truth, and it is written in Rust.
