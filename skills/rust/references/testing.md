# Testing & Quality

Testing strategies and quality tooling for robust Rust systems.

## Unit Tests

- **Round-trip tests**: parse → serialize → parse again → same data
- **Mutation tests**: updates only touch intended fields
- **Normalization tests**: case-insensitive values parse equivalently

## Golden Files

Store sample fixtures in `tests/fixtures/` and assert exact output after transformations.

## Contract Tests

Create fixture files representing real-world scenarios:
- Verified data with all fields
- Legacy data with old field names
- Missing timestamps
- Corrupted JSON
- Unit mismatches (ms vs seconds)

## Property Tests

Use `proptest` to generate random inputs and ensure:
- Parser doesn't panic
- IDs are preserved through round-trips

## Fuzz Testing

For deeper coverage of parser edge cases:

```bash
cargo install cargo-fuzz
cargo fuzz init
cargo fuzz run parse_fuzz_target
```

A fuzz target:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = my_crate::parse(s);  // Should never panic
    }
});
```

Fuzz testing finds edge cases proptest may miss—particularly valuable for parsers and string processing.

## Subprocess Integration Tests

Don't depend on real external processes in CI:
- Provide a fake executable in PATH that returns deterministic output
- Verify timeout and parsing logic

## Essential Test Cases

- [ ] Version marker: missing/wrong → error
- [ ] Empty file: initializes properly
- [ ] Metadata injection: description resembling metadata stays in description
- [ ] Anchored updates: update affects only intended block
- [ ] Duplicate IDs: deterministic dedupe enforced
- [ ] PID correlation: mismatched PID rejected

## Quality Tools

All code should pass before commit:

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

### Strict Clippy Lints

For library code, add to `Cargo.toml`:

```toml
[lints.clippy]
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
indexing_slicing = "warn"
```
