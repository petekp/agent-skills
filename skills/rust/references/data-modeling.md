# Data Modeling

Patterns for Serde serialization, FFI boundaries, and type safety.

## Strong Types Over Strings

Strings for status/effort/triage lead to case-mismatch bugs. Use enums:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status { Open, InProgress, Done, Dismissed }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority { P0, P1, P2, P3 }
```

## Lookup by Primary Key

Use the result's primary key for subsequent lookups—not secondary attributes:

```rust
// BAD: Secondary lookup could return different record
let resolved = resolve_state(&store, project_path)?;
let record = store.find_by_cwd(&resolved.cwd);  // Could match Session B!

// GOOD: Use the resolved session_id for lookup
let resolved = resolve_state(&store, project_path)?;
let record = resolved.session_id
    .as_deref()
    .and_then(|id| store.get_by_session_id(id));
```

## Versioned Data With Serde

When adding fields to serialized structures, old data won't have them:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub pid: u32,
    pub path: String,

    #[serde(default)]
    pub proc_started: Option<u64>,  // new field - absent in old data

    #[serde(default, alias = "started")]
    pub created: Option<u64>,       // supports old field name
}
```

Key patterns:
- `#[serde(default)]` ensures missing fields deserialize cleanly
- `alias = "old_name"` reads older formats without rewriting them
- Never repurpose field meanings in-place—add new fields instead

## Sentinel Values

If spec uses sentinel strings (e.g., `Related: None`) but your type is `Option<String>`:

```rust
fn parse_optional_field(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() || t.eq_ignore_ascii_case("none") {
        None
    } else {
        Some(t.to_string())
    }
}
```

## UniFFI Boundaries

UniFFI works best with flat, stable types:

- `String` for IDs and timestamps (convert `Uuid`/`DateTime` at the boundary)
- Flat enums (no associated data) or string representations
- `Vec<T>` and `Option<T>` where `T` is FFI-friendly

```rust
#[derive(Clone, Debug, uniffi::Record)]
pub struct IdeaDto {
    pub id: String,
    pub created_at_ms: i64,
    pub status: String,        // "open" | "in_progress" | "done"
    pub priority: Option<String>,
}
```

**Enums vs Strings:** UniFFI supports enums via `#[derive(uniffi::Enum)]` for type safety in Kotlin/Swift. However, adding a new variant is a breaking change—foreign code won't recognize it until bindings regenerate. Using strings treats the field as an open set, more stable but requires foreign code to handle unknowns. Choose based on stability requirements.
