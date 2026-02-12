# Plan Document Template

Use this structure when writing `.claude/plan.md`. Adapt sections to the specific task.

```markdown
# Plan: [Feature/Change Name]

## Goal
[One sentence: what this change achieves for the user/system]

## Approach
[Detailed explanation of the implementation strategy — why this approach over alternatives]

## Changes

### [Component/Area 1]

**File:** `path/to/file.ts`

[Explanation of what changes and why]

```typescript
// Code snippet showing the actual change
```

### [Component/Area 2]

**File:** `path/to/other-file.ts`

[Explanation]

```typescript
// Code snippet
```

## Files Modified
- `path/to/file.ts` — [what changes]
- `path/to/other-file.ts` — [what changes]
- `path/to/new-file.ts` — [new, purpose]

## Trade-offs & Considerations
- [Trade-off 1: what was chosen and why]
- [Trade-off 2: what was deferred and why]

## Out of Scope
- [Thing explicitly not being done and why]

## Tasks
<!-- Added after annotation cycles are complete -->
```

## Guidelines

- Code snippets should show real code based on the actual codebase, not pseudocode
- Include file paths for every change
- Call out what is NOT changing — this prevents scope creep
- If the user provided reference implementations, explain how the approach adapts them
- End with: "Ready for your review. Add inline notes directly to this file and tell me when to address them."
