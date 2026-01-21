---
name: rust
description: Robust Rust patterns for file-backed data, parsing, persistence, FFI boundaries, and system integration. Use when writing Rust that handles file formats, subprocess integration, PID/process management, Serde serialization, or UniFFI boundaries. Covers UTF-8 safety, atomic writes, state machines, and defensive error handling.
---

# Rust Engineering Guide

Patterns for building reliable Rust systems that handle file-backed data, external process integration, and cross-language boundaries.

## Core Philosophy

**Conservative by Default**: Inputs from files, subprocesses, and external systems are potentially untrusted. Rust code should be:
- Conservative: Prefer false negatives over false positives
- Deterministic: Same input → same output
- Resilient: Never panic on user machines due to bad input

**Canonical Model Ownership**: If Rust is the source of truth:
- Internal domain model: Expressive, ergonomic
- FFI DTOs: Boring, stable, language-friendly
- File format model: Stable, versioned, round-trippable
- External input model: Strictly validated, never trusted

**Safe Rust Only**: None of these patterns require `unsafe`. Use ecosystem crates (`tempfile`, `fs2`, `sysinfo`, `uniffi`, `unicode-segmentation`) for safe abstractions.

---

## Reference Guides

Load the relevant reference when working in that domain:

| Domain | Reference | When to Use |
|--------|-----------|-------------|
| **Data Modeling** | [data-modeling.md](references/data-modeling.md) | Serde patterns, UniFFI boundaries, strong types, versioned schemas |
| **File I/O** | [file-io.md](references/file-io.md) | Atomic writes, concurrency control, durable persistence, file watching |
| **Process Integration** | [process-integration.md](references/process-integration.md) | PID verification, subprocess handling, timestamp normalization |
| **Text & Parsing** | [text-and-parsing.md](references/text-and-parsing.md) | UTF-8 safety, path normalization, state machine parsing, anchored updates |
| **Testing** | [testing.md](references/testing.md) | Round-trip tests, fuzz testing, golden files, Clippy lints |

---

## Error Handling

**Never Panic on Bad Input**: File I/O and JSON parsing must never panic:
- If metadata can't be parsed → ignore that entry (don't crash, don't guess)
- If timestamps are missing → treat conservatively
- Use `Option` and early returns liberally

**Explicit Error Types**: Define a single error type with `thiserror`:

```rust
#[derive(thiserror::Error, Debug)]
pub enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Subprocess failed: {0}")]
    SubprocessFailed(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Unsupported format: expected {expected}, found {found:?}")]
    UnsupportedFormat { expected: String, found: Option<String> },
}
```

**Graceful Degradation**: Errors degrade functionality, not crash:
- Capture succeeds even if validation fails
- Data is preserved even if enrichment times out

---

## Performance

**Cache Compiled Regex**: Use `once_cell::sync::Lazy`:

```rust
static HEADING_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^### \[#item-([A-Z0-9]+)\]").unwrap()
});
```

**Cache Parse Results**: Key by `(mtime, size)` or content hash to avoid re-parsing unchanged files.

**Deterministic Selection**: Directory iteration order is unstable. When selecting from candidates, use a deterministic tiebreaker (e.g., creation time, then path).

---

## Quick Reference

### Do

- Truncate strings with `chars()` or graphemes, not byte slicing
- Anchor identification to heading lines (`^### [#item-...]`)
- Parse metadata only within defined regions
- Normalize keys/values early per spec
- Cache compiled regex and parse results
- Write files atomically with `sync_all()` before rename
- Verify PID identity with process start time
- Use `saturating_sub` for time arithmetic
- Use explicit error variants with `thiserror`
- Run `cargo clippy` and `cargo fmt` before commit

### Don't

- Slice strings with `&s[..N]` without checking char boundaries
- Use `.contains()` to decide which block to update
- Treat metadata patterns anywhere as real metadata
- Ignore version markers if spec requires enforcement
- Recompile regex each parse
- Assume IDs are unique with external edits
- Trust subprocess output without validation
- Mix timestamp units without normalization
- Panic on malformed input
- Use `unsafe` (not needed for these patterns)

---

## Change Checklist

When modifying these systems, verify:

**Schema / Serde**
- [ ] New fields are `Option` + `#[serde(default)]`
- [ ] Old field names supported via `alias`
- [ ] No field meaning repurposed in-place

**Paths**
- [ ] All comparisons use shared normalizer
- [ ] Root handled explicitly (no accidental `//`)
- [ ] Hashing uses normalized paths

**PID Safety**
- [ ] Existence ≠ identity unless legacy mode
- [ ] Verified entries check `proc_started`
- [ ] Legacy mode has mitigations + age expiry

**Timestamps**
- [ ] Units consistent or normalized on read
- [ ] Selection deterministic on ties
- [ ] Age checks immune to unit mismatch

**Robustness**
- [ ] No panics on file I/O or parse errors
- [ ] Unreadable data ignored, not guessed
- [ ] Performance stable under frequent refresh

**Quality**
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] No `unsafe` blocks (unless justified and audited)
