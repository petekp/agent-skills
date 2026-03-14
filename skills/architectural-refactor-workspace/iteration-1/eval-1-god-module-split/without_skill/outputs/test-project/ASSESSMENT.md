# Architectural Assessment: test-project

## Dependency Map

```
core.js (god module)
  ← api.js (imports 10+ functions)
  ← workers.js (imports 4 functions)
  ← tests/run.js (imports everything)
```

All roads lead to `core.js`. There are no other module-to-module dependencies.

## Critical Findings

### Finding 1: God Module (`src/core.js`)
**Severity:** Critical
**Pattern:** God Module

`core.js` exports 15 functions and 3 internal data structures spanning four unrelated domains: authentication, user management, data processing, and email notifications. Every other file in the project imports from it. A change to email logic risks breaking auth; a change to CSV parsing scrolls past password hashing.

### Finding 2: Cross-Domain Coupling in API Layer
**Severity:** High
**Pattern:** Feature Envy

`handleUploadCSV` in `api.js` reaches across three domains in a single function: it validates a session (auth), looks up a user (user management), processes CSV data (data processing), and queues a notification email (email). The function has no single responsibility.

### Finding 3: Exposed Internals
**Severity:** Medium
**Pattern:** Inappropriate Intimacy

`core.js` exports `_users`, `_sessions`, and `_emailQueue` — raw mutable state used directly by tests. This couples the test suite to internal data structures and makes it impossible to change the storage approach without rewriting tests.

## Recommended Architecture

Split `core.js` into four focused modules:

```
src/
  auth.js        — hashPassword, verifyPassword, createSession, validateSession
  users.js       — createUser, findUser, findUserByEmail, updateUserRole, listUsers
  data.js        — processCSV, aggregateByField, generateReport
  email.js       — queueEmail, processEmailQueue, getEmailHistory
```

Each module owns its own data (users array, sessions object, email queue). Cross-module calls go through public interfaces, not shared mutable state.

## Transformation Sequence

1. Extract `auth.js` from `core.js` — move auth functions and sessions state
2. Extract `email.js` from `core.js` — move email functions and queue state
3. Extract `data.js` from `core.js` — move CSV/report functions (no state)
4. Extract `users.js` from `core.js` — move remaining user functions and users array
5. Update `api.js` imports to use the new modules
6. Update `workers.js` imports to use the new modules
7. Update tests to use the new modules and stop relying on exported internals
8. Delete the now-empty `core.js`
