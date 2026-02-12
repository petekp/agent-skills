# Research Document Template

Use this structure when writing `.claude/research.md`. Adapt sections to the specific investigation.

```markdown
# Research: [Area/Feature/System Name]

## Overview
[What this part of the system does, in 2-3 sentences]

## Architecture
[How the components fit together — data flow, key abstractions, entry points]

## Key Files
| File | Purpose |
|------|---------|
| `path/to/file.ts` | [What it does] |

## Core Logic
[Detailed walkthrough of the main flows — not just what functions exist, but how data moves through them, what edge cases are handled, what conventions are followed]

## Patterns & Conventions
[Existing patterns the codebase uses — naming, error handling, state management, caching, etc. New code must follow these.]

## Integration Points
[How this area connects to other parts of the system — shared utilities, event systems, APIs, database access patterns]

## Potential Issues
[Bugs found, fragile areas, missing error handling, race conditions, technical debt]

## Key Observations
[Anything surprising, non-obvious, or critical for planning — implicit assumptions, undocumented behavior, performance characteristics]
```

## Guidelines

- Include actual code snippets for complex logic, not just descriptions
- Note what ORM, framework conventions, or internal utilities exist — new code must use them
- Flag anything that could break if modified without care
- Be specific about file paths and line numbers
