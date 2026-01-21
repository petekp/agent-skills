# Process & System Integration

Patterns for PID verification, subprocess handling, and timestamps.

## PID Liveness vs Identity

`kill(pid, 0)` detects if a PID exists, not if it's the same process. PID reuse creates "ghost" sessions.

**Solution:** Store and verify process start time:

```rust
fn is_pid_alive_verified(pid: u32, expected_start: Option<u64>) -> bool {
    let Some(expected) = expected_start else {
        return is_pid_alive_legacy(pid);
    };

    match get_process_start_time(pid) {
        Some(actual) => actual == expected,
        None => false,
    }
}
```

## Process Start Time With sysinfo

Cache `sysinfo::System` to avoid repeated expensive allocations:

```rust
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

thread_local! {
    static SYSTEM_CACHE: std::cell::RefCell<Option<System>> =
        std::cell::RefCell::new(None);
}

pub fn get_process_start_time(pid: u32) -> Option<u64> {
    SYSTEM_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let sys = cache.get_or_insert_with(|| {
            System::new_with_specifics(
                RefreshKind::new().with_processes(ProcessRefreshKind::new()),
            )
        });
        sys.refresh_processes_specifics(ProcessRefreshKind::new());
        sys.process(Pid::from(pid as usize)).map(|p| p.start_time())
    })
}
```

## Legacy Mitigation

For legacy data without process verification:

1. PID exists (`kill(pid, 0)`)
2. Process identity heuristic (e.g., "claude" in process name)
3. Age expiry (e.g., 24h) for unverified entries

## Subprocess Integration

Treat subprocess output as hostile input:

```rust
use std::process::{Command, Stdio};
use std::io::Write;

pub fn run_subprocess(prompt: &str, stdin_payload: &str) -> anyhow::Result<String> {
    let mut child = Command::new("claude")
        .args(["--print", "--output-format", "json", prompt])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut()
            .ok_or_else(|| anyhow::anyhow!("no stdin"))?;
        stdin.write_all(stdin_payload.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "subprocess failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8(output.stdout)?)
}
```

**Validation:** Parse JSON strictly. If invalid, mark as failed but keep data intact.

**Timeouts:** Enforce timeouts for background operations. On timeout, kill process and record failure.

## Timestamp Handling

The number-one timestamp bug: unit mismatch (seconds vs milliseconds).

```rust
fn normalize_epoch_to_secs(v: u64) -> u64 {
    if v >= 1_000_000_000_000 { v / 1000 } else { v }
}

fn normalize_epoch_to_ms(v: u64) -> u64 {
    if v < 1_000_000_000_000 { v * 1000 } else { v }
}
```

The threshold `1_000_000_000_000` (1e12) distinguishes ms from sec—1e12 ms is ~31,688 years.

Use `saturating_sub` for age computations to prevent underflow:

```rust
let age = now.saturating_sub(created);
```

Parse ISO timestamps for legacy data:

```rust
fn parse_rfc3339_to_secs(s: &str) -> Option<u64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp() as u64)
}
```
