---
name: explanatory-playground
description: Build interactive debugging interfaces that reveal internal system behavior. Use when asked to "help me understand how this works", "show me what's happening", "visualize the state", "build a debug view", "I can't see what's going on", or any request to make opaque system behavior visible. Applies to state machines, data flow, event systems, algorithms, render cycles, animations, CSS calculations, or any mechanism with hidden internals.
---

# Explanatory Playground

Build dev-only visualizations that make invisible system behavior visible.

## Workflow

### 1. Identify what's hidden

Ask: What can't the user see?
- **State** — Values that change over time
- **Transitions** — Events that trigger changes
- **Relationships** — How parts communicate
- **Logic** — Conditions, thresholds, rules

### 2. Pick visualization approach

| System | Visualization | Library |
|--------|--------------|---------|
| State machines | Node-edge graph | react-flow |
| Data flow | Directed graph / Sankey | react-flow |
| Events | Timeline | custom or recharts |
| Algorithms | Step animation | custom |
| Render cycles | Component tree + diffs | custom |
| Animations | Timeline scrubber | custom |
| CSS/Layout | Box model overlay | custom |

See [references/patterns.md](references/patterns.md) for layouts, code, and implementation details.

### 3. Choose interactivity level

| Level | Features | When |
|-------|----------|------|
| 1 - Observe | Real-time state display | Always |
| 2 - Inspect | Click/hover for details | Usually |
| 3 - Manipulate | Trigger events, modify state | Edge cases |
| 4 - Time travel | History scrubbing, replay | Race conditions |

Start with 1-2. Add 3-4 when needed.

### 4. Instrument minimally

**Prefer event emitters** (least invasive):
```typescript
const debugEmitter = new EventEmitter();
function transition(from, to, event) {
  debugEmitter.emit('transition', { from, to, event, timestamp: Date.now() });
  // existing logic...
}
```

**Use proxies** for third-party code:
```typescript
function observable<T extends object>(obj: T) {
  return new Proxy(obj, {
    set(target, prop, value) {
      window.dispatchEvent(new CustomEvent('state:change', {
        detail: { prop, old: target[prop], new: value }
      }));
      return Reflect.set(target, prop, value);
    }
  });
}
```

### 5. Create dev-only route

```
app/__dev/[system-name]/page.tsx
```

Guard against production:
```typescript
if (process.env.NODE_ENV !== 'development') {
  return notFound();
}
```

### 6. Document removal

Header in every created file:
```typescript
/**
 * EXPLANATORY-PLAYGROUND DEBUG TOOL
 * Remove when done:
 * 1. Delete: app/__dev/[name]/page.tsx
 * 2. Delete: src/lib/[system]-debug.ts
 * 3. Remove hooks from: src/lib/[system].ts (lines XX-YY)
 * Purpose: [what this debugs]
 */
```

## Cleanup

On removal request:
1. Delete `__dev/` route
2. Remove instrumentation (emitters, proxies)
3. Uninstall added deps if unused elsewhere
4. Search for `EXPLANATORY-PLAYGROUND` markers

Report what was removed.
