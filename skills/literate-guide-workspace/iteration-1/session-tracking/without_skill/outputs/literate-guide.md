# The Story of Session Tracking in Capacitor

## A Literate Guide to How Capacitor Tracks and Restores Claude Code Sessions

*In the style of Knuth's Literate Programming: code and prose interwoven, ordered for human understanding rather than compiler convenience.*

---

## Prologue: The Problem

Claude Code runs in a terminal. It has no GUI of its own. When a developer is deep in a coding session -- Claude working, waiting for permission, or sitting idle -- there is no ambient visual indicator of what is happening. The developer must switch to the terminal to check.

Capacitor exists to solve this. It is a macOS companion app that shows the live state of every Claude Code session across every project the developer works on. But to do that, it must answer a deceptively hard question: *what is Claude doing right now, in every project, at all times?*

The answer spans three layers of the system, each with its own language and idiom:

1. **The Hook System** (`core/hud-hook/`) -- a Rust binary that Claude Code invokes at lifecycle moments, serving as the event ingress
2. **The Runtime Core** (`core/capacitor-core/`) -- a Rust library that reduces events into authoritative state, persists snapshots, and serves them over HTTP
3. **The Swift UI** (`apps/swift/`) -- a macOS app that polls the runtime service and projects state into visual cards

This guide traces the complete lifecycle of a session event from the moment Claude Code fires it to the moment it becomes a glowing card on screen.

---

## Act I: The Source of Truth -- Claude Code's Hook Events

Every Claude Code session emits structured lifecycle events through its hook system. These events are the atomic facts from which all session state is derived.

The complete vocabulary of hook events is defined in `core/hud-hook/src/hook_types.rs`:

```rust
// core/hud-hook/src/hook_types.rs

pub enum HookEvent {
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PreToolUse  { tool_name: Option<String>, file_path: Option<String> },
    PostToolUse { tool_name: Option<String>, file_path: Option<String> },
    PostToolUseFailure { tool_name: Option<String>, file_path: Option<String> },
    PermissionRequest,
    PreCompact,
    Notification { notification_type: String },
    SubagentStart,
    SubagentStop,
    Stop { stop_hook_active: bool },
    TeammateIdle,
    TaskCompleted,
    WorktreeCreate,
    WorktreeRemove,
    ConfigChange,
    Unknown { event_name: String },
}
```

These events arrive as JSON from Claude Code. The `HookInput` struct deserializes them:

```rust
// core/hud-hook/src/hook_types.rs

pub struct HookInput {
    pub hook_event_name: Option<String>,
    pub session_id:      Option<String>,
    pub cwd:             Option<String>,
    pub notification_type: Option<String>,
    pub stop_hook_active:  Option<bool>,
    pub tool_name:         Option<String>,
    pub tool_input:        Option<ToolInput>,
    pub tool_response:     Option<ToolResponse>,
    pub agent_id:          Option<String>,
    pub teammate_name:     Option<String>,
}
```

The `to_event()` method on `HookInput` performs the string-to-enum mapping, translating `"SessionStart"` into `HookEvent::SessionStart`, `"PreToolUse"` into `HookEvent::PreToolUse { ... }`, and so on. Unknown event names produce `HookEvent::Unknown` rather than an error -- the system is designed to be forward-compatible with future Claude Code versions.

---

## Act II: The Runtime Service -- A Long-Lived Local Server

### 2.1 How the Server Starts

The Swift app is responsible for launching and managing the runtime service process. This is handled by `HookServerManager` in `apps/swift/Sources/Capacitor/Models/HookServerManager.swift`.

At app startup, the manager calls `startIfNeeded()`. The sequence is:

1. **Check for a stale PID file** from a previous app session at `~/.capacitor/runtime/runtime-service-7474.pid`
2. **If the old process is still alive** and its executable path matches the expected `~/.local/bin/hud-hook` binary, **adopt it** -- reuse the existing server rather than launching a fresh one
3. **Otherwise**, launch a new `hud-hook serve --port 7474` process

```swift
// apps/swift/Sources/Capacitor/Models/HookServerManager.swift

func startIfNeeded() {
    guard status != .running, status != .starting else { return }

    if let stalePid = dependencies.readPidFile(pidFilePath) {
        if dependencies.isProcessAlive(stalePid),
           dependencies.isManagedServerProcess(stalePid, binaryPath)
        {
            // Reuse the running server
            beginLifecycleObservation(adoptedPid: stalePid, ...)
            return
        }
        // Clean up stale PID file
        dependencies.removePidFile(pidFilePath)
    }

    start()
}
```

When launching fresh, the manager generates a random bearer token and passes it via environment variables:

```swift
// apps/swift/Sources/Capacitor/Models/HookServerManager.swift

private func start() {
    var environment = ProcessInfo.processInfo.environment
    environment["CAPACITOR_CORE_ENABLED"] = "1"
    let authToken = UUID().uuidString
    environment["CAPACITOR_RUNTIME_SERVICE_BOOTSTRAP"] = "1"
    environment["CAPACITOR_RUNTIME_SERVICE_PORT"] = String(port)
    environment["CAPACITOR_RUNTIME_SERVICE_TOKEN"] = authToken
    healthAuthorizationToken = authToken

    let launchedProcess = try dependencies.launchProcess(
        binaryPath, port, environment
    )
}
```

The auth token is critical: it prevents unauthorized processes from reading session data or injecting fake events. Every request to the runtime service must carry `Authorization: Bearer <token>`.

### 2.2 The Server's HTTP Surface

The `hud-hook serve` command starts a `tiny_http` server. Its dispatch table in `core/hud-hook/src/serve.rs` reveals the complete API:

```rust
// core/hud-hook/src/serve.rs

fn dispatch(request: tiny_http::Request, runtime_service: &RuntimeServerState) {
    match (request.method(), request.url()) {
        (&Get,  "/health")                       => handle_health(...),
        (&Get,  "/runtime/snapshot")              => handle_runtime_snapshot(...),
        (&Post, "/runtime/ingest/hook-event")     => handle_runtime_ingest_hook_event(...),
        (&Post, "/runtime/ingest/shell-signal")   => handle_runtime_ingest_shell_signal(...),
        (&Post, "/hook")                          => handle_hook(...),
        _ => { /* 404 */ }
    }
}
```

Four endpoints, each with a distinct role:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Liveness probe for the Swift app's health checker |
| `/runtime/snapshot` | GET | Returns the full `AppSnapshot` -- all projects, sessions, shells, routing |
| `/runtime/ingest/hook-event` | POST | Receives a normalized hook event command |
| `/runtime/ingest/shell-signal` | POST | Receives shell CWD tracking signals |
| `/hook` | POST | Legacy path: receives raw `HookInput` JSON and processes it through the handler |

### 2.3 Server Startup and State Recovery

When the server starts, it initializes the `CoreRuntime` with a snapshot file path. This is the key to **session restoration across restarts**:

```rust
// core/hud-hook/src/serve.rs

impl RuntimeServerState {
    fn new(port: u16) -> Result<Self, String> {
        let bootstrap = RuntimeServiceBootstrap::from_env(port)?;
        let artifact_path = runtime_artifact_path()?;
        let runtime = CoreRuntime::new_with_snapshot_file(
            artifact_path.to_string_lossy().to_string()
        )?;
        // ...
    }
}
```

The default artifact path is `~/.capacitor/runtime/app_snapshot.json`. The `CoreRuntime` constructor loads this file and reconstitutes the reducer state from it:

```rust
// core/capacitor-core/src/lib.rs

impl CoreRuntime {
    fn from_storage(
        snapshot_storage: Arc<dyn SnapshotStorage>,
        app_storage: StorageConfig,
    ) -> Result<Arc<Self>, CoreRuntimeError> {
        let state = snapshot_storage
            .load_snapshot()?
            .map(reduce::ReducerState::from_snapshot)
            .unwrap_or_default();

        Ok(Arc::new(Self {
            state: std::sync::Mutex::new(state),
            snapshot_storage,
            app_storage,
        }))
    }
}
```

If the snapshot file exists, all previous session records, project summaries, and shell signals are restored into the `ReducerState`. If it doesn't exist (first run), the state starts empty. This means: **sessions survive server restarts, app restarts, and even machine reboots** -- as long as the snapshot file persists on disk.

### 2.4 Snapshot Persistence

The `JsonFileSnapshotStorage` uses an atomic write pattern -- write to a `.tmp` file, then rename -- to prevent corruption if the process crashes mid-write:

```rust
// core/capacitor-core/src/storage/mod.rs

impl SnapshotStorage for JsonFileSnapshotStorage {
    fn save_snapshot(&self, snapshot: &AppSnapshot) -> Result<(), String> {
        let payload = serde_json::to_vec_pretty(snapshot)?;
        let temp_path = self.path.with_file_name(format!("{file_name}.tmp"));

        fs::write(&temp_path, payload)?;
        fs::rename(&temp_path, &self.path)?;
        Ok(())
    }
}
```

Every time an event is ingested, the full snapshot is persisted. This is the durability guarantee.

---

## Act III: Event Ingestion -- From Raw JSON to Reduced State

### 3.1 The Handler Pipeline

When a hook event arrives at `/runtime/ingest/hook-event`, the server passes it directly to the `CoreRuntime`. But when it arrives at the legacy `/hook` endpoint, it flows through the handler in `core/hud-hook/src/handle.rs`:

```rust
// core/hud-hook/src/handle.rs

pub(crate) fn handle_hook_input(hook_input: HookInput) -> Result<(), String> {
    let event = hook_input.to_event()?;         // Parse the event type
    let session_id = hook_input.session_id?;     // Require a session ID
    let cwd = hook_input.resolve_cwd(None);      // Resolve working directory

    // Guard: skip subagent Stop events
    if matches!(event, HookEvent::Stop { .. }) && hook_input.agent_id.is_some() {
        return Ok(());
    }

    // Guard: skip events without a CWD (except Delete/SessionEnd)
    if cwd.is_none() && action != Action::Delete {
        return Ok(());
    }

    // Send to the core runtime
    runtime_client::send_handle_event(&event, &hook_input, &session_id, None, &cwd)
}
```

The handler applies several guards before forwarding:

- **Unknown events** are silently skipped (forward-compatibility)
- **Missing session_id** events are skipped (can't track what we can't identify)
- **Subagent Stop events** are skipped (they share the parent's session_id but shouldn't affect its state)
- **Missing CWD** events are skipped (except SessionEnd, which is a delete operation)

### 3.2 The Runtime Client -- Bridging Hook to Core

The runtime client in `core/hud-hook/src/runtime_client.rs` translates hook events into the core domain's `IngestHookEventCommand`:

```rust
// core/hud-hook/src/runtime_client.rs

pub fn send_handle_event(
    event: &HookEvent,
    hook_input: &HookInput,
    session_id: &str,
    pid: Option<u32>,
    cwd: &str,
) -> bool {
    let event_type = event_type_for_hook(event)?;

    let command = IngestHookEventCommand {
        event_id:          make_event_id(pid.unwrap_or(0)),
        recorded_at:       Utc::now().to_rfc3339(),
        event_type,
        session_id:        session_id.to_string(),
        pid,
        project_path:      cwd.to_string(),
        cwd:               Some(cwd.to_string()),
        file_path:         event_file_path(event, hook_input),
        workspace_id:      None,
        notification_type: event_notification_type(event),
        stop_hook_active:  event_stop_hook_active(event),
        tool_name:         event_tool_name(event, hook_input),
        agent_id:          normalize_optional(&hook_input.agent_id),
        teammate_name:     normalize_optional(&hook_input.teammate_name),
    };

    send_event(command).is_ok()
}
```

A key design decision here: the runtime client has **two transport modes**. When running inside the `hud-hook serve` process, it uses a direct in-process `Arc<CoreRuntime>` reference. When running as a standalone CLI invocation, it discovers the runtime service via its token file and sends HTTP:

```rust
// core/hud-hook/src/runtime_client.rs

enum RuntimeTransport {
    Service(RuntimeServiceEndpoint),          // HTTP to the server
    RegisteredService(Arc<CoreRuntime>),       // Direct in-process call
}

fn runtime_transport() -> Result<RuntimeTransport, String> {
    // Prefer the in-process runtime (registered by serve.rs at startup)
    if let Some(runtime) = REGISTERED_SERVICE_RUNTIME.get() {
        return Ok(RuntimeTransport::RegisteredService(Arc::clone(runtime)));
    }
    // Fall back to HTTP discovery
    runtime_service_endpoint()?
        .map(RuntimeTransport::Service)
        .ok_or_else(|| "runtime service endpoint unavailable".to_string())
}
```

### 3.3 Normalization

Before events reach the reducer, the `ingest` module normalizes all string fields:

```rust
// core/capacitor-core/src/ingest/mod.rs

pub fn normalize_hook_event(command: IngestHookEventCommand) -> IngestHookEventCommand {
    IngestHookEventCommand {
        event_id:     command.event_id.trim().to_string(),
        session_id:   command.session_id.trim().to_string(),
        project_path: normalize_required_path(&command.project_path),
        cwd:          normalize_optional_path(command.cwd),
        // ... every field is trimmed, paths are normalized
    }
}
```

Path normalization strips trailing slashes and, on macOS, lowercases for case-insensitive comparison. This prevents `/Users/Pete/Code/` and `/users/pete/code` from being treated as different projects.

### 3.4 The Reducer -- Heart of the State Machine

The reducer in `core/capacitor-core/src/reduce/mod.rs` is the authoritative state machine. It maintains:

```rust
// core/capacitor-core/src/reduce/mod.rs

pub struct ReducerState {
    pub projects: BTreeMap<String, ProjectSummary>,
    pub sessions: HashMap<String, SessionSummary>,
    pub shells:   HashMap<u32, ShellSignal>,
    pub routing:  BTreeMap<String, RoutingView>,
    // diagnostics counters...
}
```

When an event arrives, `apply_hook_event` follows this sequence:

1. **Increment** the events_ingested counter
2. **Validate** the event_id and session_id
3. **Check staleness** -- if the event's timestamp is older than the session's current state, skip it
4. **Reduce** the session -- apply the state transition
5. **Recompute** the project summaries (aggregate across sessions)
6. **Recompute** routing (match sessions to terminal shells)

The `reduce_session` function implements the state machine. The states and their transitions form a directed graph:

```
                      SessionStart
                          |
                          v
                       [Ready] <-------- Stop (stop_hook_active=false)
                       /    \            TaskCompleted (main agent)
                      /      \           Notification(idle_prompt)
     UserPromptSubmit/        \          Notification(auth_success)
     PreToolUse     /          \
     PostToolUse   v            v
              [Working]    [Waiting] <-- PermissionRequest
                  |              ^       Notification(permission_prompt)
                  |             /        Notification(elicitation_dialog)
                  v            /
             [Compacting] <-- PreCompact
                  |
                  v
            (back to Ready on next Stop/TaskCompleted)


     SessionEnd --> [Delete session record]
```

Here is the actual reducer code that implements these transitions:

```rust
// core/capacitor-core/src/reduce/mod.rs

fn reduce_session(
    current: Option<&SessionSummary>,
    event: &IngestHookEventCommand,
) -> SessionUpdate {
    match event.event_type {
        HookEventType::SessionStart => {
            // Guard: don't reset an already-active session
            let already_working = current
                .map(|r| r.state == Working || r.state == Waiting)
                .unwrap_or(false);
            if already_working {
                SessionUpdate::Skip("session_start_already_active")
            } else {
                SessionUpdate::Upsert(upsert_session(current, event, Ready, None))
            }
        }

        HookEventType::UserPromptSubmit | HookEventType::PreToolUse =>
            SessionUpdate::Upsert(upsert_session(current, event, Working, None)),

        HookEventType::PostToolUse | HookEventType::PostToolUseFailure =>
            SessionUpdate::Upsert(upsert_session(current, event, Working, None)),

        HookEventType::PermissionRequest =>
            SessionUpdate::Upsert(upsert_session(current, event, Waiting, None)),

        HookEventType::PreCompact =>
            SessionUpdate::Upsert(upsert_session(current, event, Compacting, None)),

        HookEventType::SessionEnd => {
            // If the PID is still alive, don't delete -- downgrade to Ready
            let pid = event.pid.or_else(|| current.map(|r| r.pid)).unwrap_or(0);
            if pid > 0 && is_pid_alive(pid) {
                SessionUpdate::Upsert(upsert_session(current, event, Ready, ...))
            } else {
                SessionUpdate::Delete(event.session_id.clone())
            }
        }

        // Informational events don't change state
        HookEventType::SubagentStart | HookEventType::TeammateIdle | ... =>
            SessionUpdate::Skip("informational_event"),
    }
}
```

A notable subtlety: `SessionEnd` does not always delete the session. If the process is still alive (checked via `kill(pid, 0)`), the session is downgraded to Ready instead. This handles the case where Claude Code sends a SessionEnd during a conversation reset but the process continues running.

### 3.5 The Session Domain Types

Each session is tracked as a `SessionSummary`:

```rust
// core/capacitor-core/src/domain/types.rs

pub struct SessionSummary {
    pub session_id:       String,
    pub pid:              u32,
    pub cwd:              String,
    pub project_id:       String,
    pub project_path:     String,
    pub workspace_id:     String,
    pub state:            SessionState,
    pub state_changed_at: String,
    pub updated_at:       String,
    pub last_event:       Option<String>,
    pub last_activity_at: Option<String>,
    pub tools_in_flight:  u32,
    pub ready_reason:     Option<String>,
}
```

Sessions are aggregated into projects. Each `ProjectSummary` represents the combined state of all sessions within a project:

```rust
// core/capacitor-core/src/domain/types.rs

pub struct ProjectSummary {
    pub project_path:              String,
    pub project_id:                String,
    pub workspace_id:              String,
    pub display_name:              String,
    pub state:                     SessionState,    // highest-priority session's state
    pub representative_session_id: Option<String>,  // the "winning" session
    pub latest_session_id:         Option<String>,
    pub session_count:             u64,
    pub active_count:              u64,
    pub has_session:               bool,
}
```

The five states themselves carry priority rankings:

```rust
// core/capacitor-core/src/domain/types.rs

pub enum SessionState {
    Working,     // priority 3 -- Claude is actively generating/using tools
    Ready,       // priority 1 -- Claude is waiting for user input
    Idle,        // priority 0 -- no active session
    Compacting,  // priority 2 -- Claude is compacting context
    Waiting,     // priority 4 -- Claude needs permission or user decision
}
```

When a project has multiple sessions, the one with the highest-priority state becomes the representative. Waiting (4) beats Working (3) beats Compacting (2) beats Ready (1) beats Idle (0).

### 3.6 The AppSnapshot -- The Complete Picture

All of this data is bundled into a single `AppSnapshot`:

```rust
// core/capacitor-core/src/domain/types.rs

pub struct AppSnapshot {
    pub projects:     Vec<ProjectSummary>,
    pub sessions:     Vec<SessionSummary>,
    pub shells:       Vec<ShellSignal>,
    pub routing:      Vec<RoutingView>,
    pub diagnostics:  DiagnosticsSummary,
    pub generated_at: String,
}
```

This snapshot is both the HTTP response format (for the `/runtime/snapshot` endpoint) and the persistence format (for `app_snapshot.json`). One schema, two purposes. This means restoring from disk and reading over HTTP produce identical data structures.

---

## Act IV: The Swift UI -- Polling, Projection, and Stabilization

### 4.1 The Polling Loop

The Swift app polls the runtime service every 2 seconds via a repeating timer:

```swift
// apps/swift/Sources/Capacitor/Models/AppState.swift

private func setupRefreshTimer() {
    refreshTimer = Timer.scheduledTimer(withTimeInterval: 2.0, repeats: true) { [weak self] in
        DispatchQueue.main.async {
            self?.refreshSessionStates()
        }
    }
}
```

Each tick of `refreshSessionStates()` increments a generation counter, cancels any in-flight request, and fires a new one:

```swift
// apps/swift/Sources/Capacitor/Models/AppState.swift

func refreshSessionStates() {
    runtimeSnapshotGeneration &+= 1
    let refreshGeneration = runtimeSnapshotGeneration
    let correlationId = nextRuntimeSnapshotCorrelationId()
    let currentProjects = projects

    runtimeSnapshotTask?.cancel()
    runtimeSnapshotTask = Task { [weak self] in
        let snapshot = try await RuntimeClient.shared.fetchRuntimeSnapshot(
            correlationId: correlationId
        )
        await self?.applyRuntimeSnapshotIfFresh(
            snapshot,
            refreshGeneration: refreshGeneration,
            correlationId: correlationId,
            projects: currentProjects,
        )
    }
}
```

The generation counter is essential: if a response arrives from a stale request (one that was superseded by a newer tick), it is silently dropped. This prevents out-of-order updates from corrupting the UI.

### 4.2 The RuntimeClient -- HTTP to the Runtime Service

The `RuntimeClient` discovers the runtime service's address by reading `~/.capacitor/runtime/runtime-service.json`:

```swift
// apps/swift/Sources/Capacitor/Models/RuntimeClient.swift

static func current() -> RuntimeServiceConnection? {
    // First, check environment variables
    if let port = processInfo.environment["CAPACITOR_RUNTIME_SERVICE_PORT"],
       let token = processInfo.environment["CAPACITOR_RUNTIME_SERVICE_TOKEN"] {
        return RuntimeServiceConnection(
            baseURL: URL(string: "http://127.0.0.1:\(port)")!,
            bearerToken: token,
        )
    }

    // Fall back to connection file on disk
    let connectionURL = homeDir.appendingPathComponent(
        ".capacitor/runtime/runtime-service.json"
    )
    let record = JSONDecoder().decode(ConnectionRecord.self, from: data)
    return RuntimeServiceConnection(
        baseURL: URL(string: "http://127.0.0.1:\(record.port)")!,
        bearerToken: record.authToken,
    )
}
```

The connection file is written by the Rust server at startup (`RuntimeServiceBootstrap::write_token_file`) and cleaned up when the server exits (via a `Drop` guard on `RuntimeServiceTokenGuard`). The Swift app reads it to discover how to authenticate.

### 4.3 The SessionStateManager -- Projection and Stabilization

The `SessionStateManager` in `apps/swift/Sources/Capacitor/Models/SessionStateManager.swift` is where the runtime service's raw data is transformed into view-ready state. It owns several crucial responsibilities:

**Project Matching.** Runtime sessions report a `project_path`, but the user's pinned projects may have different paths (symlinks, worktrees, monorepo subdirectories). The manager resolves matches through multiple strategies:

```swift
// apps/swift/Sources/Capacitor/Models/SessionStateManager.swift

private nonisolated func matchesProject(
    _ project: ProjectMatchInfo,
    state: StateMatchInfo,
    homeNormalized: String,
) -> Bool {
    // Strategy 1: workspace ID match (handles worktrees)
    if project.workspaceId == state.workspaceId { return true }

    // Strategy 2: path containment (handles monorepo subdirectories)
    if isParentOrSelfExcludingHome(
        parent: project.normalizedPath,
        child: state.normalizedPath,
        homeNormalized: homeNormalized,
    ) { return true }

    // Strategy 3: git common dir match (handles multiple worktrees)
    if projectCommon == stateCommon { /* ... */ return true }

    return false
}
```

**Stale-Working Normalization.** Claude Code doesn't always fire a `Stop` event when the user interrupts a response (e.g., Ctrl-C). The manager detects this and downgrades stale "working" sessions to "ready":

```swift
// apps/swift/Sources/Capacitor/Utilities/SessionStaleness.swift

enum SessionStaleness {
    /// 30s is well beyond the longest normal gap between tool-use events
    /// (~5-15s) but short enough to feel responsive after an interrupt.
    static let workingStaleThreshold: TimeInterval = 30

    static func isWorkingStale(
        state: SessionState?,
        updatedAt: String?,
        now: Date,
    ) -> Bool {
        guard state == .working, let date = parseISO8601Date(updatedAt) else {
            return false
        }
        return now.timeIntervalSince(date) > workingStaleThreshold
    }
}
```

**Empty Snapshot Hysteresis.** If the runtime service returns an empty snapshot (no sessions), the manager doesn't immediately clear the UI. It requires two consecutive empty snapshots before committing, preventing flicker during transient network issues:

```swift
// apps/swift/Sources/Capacitor/Models/SessionStateManager.swift

private func stabilizeEmptyRuntimeSnapshotIfNeeded(
    _ merged: [String: ProjectSessionState],
) -> [String: ProjectSessionState] {
    if merged.isEmpty {
        guard !sessionStates.isEmpty else { return merged }

        consecutiveEmptySnapshotCount += 1
        if consecutiveEmptySnapshotCount < Constants.emptySnapshotCommitThreshold {
            return sessionStates  // Hold the previous state
        }

        consecutiveEmptySnapshotCount = 0
        return merged  // Commit the empty state
    }
    // ...
}
```

**Idle Transition Hysteresis.** Similarly, transitions from active states to idle are delayed by two consecutive idle snapshots (4 seconds at the 2-second polling interval). Transitions *from* idle *to* active are instant. This asymmetric hysteresis prevents brief idle flickers during the gap between `SessionEnd` and `SessionStart` events:

```swift
// apps/swift/Sources/Capacitor/Models/SessionStateManager.swift

/// At 2s polling, threshold of 2 means a 4s hold before showing idle.
static let idleCommitThreshold = 2

private func stabilizeIdleTransitions(
    _ incoming: [String: ProjectSessionState],
) -> [String: ProjectSessionState] {
    for (path, incomingState) in incoming {
        let isIncomingIdle = incomingState.state == .idle
        let wasActive = sessionStates[path].map { $0.state != .idle } ?? false

        if isIncomingIdle, wasActive {
            let count = (consecutiveIdleCounts[path] ?? 0) + 1
            if count < Constants.idleCommitThreshold {
                result[path] = sessionStates[path]!  // Hold active state
            }
        }
    }
}
```

**Flash Animation Triggers.** When a session transitions between states (e.g., working to ready), the manager triggers a brief visual flash on the project card:

```swift
// apps/swift/Sources/Capacitor/Models/SessionStateManager.swift

private func triggerFlashIfNeeded(for path: String, state: SessionState) {
    switch state {
    case .ready, .waiting, .compacting:
        flashingProjects[path] = state
        DispatchQueue.main.asyncAfter(
            deadline: .now() + Constants.flashDurationSeconds
        ) { [weak self] in
            self?.flashingProjects.removeValue(forKey: path)
        }
    case .working, .idle:
        break  // No flash for these transitions
    }
}
```

### 4.4 Failure Handling

The polling system is designed to degrade gracefully:

```swift
// apps/swift/Sources/Capacitor/Models/AppState.swift

private func handleRuntimeSnapshotFailureIfFresh(...) {
    consecutiveRuntimeSnapshotFailures += 1

    if consecutiveRuntimeSnapshotFailures >= 2 {
        // After 2 consecutive failures (4 seconds), clear all state
        sessionStateManager.clearRuntimeProjectStates()
        shellStateStore.clearRuntimeShellState(...)
        routingStateStore.clearRuntimeRoutingViews(...)
    }
}
```

One failure is tolerated silently. Two consecutive failures (4 seconds of downtime) clears the UI. When the server recovers, the next successful poll restores everything.

---

## Act V: Shell CWD Tracking -- The Second Signal Source

In addition to hook events, Capacitor tracks which terminal shell is in which directory. This enables the **routing** system: knowing which terminal window or tmux pane a Claude Code session is running in.

Shell CWD tracking is handled by `hud-hook cwd`, invoked by shell precmd hooks:

```bash
# Called automatically by the user's shell after every command
hud-hook cwd /path/to/current/directory $$ $(tty)
```

This runs in the background (< 15ms target) and sends a `ShellSignal` to the runtime service:

```rust
// core/hud-hook/src/cwd.rs

pub fn run(path: &str, pid: u32, tty: &str) -> Result<(), CwdError> {
    let parent_app = detect_parent_app(pid);    // Ghostty, iTerm, VS Code, etc.
    let tmux_pane = detect_tmux_pane();          // TMUX_PANE env var

    runtime_client::send_shell_cwd_event(
        pid, &normalized_path, &resolved_tty,
        parent_app, tmux_session, tmux_client_tty,
        proc_start, tmux_pane,
    )
}
```

The shell signals are stored in the `ReducerState` and used during routing recomputation to determine which terminal a session's project lives in.

---

## Act VI: Project Identity -- Solving the Matching Problem

One of the most subtle challenges is: *when a Claude Code session reports it's working in `/Users/pete/Code/myapp`, how does Capacitor know which pinned project card to update?*

The answer is the project identity system in `core/capacitor-core/src/domain/identity.rs`. It resolves a file path to a stable project identity by walking up the directory tree looking for project markers:

```rust
// core/capacitor-core/src/domain/identity.rs

const PROJECT_MARKERS: &[(&str, u8)] = &[
    ("CLAUDE.md",       1),  // Highest priority
    ("package.json",    2),
    ("Cargo.toml",      2),
    ("pyproject.toml",  2),
    ("go.mod",          2),
    (".git",            3),
    ("Makefile",        4),  // Lowest priority
];
```

The workspace ID is an MD5 hash of `project_id|relative_path`, which makes it stable across worktrees. Two worktrees of the same Git repository will have the same `project_id` (pointing to the shared `.git/commondir`) and thus the same workspace ID.

---

## Act VII: The Complete Journey of an Event

Let us trace a single `UserPromptSubmit` event from keyboard to pixel:

1. **The user types a prompt** in Claude Code and presses Enter

2. **Claude Code fires** its `UserPromptSubmit` hook, sending JSON to the configured hook endpoint

3. **The HTTP server** (`hud-hook serve`) receives the POST at `/runtime/ingest/hook-event`

4. **The ingest module** normalizes paths and trims whitespace

5. **The reducer** receives the `IngestHookEventCommand`, looks up the session by `session_id`, and applies the state transition: `Ready -> Working`

6. **The reducer** updates the session's `updated_at`, `state_changed_at`, and `last_event` fields

7. **Project recomputation** promotes the project's aggregate state to `Working` (since it now has a working session)

8. **The snapshot is persisted** to `~/.capacitor/runtime/app_snapshot.json` via atomic write

9. **Within 2 seconds**, the Swift app's polling timer fires

10. **The RuntimeClient** sends `GET /runtime/snapshot` with the bearer token

11. **The server returns** the full `AppSnapshot` as JSON

12. **The RuntimeClient** decodes it into `RuntimeProjectState` structs

13. **The SessionStateManager** matches the runtime project to the user's pinned project card

14. **Stale-working check** passes (the event is fresh)

15. **Hysteresis checks** pass (not empty, not an idle transition)

16. **The session state** is committed to `sessionStates` dictionary

17. **SwiftUI observes** the state change and re-renders the project card with a "working" indicator

18. **If transitioning** from a different state, a flash animation is triggered

Total latency: typically under 2.5 seconds from keystroke to visual update (dominated by the 2-second polling interval).

---

## Act VIII: Session Restoration -- Surviving Restarts

The system's durability story has two complementary mechanisms:

### 8.1 Snapshot Persistence (Rust side)

Every event ingestion persists the full snapshot to disk:

```
CoreRuntime::ingest_hook_event()
  -> state.apply_hook_event(normalized)
  -> let snapshot = state.snapshot()
  -> self.persist_snapshot(&snapshot)  // atomic write to disk
```

On server restart, `CoreRuntime::new_with_snapshot_file()` loads this file and reconstitutes the `ReducerState` via `ReducerState::from_snapshot()`.

### 8.2 Process Adoption (Swift side)

The `HookServerManager` checks for a PID file on startup. If the previous server process is still alive and is the correct binary:

```swift
if dependencies.isProcessAlive(stalePid),
   dependencies.isManagedServerProcess(stalePid, binaryPath)
{
    // Adopt the existing server -- no restart needed
    beginLifecycleObservation(adoptedPid: stalePid, ...)
}
```

This means: if the Swift app crashes and restarts, it can reconnect to the still-running server without losing any session state. The auth token is read from the persisted connection file at `~/.capacitor/runtime/runtime-service.json`.

### 8.3 The Full Recovery Matrix

| Scenario | Data preserved? | How? |
|----------|----------------|------|
| Swift app restart, server still running | Yes | PID adoption + snapshot in memory |
| Server restart, Swift app still running | Yes | Snapshot loaded from disk |
| Both restart | Yes | Snapshot loaded from disk, new server launched |
| Machine reboot | Yes | Snapshot loaded from disk on next launch |
| Snapshot file deleted | No | Fresh start with empty state |

---

## Epilogue: Design Principles

Several principles emerge from studying this system:

**Events in, state out.** The hook events are the only input. The snapshot is the only output. There is one reducer, one state machine, one source of truth. The Swift UI never mutates session state -- it only projects it.

**Graceful degradation everywhere.** Unknown events are skipped. Missing fields are tolerated. Stale responses are dropped. Failed polls are retried. The system always prefers showing slightly stale data over showing nothing.

**Asymmetric hysteresis.** Transitions to less-interesting states (active to idle, data to empty) are delayed. Transitions to more-interesting states (idle to active) are instant. The user sees action immediately but is shielded from noise.

**Atomic persistence.** The snapshot is written atomically (write temp, rename). The reducer is behind a Mutex. There is no partial state on disk.

**Forward compatibility.** Unknown hook events produce `HookEvent::Unknown` and are silently skipped. The system will continue to work correctly with future Claude Code versions that emit new event types -- it will simply ignore what it doesn't understand.

---

## Appendix: File Reference

| Layer | File | Role |
|-------|------|------|
| Hook | `core/hud-hook/src/main.rs` | CLI entry point: `serve` and `cwd` subcommands |
| Hook | `core/hud-hook/src/serve.rs` | HTTP server: dispatches to handler and runtime |
| Hook | `core/hud-hook/src/handle.rs` | Event handler: guards, classification, forwarding |
| Hook | `core/hud-hook/src/hook_types.rs` | `HookInput` deserialization, `HookEvent` enum |
| Hook | `core/hud-hook/src/runtime_client.rs` | Translates hook events to core commands |
| Hook | `core/hud-hook/src/cwd.rs` | Shell CWD tracking via precmd hooks |
| Core | `core/capacitor-core/src/lib.rs` | `CoreRuntime`: the central coordinator |
| Core | `core/capacitor-core/src/domain/types.rs` | All domain types: `SessionState`, `AppSnapshot`, etc. |
| Core | `core/capacitor-core/src/domain/identity.rs` | Project boundary detection and workspace identity |
| Core | `core/capacitor-core/src/ingest/mod.rs` | Input normalization before reduction |
| Core | `core/capacitor-core/src/reduce/mod.rs` | `ReducerState`: the authoritative state machine |
| Core | `core/capacitor-core/src/storage/mod.rs` | Snapshot persistence (JSON file and in-memory) |
| Core | `core/capacitor-core/src/runtime_service/mod.rs` | Runtime service endpoint discovery and HTTP client |
| Core | `core/capacitor-core/src/runtime_sessions.rs` | Project status file reading (`.claude/hud-status.json`) |
| Core | `core/capacitor-core/src/runtime_state/snapshot.rs` | Snapshot fetching from runtime service for read-only consumers |
| Core | `core/capacitor-core/src/runtime_contracts/claude_hooks.rs` | Hook event contract definitions (transport modes) |
| Swift | `apps/swift/Sources/Capacitor/Models/AppState.swift` | App coordinator: polling timer, snapshot application |
| Swift | `apps/swift/Sources/Capacitor/Models/SessionStateManager.swift` | State projection: matching, hysteresis, flash animations |
| Swift | `apps/swift/Sources/Capacitor/Models/RuntimeClient.swift` | HTTP client to runtime service |
| Swift | `apps/swift/Sources/Capacitor/Models/HookServerManager.swift` | Server lifecycle: launch, adopt, health-check, restart |
| Swift | `apps/swift/Sources/Capacitor/Helpers/HookInstaller.swift` | Binary installation to `~/.local/bin/hud-hook` |
| Swift | `apps/swift/Sources/Capacitor/Utilities/SessionStaleness.swift` | Stale-working detection (30s threshold) |
