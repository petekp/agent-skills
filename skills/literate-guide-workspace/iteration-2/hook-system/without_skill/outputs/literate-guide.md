# The Capacitor Hook System: A Literate Guide

*In the tradition of Knuth's literate programming, this guide presents the
hook system not in the order a compiler demands, but in the order a human
mind can best absorb it. We begin with the big picture and spiral inward,
weaving prose and code together until the whole mechanism lies transparent
before us.*

---

## 1. What Problem Are We Solving?

Capacitor is a desktop HUD (heads-up display) for Claude Code. It needs to
know what every Claude Code session is doing -- right now, in real time.
Is a session working? Waiting for user permission? Compacting its context
window? Has it finished?

Claude Code exposes a **hook system**: at key lifecycle moments, it invokes
a configured binary with a JSON payload describing what just happened. The
`hud-hook` crate is the binary on the receiving end of those hooks. Its job
is deceptively simple: receive a JSON event, determine what state the
session should be in, and tell the core runtime.

The difficulty lies in doing this reliably, quickly, and without corrupting
state -- even when events arrive out of order, from subagents that share
their parent's session ID, or without a working directory at all.

---

## 2. Architecture at a Glance

The system has four layers, each corresponding to a source file in
`core/hud-hook/src/`:

```
                    Claude Code
                        |
                   (HTTP POST)
                        |
                        v
                 +-------------+
                 |  serve.rs   |   HTTP server (tiny_http)
                 +------+------+
                        |
                        v
                 +-------------+
                 |  handle.rs  |   Event handler & state machine
                 +------+------+
                        |
                        v
             +-------------------+
             | runtime_client.rs |   Transport layer to core runtime
             +-------------------+
                        |
                        v
                +---------------+
                | capacitor-core|   Reducer, snapshot, projection
                +---------------+
```

Two auxiliary modules complete the picture:

- **`hook_types.rs`** -- the data model for incoming hook payloads
- **`cwd.rs`** -- a separate subcommand for shell working-directory tracking
- **`logging.rs`** -- structured logging with daily rotation
- **`main.rs`** -- the CLI entry point that wires everything together

Let us now walk through each layer.

---

## 3. The Entry Point: `main.rs`

The binary offers two subcommands. The first, `serve`, starts a long-lived
HTTP server. The second, `cwd`, is a fire-and-forget command invoked by
shell prompt hooks to report directory changes.

```rust
#[derive(Subcommand)]
enum Commands {
    /// Run the local runtime service for hook ingress and runtime reads
    Serve {
        #[arg(long, default_value = "7474")]
        port: u16,
    },

    /// Report shell current working directory
    Cwd {
        #[arg(value_name = "PATH")]
        path: String,
        #[arg(value_name = "PID")]
        pid: u32,
        #[arg(value_name = "TTY")]
        tty: String,
    },
}
```

The `main` function is brief. It initializes structured logging, parses
arguments with `clap`, and dispatches:

```rust
fn main() {
    let _logging_guard = logging::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port } => {
            if let Err(e) = serve::run(port) {
                tracing::error!(error = %e, "hud-hook serve failed");
                std::process::exit(1);
            }
        }
        Commands::Cwd { path, pid, tty } => {
            if let Err(e) = cwd::run(&path, pid, &tty) {
                eprintln!("hud-hook cwd failed: {e}");
                std::process::exit(1);
            }
        }
    }
}
```

Note the `_logging_guard`. The tracing system uses a non-blocking file
appender, and the guard ensures buffered log entries are flushed before the
process exits. Dropping it prematurely would lose log data.

A small but important detail lives here for tests:

```rust
#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    pub(crate) fn env_lock() -> MutexGuard<'static, ()> {
        match ENV_LOCK.get_or_init(|| Mutex::new(())).lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}
```

Tests that manipulate environment variables (a global, mutable resource)
must serialize. This lock ensures that even if Rust's test harness runs
tests in parallel, environment mutations don't interleave.

---

## 4. The Data Model: `hook_types.rs`

Before we can process events, we must understand what they look like.
Claude Code sends a JSON payload when a hook fires. The `HookInput` struct
is the deserialized form of that payload:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct HookInput {
    pub hook_event_name: Option<String>,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub notification_type: Option<String>,
    pub stop_hook_active: Option<bool>,
    pub tool_name: Option<String>,
    pub tool_input: Option<ToolInput>,
    pub tool_response: Option<ToolResponse>,
    pub agent_id: Option<String>,
    pub teammate_name: Option<String>,
}
```

Every field is `Option`. This is defensive by design: Claude Code is an
evolving system, and new hook event types may carry different fields. The
system must degrade gracefully when fields are absent.

### 4.1 From Strings to a Typed Enum

The raw `hook_event_name` is a freeform string. The `to_event()` method
converts it into a proper Rust enum, `HookEvent`, which the rest of the
system can pattern-match on exhaustively:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum HookEvent {
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PreToolUse   { tool_name: Option<String>, file_path: Option<String> },
    PostToolUse  { tool_name: Option<String>, file_path: Option<String> },
    PostToolUseFailure { tool_name: Option<String>, file_path: Option<String> },
    PermissionRequest,
    PreCompact,
    Notification { notification_type: String },
    SubagentStart,
    SubagentStop,
    Stop         { stop_hook_active: bool },
    TeammateIdle,
    TaskCompleted,
    WorktreeCreate,
    WorktreeRemove,
    ConfigChange,
    Unknown      { event_name: String },
}
```

The `Unknown` variant is the safety valve. When Claude Code adds a new
hook event that this binary doesn't yet know about, it lands here rather
than causing a deserialization failure or panic. The handler will log it
and move on.

The conversion method `to_event()` performs some noteworthy field
resolution. For tool-related events, the file path can come from either
`tool_input` or `tool_response`:

```rust
"PostToolUse" => {
    let file_path = tool_input_file_path().or_else(|| {
        self.tool_response
            .as_ref()
            .and_then(|tr| tr.file_path.clone())
    });
    HookEvent::PostToolUse {
        tool_name: self.tool_name.clone(),
        file_path,
    }
}
```

This fallback chain (`tool_input.file_path` -> `tool_input.path` ->
`tool_response.filePath`) handles the different shapes that tool payloads
take across Claude Code's various tools.

### 4.2 Resolving the Working Directory

The `resolve_cwd` method determines which directory a session is operating
in. The resolution order is:

1. The `cwd` field from the hook payload itself (highest priority)
2. A `current_cwd` parameter passed by the caller (used when the server
   already knows the session's working directory from a previous event)
3. If neither is available, `None` -- and the event will be skipped

```rust
pub fn resolve_cwd(&self, current_cwd: Option<&str>) -> Option<String> {
    self.cwd
        .clone()
        .or_else(|| current_cwd.map(ToString::to_string))
        .map(|cwd| normalize_path(&cwd))
}
```

Path normalization strips trailing slashes, ensuring `/repo/path/` and
`/repo/path` are treated as the same directory.

---

## 5. The HTTP Server: `serve.rs`

The server is the front door. It binds to `127.0.0.1:{port}` (default
7474), listens for HTTP requests, and dispatches them to the appropriate
handler.

### 5.1 Server Lifecycle

```rust
pub fn run(port: u16) -> Result<(), String> {
    install_signal_handlers();

    let addr = format!("127.0.0.1:{port}");
    let server = tiny_http::Server::http(&addr)
        .map_err(|e| format!("Failed to bind {addr}: {e}"))?;
    let runtime_service = RuntimeServerState::new(port)?;

    tracing::info!(port, "hud-hook serve listening");

    let _pid_guard = PidFile::write(port, runtime_service.bootstrap.is_some())?;
    // ...

    loop {
        if SHUTDOWN.load(Ordering::Relaxed) {
            tracing::info!("Shutdown signal received, exiting");
            break;
        }

        let request = match server.recv_timeout(
            std::time::Duration::from_millis(500)
        ) {
            Ok(Some(req)) => req,
            Ok(None) => continue,
            Err(e) => { tracing::warn!(...); continue; }
        };

        dispatch(request, &runtime_service);
    }

    Ok(())
}
```

Several design choices are worth highlighting:

**Polling with timeout.** Rather than blocking indefinitely on
`incoming_requests()`, the server polls with a 500ms timeout. This lets it
check the `SHUTDOWN` flag periodically, ensuring clean termination when
SIGTERM or SIGINT arrives.

**Signal handling.** The signal handler is minimal -- it sets an atomic
boolean. No heap allocation, no locks, no I/O. This is important because
signal handlers execute in an async-signal context where most operations
are undefined behavior.

```rust
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

extern "C" fn signal_handler(_sig: libc::c_int) {
    SHUTDOWN.store(true, Ordering::Relaxed);
}
```

**PID file with RAII cleanup.** The `PidFile` struct writes the process
ID to `~/.capacitor/runtime/hud-hook-serve-{port}.pid` on creation and
removes it on drop:

```rust
struct PidFile {
    path: std::path::PathBuf,
}

impl Drop for PidFile {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.path) {
            tracing::warn!(...);
        }
    }
}
```

This pattern -- sometimes called a "guard" -- ensures cleanup happens even
if the server exits via an error path.

### 5.2 Route Dispatch

The server exposes five endpoints:

```rust
fn dispatch(request: tiny_http::Request, runtime_service: &RuntimeServerState) {
    match (request.method(), request.url()) {
        (&tiny_http::Method::Get,  "/health")                      => handle_health(...),
        (&tiny_http::Method::Get,  "/runtime/snapshot")             => handle_runtime_snapshot(...),
        (&tiny_http::Method::Post, "/runtime/ingest/hook-event")    => handle_runtime_ingest_hook_event(...),
        (&tiny_http::Method::Post, "/runtime/ingest/shell-signal")  => handle_runtime_ingest_shell_signal(...),
        (&tiny_http::Method::Post, "/hook")                         => handle_hook(...),
        _ => { let _ = request.respond(json_error(404, "not found")); }
    }
}
```

The endpoints fall into two categories:

1. **Legacy hook endpoint** (`POST /hook`): receives the raw Claude Code
   hook JSON, runs it through the `handle.rs` state machine, and forwards
   it to the core runtime.

2. **Runtime service endpoints** (`/runtime/*`): a more structured API
   with bearer-token authentication. These allow the core runtime to be
   used as a proper service -- ingesting events, reading snapshots, and
   checking health.

### 5.3 Body Reading and Safety

The server enforces a 1 MiB body limit to prevent memory exhaustion.
This limit is checked both when `Content-Length` is present and during
streaming reads (for chunked transfer encoding):

```rust
const MAX_BODY_BYTES: u64 = 1_024 * 1_024;

fn read_request_body(request: &mut tiny_http::Request) -> Result<String, ...> {
    if let Some(len) = request.body_length() {
        if (len as u64) > MAX_BODY_BYTES {
            return Err(json_error(413, "body too large"));
        }
    }

    let mut body_bytes = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        match std::io::Read::read(&mut reader, &mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if (body_bytes.len() + n) as u64 > MAX_BODY_BYTES {
                    return Err(json_error(413, "body too large"));
                }
                body_bytes.extend_from_slice(&chunk[..n]);
            }
            Err(error) => return Err(json_error(400, "failed to read body")),
        }
    }
    // ...
}
```

The double check (pre-read and during-read) is deliberate. The first catches
well-behaved clients that declare their body size. The second catches
chunked-encoding clients or those that lie about `Content-Length`.

### 5.4 Runtime Server State

When the server starts, it creates a `RuntimeServerState` that optionally
bootstraps an authenticated runtime service:

```rust
struct RuntimeServerState {
    bootstrap: Option<RuntimeServiceBootstrap>,
    runtime: Option<Arc<CoreRuntime>>,
}
```

If the `CAPACITOR_RUNTIME_SERVICE_BOOTSTRAP` environment variable is set,
the server creates a `RuntimeServiceBootstrap` with a bearer token for
authentication. It also writes a token file to disk so that other
processes (like `hud-hook cwd`) can discover the service endpoint.

The `CoreRuntime` is wrapped in an `Arc` so it can be shared: the server
registers it with the `runtime_client` module, allowing the `POST /hook`
handler to use the same runtime instance directly rather than making a
loopback HTTP call.

---

## 6. The State Machine: `handle.rs`

This is the heart of the system. The `handle_hook_input` function receives
a deserialized `HookInput` and determines what to do with it. The logic is
a careful sequence of guards, each ruling out a class of events before the
main processing begins.

### 6.1 The Guard Sequence

```
    HookInput arrives
         |
         v
    to_event() -> None?  ──> return Ok(())     [no event name]
         |
         v
    Unknown event?       ──> log & return Ok()  [forward-compatible]
         |
         v
    Missing session_id?  ──> log & return Ok()  [can't track without ID]
         |
         v
    Runtime disabled?    ──> return Err(...)     [CAPACITOR_CORE_ENABLED=0]
         |
         v
    Subagent Stop?       ──> return Ok(())      [protect parent state]
         |
         v
    Missing cwd          ──> return Ok()        [except for Delete]
    (and not Delete)?
         |
         v
    Send to runtime
```

Each guard is instrumented with structured tracing fields that follow a
consistent schema:

```rust
tracing::debug!(
    gate_id = SESSION_STATE_GATE_ID,
    scenario_id = SESSION_STATE_MAPPING_SCENARIO_ID,
    classification = "stateful_noop",
    transition = "skip",
    skip_reason = "subagent_stop_guard",
    // ...
    "Skipping subagent Stop event"
);
```

The `gate_id` and `scenario_id` fields exist for test traceability. The
integration tests in `session_state_mapping_gate.rs` verify that every
known hook event maps to the correct session state -- a form of
specification-as-test.

### 6.2 The Subagent Stop Guard

This guard deserves special attention because it addresses a subtle
problem. When Claude Code spawns a subagent (e.g., for parallel research),
the subagent shares the parent session's `session_id`. When the subagent
finishes, it fires a `Stop` event with that shared ID.

Without the guard, this would transition the parent session to `Ready`
even though the parent might still be `Working`:

```rust
if matches!(event, HookEvent::Stop { .. }) && hook_input.agent_id.is_some() {
    // Skip -- this is a subagent's Stop, not the parent's
    return Ok(());
}
```

The discriminator is `agent_id`: subagent events carry an `agent_id` field
that the main agent's events lack.

### 6.3 The State Transition Function

The `process_event` function is a pure function (no side effects) that
maps `(event, current_state)` to `(action, new_state)`:

```rust
fn process_event(
    event: &HookEvent,
    current_state: Option<SessionState>,
    input: &HookInput,
) -> (Action, Option<SessionState>, Option<(String, String)>) {
    match event {
        HookEvent::SessionStart => {
            if is_active_state(current_state) {
                (Action::Skip, None, None)
            } else {
                (Action::Upsert, Some(SessionState::Ready), None)
            }
        }

        HookEvent::UserPromptSubmit =>
            (Action::Upsert, Some(SessionState::Working), None),

        HookEvent::PreToolUse { .. } |
        HookEvent::PostToolUse { .. } |
        HookEvent::PostToolUseFailure { .. } => {
            if current_state == Some(SessionState::Working) {
                (Action::Refresh, None, None)   // already working, just refresh timestamp
            } else {
                (Action::Upsert, Some(SessionState::Working), None)
            }
        }

        HookEvent::PermissionRequest =>
            (Action::Upsert, Some(SessionState::Waiting), None),

        HookEvent::PreCompact =>
            (Action::Upsert, Some(SessionState::Compacting), None),

        HookEvent::SessionEnd =>
            (Action::Delete, None, None),

        // ...
    }
}
```

The actions form a small algebra:

| Action    | Meaning |
|-----------|---------|
| `Upsert`  | Create or update the session with a new state |
| `Refresh` | Touch the session's timestamp without changing state |
| `Delete`  | Remove the session record entirely |
| `Skip`    | Do nothing -- this event is not relevant |

### 6.4 The Complete State Machine

The following table shows every hook event and its effect on session state.
This is the same table documented in the module header and verified
exhaustively by integration tests:

```
Event                   New State       Condition
-----                   ---------       ---------
SessionStart            Ready           (unless already active)
UserPromptSubmit        Working
PreToolUse              Working         (Refresh if already Working)
PostToolUse             Working         (Refresh if already Working)
PostToolUseFailure      Working         (Refresh if already Working)
PermissionRequest       Waiting
PreCompact              Compacting
Notification            Ready           (idle_prompt, auth_success)
Notification            Waiting         (permission_prompt, elicitation_dialog)
Notification            Skip            (any other notification_type)
Stop                    Ready           (unless stop_hook_active=true)
Stop                    Skip            (stop_hook_active=true)
TaskCompleted           Ready           (main agent only)
TaskCompleted           Skip            (subagent or teammate)
SessionEnd              Delete
SubagentStart           Skip            (informational)
SubagentStop            Skip            (informational)
TeammateIdle            Skip            (informational)
WorktreeCreate          Skip            (informational)
WorktreeRemove          Skip            (informational)
ConfigChange            Skip            (informational)
Unknown                 Skip            (forward-compatible)
```

The `SessionState` enum itself has five variants with explicit priority:

```rust
pub enum SessionState {
    Working,    // priority 3
    Ready,      // priority 1
    Idle,       // priority 0 (default)
    Compacting, // priority 2
    Waiting,    // priority 4 (highest)
}
```

Priority matters when the HUD needs to determine the aggregate state of a
project with multiple sessions. The highest-priority session state
"wins" as the project's representative state. `Waiting` is highest because
it requires human attention.

---

## 7. The Runtime Client: `runtime_client.rs`

This module is the bridge between the hook handler and the core runtime.
It has a dual personality: inside the `hud-hook serve` process it talks
directly to the registered `CoreRuntime` instance; outside that process it
discovers the runtime service endpoint and communicates via HTTP.

### 7.1 Transport Selection

```rust
enum RuntimeTransport {
    Service(RuntimeServiceEndpoint),
    RegisteredService(Arc<CoreRuntime>),
}

fn runtime_transport() -> Result<RuntimeTransport, String> {
    if !runtime_enabled() {
        return Err("Core runtime disabled".to_string());
    }

    // Prefer in-process runtime if available (we're inside `serve`)
    if let Some(runtime) = REGISTERED_SERVICE_RUNTIME.get() {
        return Ok(RuntimeTransport::RegisteredService(Arc::clone(runtime)));
    }

    // Otherwise, discover the service endpoint on disk
    runtime_service_endpoint()?
        .map(RuntimeTransport::Service)
        .ok_or_else(|| "runtime service endpoint unavailable".to_string())
}
```

The `REGISTERED_SERVICE_RUNTIME` is a `OnceLock<Arc<CoreRuntime>>` -- a
global singleton that the server sets during initialization. This avoids
the overhead of HTTP serialization when events are handled in the same
process.

### 7.2 Event Construction

The `send_handle_event` function translates from the hook-level
`HookEvent` enum into the core domain's `IngestHookEventCommand`:

```rust
pub fn send_handle_event(
    event: &HookEvent,
    hook_input: &HookInput,
    session_id: &str,
    pid: Option<u32>,
    cwd: &str,
) -> bool {
    let event_type = match event_type_for_hook(event) {
        Some(event_type) => event_type,
        None => return false,
    };

    let command = IngestHookEventCommand {
        event_id: make_event_id(pid.unwrap_or(0)),
        recorded_at: Utc::now().to_rfc3339(),
        event_type,
        session_id: session_id.to_string(),
        pid,
        project_path: cwd.to_string(),
        cwd: Some(cwd.to_string()),
        file_path: event_file_path(event, hook_input),
        // ...
    };

    send_event(command).is_ok()
}
```

Event IDs are generated with a timestamp-PID-random scheme:

```rust
fn make_event_id(pid: u32) -> String {
    let mut random = rand::thread_rng();
    let rand = random.next_u64();
    format!("evt-{}-{}-{:x}", Utc::now().timestamp_millis(), pid, rand)
}
```

This produces IDs like `evt-1710345600000-12345-a3f8b2c1` -- globally
unique without requiring a UUID library, naturally sortable by time.

### 7.3 The Enable Flag

A simple environment variable controls whether the runtime processes
events:

```rust
pub fn runtime_enabled() -> bool {
    env_flag(ENABLE_ENV).unwrap_or(true)
}
```

The default is `true` -- the runtime is on unless explicitly disabled.
The `env_flag` parser accepts the full gamut of boolean representations:
`1`, `true`, `yes`, `on` and their negations.

---

## 8. Shell CWD Tracking: `cwd.rs`

The `cwd` subcommand solves a different problem from hook events. Shell
prompt hooks (e.g., `precmd` in zsh) invoke `hud-hook cwd` every time
the user changes directories. This gives Capacitor "ambient awareness" of
which projects are active in which terminals, even when Claude Code isn't
running.

### 8.1 Design Constraints

The module header states the performance target explicitly:

```
Target: < 15ms total execution time.
The shell spawns this in the background, so users never wait.
```

Despite running in the background, speed matters because slow background
processes accumulate and can degrade shell responsiveness.

### 8.2 The Data Model

Each CWD report creates a `ShellEntry`:

```rust
pub struct ShellEntry {
    pub cwd: String,
    pub tty: String,
    pub parent_app: ParentApp,
    pub tmux_session: Option<String>,
    pub tmux_client_tty: Option<String>,
    pub updated_at: DateTime<Utc>,
}
```

The `parent_app` field identifies which terminal emulator or IDE hosts the
shell. Detection uses `TERM_PROGRAM` and `TERM` environment variables:

```rust
fn detect_parent_app(_pid: u32) -> ParentApp {
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        let normalized = term_program.to_lowercase();
        match normalized.as_str() {
            "iterm.app" | "iterm2"               => return ParentApp::ITerm,
            "apple_terminal" | "terminal.app"    => return ParentApp::Terminal,
            "warpterminal" | "warp"              => return ParentApp::Warp,
            "ghostty"                            => return ParentApp::Ghostty,
            "vscode"                             => return ParentApp::VSCode,
            "cursor"                             => return ParentApp::Cursor,
            "zed"                                => return ParentApp::Zed,
            _ => {}
        }
    }
    // ... fallback checks for TERM, TMUX ...
    ParentApp::Unknown
}
```

### 8.3 Path Normalization on Case-Insensitive Filesystems

macOS has a case-insensitive (but case-preserving) filesystem by default.
A user might `cd` to `/Users/alice/Code` but the canonical path might be
`/Users/Alice/code`. The `normalize_path` function resolves this by
walking the path components and matching each one against the actual
directory listing:

```rust
fn merge_canonical_case(original: &Path, _canonical: &Path) -> String {
    let original_parts = path_components(original);
    let mut real_path = PathBuf::new();
    if original.is_absolute() {
        real_path.push("/");
    }

    for part in &original_parts {
        let mut found_match = false;
        if let Ok(entries) = std::fs::read_dir(&real_path) {
            for entry in entries.filter_map(Result::ok) {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.eq_ignore_ascii_case(part) {
                        real_path.push(name);
                        found_match = true;
                        break;
                    }
                }
            }
        }
        if !found_match {
            real_path.push(part);
        }
    }
    // ...
}
```

This ensures that the same physical directory always maps to the same
string, preventing phantom duplicate projects in the HUD.

### 8.4 Tmux Awareness

When the shell runs inside tmux, the CWD tracker captures the tmux
session name and client TTY. This information flows into the routing
system, which determines how to navigate to a project's terminal:

```rust
fn detect_tmux_context() -> Option<(String, String)> {
    let mut child = std::process::Command::new("tmux")
        .args(["display-message", "-p", "#S\t#{client_tty}"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .ok()?;

    let timeout = Duration::from_millis(500);
    match child.wait_timeout(timeout).ok()? {
        Some(status) if status.success() => {
            // parse "session_name\t/dev/ttysNNN"
        }
        _ => None,
    }
}
```

The 500ms timeout prevents a stuck tmux command from blocking the shell.
If tmux doesn't respond in time, the child process is killed and the
information is simply omitted.

---

## 9. Logging: `logging.rs`

The logging module uses `tracing` with a file appender that rotates daily
and keeps seven days of history:

```rust
fn create_file_appender(
    capacitor_dir: &PathBuf,
) -> Result<RollingFileAppender, tracing_appender::rolling::InitError> {
    RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("hud-hook-debug")
        .filename_suffix("log")
        .max_log_files(7)
        .build(capacitor_dir)
}
```

Logs go to `~/.capacitor/hud-hook-debug.{date}.log`. If the file appender
can't be created (e.g., permissions), it falls back to stderr. The default
filter level is `hud_hook=debug,capacitor_core=warn`, adjustable via
`RUST_LOG`.

---

## 10. The Core Runtime Pipeline

When an event survives all guards and reaches the core runtime, it passes
through a three-stage pipeline:

### 10.1 Ingest: Normalization

The `ingest` module normalizes the incoming command: trimming whitespace,
stripping trailing slashes from paths, and converting empty strings to
`None`:

```rust
pub fn normalize_hook_event(command: IngestHookEventCommand) -> IngestHookEventCommand {
    IngestHookEventCommand {
        event_id: command.event_id.trim().to_string(),
        project_path: normalize_required_path(&command.project_path),
        cwd: normalize_optional_path(command.cwd),
        file_path: normalize_optional_path(command.file_path),
        notification_type: normalize_optional_text(command.notification_type),
        tool_name: normalize_optional_text(command.tool_name),
        // ...
    }
}
```

This defensive normalization ensures that path-matching logic downstream
never fails due to trivial formatting differences.

### 10.2 Reduce: State Application

The `ReducerState` is the canonical state store. It holds maps of
projects, sessions, and shells. When a hook event arrives, the reducer:

1. Validates the event (non-empty `event_id`, non-empty `session_id`)
2. Checks for staleness (events older than 5 seconds compared to the
   session's last update are rejected)
3. Computes the session update via `reduce_session()`
4. Applies the update (upsert, delete, or skip)
5. Recomputes project aggregates and routing

```rust
pub fn apply_hook_event(&mut self, command: IngestHookEventCommand) -> MutationOutcome {
    self.events_ingested = self.events_ingested.saturating_add(1);
    // ... validation ...

    let current = self.sessions.get(&command.session_id).cloned();
    if is_event_stale(current.as_ref(), &command) {
        self.stale_events_skipped += 1;
        return MutationOutcome { ok: true, message: "stale event skipped".to_string() };
    }

    let update = reduce_session(current.as_ref(), &command);
    match &update {
        SessionUpdate::Upsert(session) => {
            self.sessions.insert(session.session_id.clone(), session.clone());
        }
        SessionUpdate::Delete(session_id) => {
            self.sessions.remove(session_id);
        }
        SessionUpdate::Skip(reason) => { /* increment counters */ }
    }

    self.recompute_projects();
    self.recompute_routing();
    // ...
}
```

### 10.3 Snapshot: Materialized View

After every mutation, the reducer can produce an `AppSnapshot` -- a
complete materialized view of the world:

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

This snapshot is what the HUD's UI reads. It contains everything needed
to render the dashboard: which projects exist, what state each session is
in, which shells are active, and how to route focus to the right terminal.

---

## 11. The Runtime Service Protocol

The runtime service adds an authentication and API layer on top of the
core runtime, enabling secure access from multiple processes.

### 11.1 Service Discovery

When `hud-hook serve` starts in bootstrap mode, it writes two files:

1. `~/.capacitor/runtime/runtime-service-{port}.token` -- the bearer token
2. `~/.capacitor/runtime/runtime-service.json` -- port + token as JSON

Other processes (like `hud-hook cwd`) discover the service by reading
these files:

```rust
pub fn discover(home_dir: &Path, default_port: u16) -> Result<Option<Self>, String> {
    // 1. Check environment variables
    if let Some(endpoint) = Self::from_env()? {
        return Ok(Some(endpoint));
    }

    // 2. Check connection file
    let connection_path = RuntimeServiceBootstrap::connection_file_path(home_dir);
    if connection_path.exists() {
        // parse and return
    }

    // 3. Check token file
    let token_path = RuntimeServiceBootstrap::token_file_path(home_dir, default_port);
    if !token_path.exists() {
        return Ok(None);
    }
    // ...
}
```

Both files are cleaned up automatically when the server shuts down, thanks
to RAII guards (`RuntimeServiceTokenGuard` and `PidFile`).

### 11.2 Authentication

Every runtime service endpoint requires a bearer token:

```rust
fn authorize_runtime_request(
    request: &tiny_http::Request,
    runtime_service: Option<&RuntimeServiceBootstrap>,
) -> bool {
    let Some(bootstrap) = runtime_service else {
        return false;
    };

    let authorization = request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Authorization"))
        .map(|header| header.value.as_str());

    bootstrap.is_authorized(authorization)
}
```

This prevents unauthorized processes from reading session state or
injecting events. The token is generated at startup and shared only
through filesystem files with appropriate permissions.

---

## 12. Testing Strategy

The test suite is layered to match the architecture.

### 12.1 Unit Tests

Each module has inline `#[cfg(test)]` blocks testing pure functions:

- `hook_types.rs` tests CWD resolution logic
- `handle.rs` tests the subagent Stop guard and unknown event handling
- `runtime_client.rs` tests boolean parsing and event type mapping
- `cwd.rs` tests path normalization on case-insensitive filesystems

### 12.2 Integration Tests

The `tests/` directory contains full-stack tests that spawn actual
`hud-hook serve` processes:

**`serve_integration.rs`** tests HTTP-level behavior:
- Health endpoint returns 200
- Bootstrap mode requires authentication
- Hook events flow through to snapshots
- Invalid JSON returns 400
- Oversized bodies return 413
- PID files are written and cleaned up

**`session_state_mapping_gate.rs`** is a specification test. It sends
every known hook event type through the full pipeline and asserts the
resulting session state in the snapshot:

```rust
let cases = vec![
    MappingCase { hook_event_name: "SessionStart",      expected_state: Some("ready") },
    MappingCase { hook_event_name: "SessionEnd",         expected_state: None },
    MappingCase { hook_event_name: "UserPromptSubmit",   expected_state: Some("working") },
    MappingCase { hook_event_name: "PreToolUse",         expected_state: Some("working") },
    MappingCase { hook_event_name: "PermissionRequest",  expected_state: Some("waiting") },
    MappingCase { hook_event_name: "PreCompact",         expected_state: Some("compacting") },
    // ... every event type
];
```

Each case spawns its own server instance to ensure complete isolation.
This is expensive but eliminates test-order dependencies.

Additional tests verify edge cases:
- Subagent `Stop` events don't affect parent session state
- Unknown notification types don't mutate state
- Determinism: the same input always produces the same snapshot
- Missing `cwd` skips non-delete events but allows `SessionEnd`

---

## 13. Data Flow Summary

To tie it all together, here is the complete lifecycle of a hook event,
from Claude Code to the HUD:

```
1. Claude Code fires a hook (e.g., user submits a prompt)

2. Claude Code POSTs JSON to http://127.0.0.1:7474/hook
   {
     "hook_event_name": "UserPromptSubmit",
     "session_id": "session-abc123",
     "cwd": "/Users/alice/my-project"
   }

3. serve.rs receives the request, reads the body (< 1 MiB),
   deserializes it into HookInput

4. handle.rs runs the guard sequence:
   - Event name present?     yes
   - Known event?            yes (UserPromptSubmit)
   - Session ID present?     yes
   - Runtime enabled?        yes
   - Subagent Stop?          no
   - CWD present?            yes

5. handle.rs calls process_event():
   UserPromptSubmit -> (Upsert, Working)

6. runtime_client.rs builds an IngestHookEventCommand:
   - event_id: "evt-1710345600000-0-a3f8b2c1"
   - event_type: UserPromptSubmit
   - session_id: "session-abc123"
   - project_path: "/Users/alice/my-project"

7. Transport selection:
   - In-process (same `serve` process): call runtime.ingest_hook_event() directly
   - Out-of-process: POST to /runtime/ingest/hook-event with bearer token

8. capacitor-core normalizes the command (trim, strip slashes)

9. The reducer:
   - Validates event_id and session_id
   - Checks staleness (< 5s grace)
   - Computes session update: Upsert with state=Working
   - Recomputes project aggregates
   - Recomputes routing table

10. The updated AppSnapshot is now available via GET /runtime/snapshot

11. The HUD's Swift UI polls the snapshot and updates the display:
    "my-project: Working"
```

---

## 14. Design Principles

Several principles recur throughout the codebase:

**Defensive optionality.** Every field in the incoming payload is
`Option`. Every resolution chain has fallbacks. The system never panics
on missing data -- it degrades gracefully, typically by skipping the event.

**RAII for cleanup.** PID files, token files, and logging guards all use
Rust's drop semantics to ensure cleanup happens regardless of exit path.

**Pure state transitions.** The `process_event` function is pure -- it
takes inputs and returns outputs with no side effects. This makes it
trivially testable and easy to reason about.

**Forward compatibility.** Unknown events and unknown notification types
are handled explicitly. When Claude Code adds new hook events, the binary
won't crash -- it will log and skip them until a new version is deployed.

**Structured tracing.** Every significant decision is logged with
machine-parseable fields (`gate_id`, `classification`, `transition`,
`skip_reason`). This enables automated analysis of hook behavior in
production.

**Specification-as-test.** The mapping gate test is both a test and a
specification. Reading it tells you exactly what each event should do,
and running it proves the implementation matches.

---

*This concludes the literate guide to Capacitor's hook system. The
code lives in `core/hud-hook/` with its domain types in
`core/capacitor-core/src/domain/`. The system is small -- roughly 1,500
lines of Rust across six source files -- but each line earns its keep
in a design that values reliability, forward compatibility, and
transparent state management above all else.*
