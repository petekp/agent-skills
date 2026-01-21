# Text & Parsing

Patterns for UTF-8 safety, path normalization, state machine parsing, and round-trip preservation.

## UTF-8 Safe Truncation

Indexing with byte offsets (`&s[..60]`) **panics** if the index isn't on a character boundary.

**By characters:**
```rust
fn truncate_chars(s: &str, max_chars: usize) -> String {
    let out: String = s.chars().take(max_chars).collect();
    if s.chars().count() > max_chars {
        format!("{out}...")
    } else {
        out
    }
}
```

**By grapheme clusters (emoji-safe):**
```rust
use unicode_segmentation::UnicodeSegmentation;

fn truncate_graphemes(s: &str, max_graphemes: usize) -> String {
    let graphemes: Vec<&str> = s.graphemes(true).collect();
    if graphemes.len() > max_graphemes {
        format!("{}...", graphemes[..max_graphemes].concat())
    } else {
        s.to_string()
    }
}
```

## Path Normalization

Use a single normalizer for all comparisons and hashing:

```rust
fn normalize_path(path: &str) -> String {
    let trimmed = path.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        trimmed.to_string()
    }
}

fn child_prefix(query: &str) -> String {
    let q = normalize_path(query);
    if q == "/" { "/".to_string() } else { format!("{}/", q) }
}
```

**Cross-platform:** On Windows, also normalize backslashes to forward slashes and handle drive letters. Consider `dunce` crate for canonicalization without `\\?\` prefix.

## Case Normalization

If spec says values are case-insensitive, normalize early:

```rust
fn norm_key(k: &str) -> String { k.trim().to_ascii_lowercase() }
fn norm_value(v: &str) -> String { v.trim().to_ascii_lowercase() }
```

## State Machine Parsing

For markdown-like formats, a state machine beats fragile regex.

**States:** `OutsideBlock`, `InBlockHeader`, `InMetadataBlock`, `InDescription`

**Rules:**
- Match block start only on the heading line (e.g., `### [#idea-<id>] <title>`)
- Parse metadata only within a contiguous region after the heading
- Treat everything else as content until delimiter (`---`) or next heading

## Anchored Updates

**Problem:** `line.contains("[#idea-123]")` can match references in descriptions, causing silent corruption.

**Solution:** Anchor on the heading line:

```rust
use once_cell::sync::Lazy;
use regex::Regex;

static IDEA_HEADING_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^### \[#idea-([0-9A-HJKMNP-TV-Z]{26})\]\s*(.*)$").unwrap()
});
```

Update logic:
1. Scan lines for heading that exactly matches target ID → `in_target_block = true`
2. While in block, update only the metadata key you care about
3. Stop when you hit delimiter (`---`) or new heading

## Round-Trip Preservation

Parse into an AST that preserves formatting:

```rust
pub struct ParsedFile {
    pub format_version: u32,
    pub sections: Vec<Section>,
    pub trailing_text: String,  // preserve unknown text/comments
}

pub struct Block {
    pub id: String,
    pub header_line: String,        // preserve original formatting
    pub fields: Vec<(String, String)>,
    pub body: String,
    pub separator: String,          // e.g., "\n---\n"
}
```

## Version Markers

If format spec mandates a version marker, be strict:

```rust
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Unsupported format. Expected {expected}, found {found:?}")]
    UnsupportedFormat { expected: String, found: Option<String> },

    #[error("Item not found: {0}")]
    NotFound(String),
}
```

## Duplicate ID Handling

External edits can create duplicate IDs. Pick a deterministic policy:

```rust
use std::collections::HashSet;

fn dedupe_by_id<T: HasId>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    items.into_iter()
        .filter(|item| seen.insert(item.id().to_string()))
        .collect()
}
```
