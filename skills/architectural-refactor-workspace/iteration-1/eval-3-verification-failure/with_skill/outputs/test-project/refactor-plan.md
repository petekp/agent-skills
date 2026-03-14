# Refactoring Plan

**Source:** ASSESSMENT.md
**Created:** 2026-03-05
**Target:** Split the god module `core.js` into four focused domain modules (auth, users, data, email) with clean boundaries and no shared mutable state.

## Pre-flight Checks
- [x] All tests pass before starting
- [ ] No uncommitted changes (N/A - not using git commits)
- [ ] Working branch created (N/A - not using git commits)

## Chunk 1: Extract `auth.js` from `core.js`
**Why:** Finding 1 (God Module) - auth functions and sessions state mixed with unrelated domains
**Entry criteria:** All tests pass, no prior chunks pending
**Steps:**
1. Create `src/auth.js` with the `sessions` state object and these functions from `core.js`: `hashPassword`, `verifyPassword`, `createSession`, `validateSession`
2. Export all four functions plus `_sessions` (for test compatibility) from `auth.js`
3. In `core.js`, remove the auth functions and sessions state; instead import them from `./auth` and re-export them (preserving the existing public API so downstream consumers don't break yet)
**Exit criteria:** All tests pass, typecheck clean, lint clean
**Commit message:** `refactor: extract auth module from core.js`

## Chunk 2: Extract `email.js` from `core.js`
**Depends on:** Chunk 1
**Why:** Finding 1 (God Module) - email functions and queue state mixed with unrelated domains
**Entry criteria:** Chunk 1 complete and verified
**Steps:**
1. Create `src/email.js` with the `emailQueue` array and these functions from `core.js`: `queueEmail`, `processEmailQueue`, `getEmailHistory`
2. Export all three functions plus `_emailQueue` (for test compatibility) from `email.js`
3. In `core.js`, remove the email functions and emailQueue state; instead import them from `./email` and re-export them
**Exit criteria:** All tests pass, typecheck clean, lint clean
**Commit message:** `refactor: extract email module from core.js`

## Chunk 3: Extract `data.js` from `core.js`
**Depends on:** Chunk 2
**Why:** Finding 1 (God Module) - data processing functions mixed with unrelated domains
**Entry criteria:** Chunk 2 complete and verified
**Steps:**
1. Create `src/data.js` with these functions from `core.js`: `processCSV`, `aggregateByField`, `generateReport`
2. Export all three functions from `data.js`
3. In `core.js`, remove the data functions; instead import them from `./data` and re-export them
**Exit criteria:** All tests pass, typecheck clean, lint clean
**Commit message:** `refactor: extract data processing module from core.js`

## Chunk 4: Extract `users.js` from `core.js`
**Depends on:** Chunk 3
**Why:** Finding 1 (God Module) - user management functions and users array mixed with other domains
**Entry criteria:** Chunk 3 complete and verified
**Steps:**
1. Create `src/users.js` with the `users` array and these functions: `createUser`, `findUser`, `findUserByEmail`, `updateUserRole`, `listUsers`
2. `createUser` calls `hashPassword` (from auth) and `queueEmail` (from email), so import those: `const { hashPassword } = require("./auth"); const { queueEmail } = require("./email");`
3. Export all five functions plus `_users` (for test compatibility) from `users.js`
4. In `core.js`, remove the user functions and users array; instead import them from `./users` and re-export them
**Exit criteria:** All tests pass, typecheck clean, lint clean
**Commit message:** `refactor: extract users module from core.js`

## Chunk 5: Update `api.js` imports to use new modules
**Depends on:** Chunk 4
**Why:** Finding 2 (Cross-Domain Coupling) - api.js should import from specific domain modules, not the god module
**Entry criteria:** Chunk 4 complete and verified
**Steps:**
1. In `api.js`, replace `const core = require("./core");` with imports from the four domain modules:
   - `const auth = require("./auth");`
   - `const users = require("./users");`
   - `const data = require("./data");`
   - `const email = require("./email");`
2. Update `handleLogin`: `core.findUserByEmail` -> `users.findUserByEmail`, `core.verifyPassword` -> `auth.verifyPassword`, `core.createSession` -> `auth.createSession`
3. Update `handleRegister`: `core.createUser` -> `users.createUser`, `core.createSession` -> `auth.createSession`
4. Update `handleUploadCSV`: `core.validateSession` -> `auth.validateSession`, `core.processCSV` -> `data.processCSV`, `core.aggregateByField` -> `data.aggregateByField`, `core.generateReport` -> `data.generateReport`, `core.findUser` -> `users.findUser`, `core.queueEmail` -> `email.queueEmail`
5. Update `handleAdminListUsers`: `core.validateSession` -> `auth.validateSession`, `core.findUser` -> `users.findUser`, `core.listUsers` -> `users.listUsers`
**Exit criteria:** All tests pass, typecheck clean, lint clean
**Commit message:** `refactor: update api.js to import from domain modules`

## Chunk 6: Update `workers.js` imports to use new modules
**Depends on:** Chunk 5
**Why:** Finding 1 - workers.js should import from specific domain modules
**Entry criteria:** Chunk 5 complete and verified
**Steps:**
1. In `workers.js`, replace `const core = require("./core");` with:
   - `const email = require("./email");`
   - `const users = require("./users");`
2. Update `runEmailWorker`: `core.processEmailQueue` -> `email.processEmailQueue`
3. Update `runUserCleanup`: `core.listUsers` -> `users.listUsers`, `core.queueEmail` -> `email.queueEmail`
**Exit criteria:** All tests pass, typecheck clean, lint clean
**Commit message:** `refactor: update workers.js to import from domain modules`

## Chunk 7: Update tests to import from new modules and delete `core.js`
**Depends on:** Chunk 6
**Why:** Finding 3 (Exposed Internals) - tests should import from domain modules; core.js should be removed
**Entry criteria:** Chunk 6 complete and verified
**Steps:**
1. In `tests/run.js`, replace `const core = require("../src/core");` with:
   - `const auth = require("../src/auth");`
   - `const users = require("../src/users");`
   - `const data = require("../src/data");`
   - `const email = require("../src/email");`
2. Update `resetState()` to use `auth._sessions`, `users._users`, `email._emailQueue`
3. Update all `core.hashPassword` -> `auth.hashPassword`, `core.verifyPassword` -> `auth.verifyPassword`, etc. mapping each call to its domain module
4. Delete `src/core.js`
**Exit criteria:** All tests pass, typecheck clean, lint clean
**Commit message:** `refactor: update tests to use domain modules, remove core.js`

## Post-flight Checks
- [ ] Full test suite passes
- [ ] No TODO/FIXME markers left from refactoring
- [ ] Assessment findings are resolved
