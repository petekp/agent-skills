# File I/O & Persistence

Patterns for durable writes, concurrency, and file watching.

## Durable-First Invariant

**Never** gate persistence on validation or subprocess output:

1. Write raw data to storage (e.g., `status: pending`)
2. Return control immediately
3. Async enrichment updates later (or not at all)

## Atomic Writes

Write to temp file, sync to disk, then rename:

```rust
use std::{io::Write, path::Path};
use tempfile::NamedTempFile;

fn atomic_write(path: &Path, contents: &str) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(dir)?;
    tmp.write_all(contents.as_bytes())?;
    tmp.flush()?;
    tmp.as_file().sync_all()?;  // Critical: ensure OS flushes before rename
    tmp.persist(path).map(|_| ()).map_err(|e| e.error)
}
```

The `sync_all()` is critical—without it, power loss after rename could leave a truncated file.

## Concurrency Control

**In-process:** `Mutex` works

**Out-of-process:** Make writes merge-friendly:
1. Re-read file before write
2. Apply patch to latest parsed model
3. Write back atomically

Advisory locking (`fs2::FileExt`) is an option for stronger guarantees.

## File Watching

Use debounced file watchers for external edit detection:

- Watcher thread pushes events into a channel
- Model thread parses and computes diffs
- UI receives "data changed for X" (avoid huge payloads)

The `notify` crate recommends debouncing because editors save via multiple quick writes (write temp + rename).
