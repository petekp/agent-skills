# Refactoring Plan

**Source:** ASSESSMENT.md
**Created:** 2026-03-05
**Target:** Split the god module `src/core.js` into four focused domain modules (auth, users, data, email), update all consumers, and delete the original.

## Pre-flight Checks
- [x] All tests pass before starting (17/17)
- [ ] No uncommitted changes (N/A - not a real git repo)
- [ ] Working branch created (N/A - not a real git repo)

## Chunk 1: Extract auth.js from core.js
**Why:** Assessment Finding 1 (God Module) — auth functions and sessions state are unrelated to users, data, and email.
**Entry criteria:** All tests pass, no prior chunks pending
**Steps:**
1. Create `src/auth.js` with the `sessions` state object and the four auth functions: `hashPassword`, `verifyPassword`, `createSession`, `validateSession`
2. Export all four functions plus `_sessions` (for backward compatibility during migration)
3. In `src/core.js`, replace the auth function implementations with re-exports from `src/auth.js`. Remove the `sessions` variable. Keep exporting the same public API so consumers don't break yet.
**Exit criteria:** All tests pass (17/17), `node -e "require('./src/auth')"` succeeds
**Commit message:** `refactor: extract auth module from core.js`

## Chunk 2: Extract email.js from core.js
**Depends on:** Chunk 1
**Why:** Assessment Finding 1 (God Module) — email functions and queue state are an independent domain.
**Entry criteria:** Chunk 1 complete and verified
**Steps:**
1. Create `src/email.js` with the `emailQueue` array and three email functions: `queueEmail`, `processEmailQueue`, `getEmailHistory`
2. Export all three functions plus `_emailQueue` (for backward compatibility during migration)
3. In `src/core.js`, replace the email function implementations with re-exports from `src/email.js`. Remove the `emailQueue` variable. Keep the same public API.
**Exit criteria:** All tests pass (17/17), `node -e "require('./src/email')"` succeeds
**Commit message:** `refactor: extract email module from core.js`

## Chunk 3: Extract data.js from core.js
**Depends on:** Chunk 2
**Why:** Assessment Finding 1 (God Module) — data processing functions are stateless and completely independent.
**Entry criteria:** Chunk 2 complete and verified
**Steps:**
1. Create `src/data.js` with three data functions: `processCSV`, `aggregateByField`, `generateReport`
2. Export all three functions
3. In `src/core.js`, replace the data function implementations with re-exports from `src/data.js`. Keep the same public API.
**Exit criteria:** All tests pass (17/17), `node -e "require('./src/data')"` succeeds
**Commit message:** `refactor: extract data processing module from core.js`

## Chunk 4: Extract users.js from core.js
**Depends on:** Chunk 3
**Why:** Assessment Finding 1 (God Module) — user management is the last domain remaining in core.js.
**Entry criteria:** Chunk 3 complete and verified
**Steps:**
1. Create `src/users.js` with the `users` array and five user functions: `createUser`, `findUser`, `findUserByEmail`, `updateUserRole`, `listUsers`
2. `createUser` calls `hashPassword` (from auth) and `queueEmail` (from email), so `src/users.js` must require `src/auth.js` and `src/email.js`
3. Export all five functions plus `_users` (for backward compatibility during migration)
4. In `src/core.js`, replace the user function implementations with re-exports from `src/users.js`. Remove the `users` array. Keep the same public API.
**Exit criteria:** All tests pass (17/17), `node -e "require('./src/users')"` succeeds
**Commit message:** `refactor: extract user management module from core.js`

## Chunk 5: Update api.js imports to use new modules directly
**Depends on:** Chunk 4
**Why:** Assessment Finding 2 (Cross-Domain Coupling) — api.js should import from domain modules, not the god module.
**Entry criteria:** Chunk 4 complete and verified
**Steps:**
1. In `src/api.js`, replace `const core = require("./core")` with imports from the four new modules: `const auth = require("./auth")`, `const users = require("./users")`, `const data = require("./data")`, `const email = require("./email")`
2. Update `handleLogin`: `core.findUserByEmail` -> `users.findUserByEmail`, `core.verifyPassword` -> `auth.verifyPassword`, `core.createSession` -> `auth.createSession`
3. Update `handleRegister`: `core.createUser` -> `users.createUser`, `core.createSession` -> `auth.createSession`
4. Update `handleUploadCSV`: `core.validateSession` -> `auth.validateSession`, `core.processCSV` -> `data.processCSV`, `core.aggregateByField` -> `data.aggregateByField`, `core.generateReport` -> `data.generateReport`, `core.findUser` -> `users.findUser`, `core.queueEmail` -> `email.queueEmail`
5. Update `handleAdminListUsers`: `core.validateSession` -> `auth.validateSession`, `core.findUser` -> `users.findUser`, `core.listUsers` -> `users.listUsers`
**Exit criteria:** All tests pass (17/17), no references to `core` remain in `api.js`
**Commit message:** `refactor: update api.js to import from domain modules`

## Chunk 6: Update workers.js imports to use new modules directly
**Depends on:** Chunk 4
**Why:** Assessment Finding 2 — workers.js should import from domain modules, not the god module.
**Entry criteria:** Chunk 4 complete and verified
**Steps:**
1. In `src/workers.js`, replace `const core = require("./core")` with `const email = require("./email")` and `const users = require("./users")`
2. Update `runEmailWorker`: `core.processEmailQueue` -> `email.processEmailQueue`
3. Update `runUserCleanup`: `core.listUsers` -> `users.listUsers`, `core.queueEmail` -> `email.queueEmail`
**Exit criteria:** All tests pass (17/17), no references to `core` remain in `workers.js`
**Commit message:** `refactor: update workers.js to import from domain modules`

## Chunk 7: Update tests to use new modules and remove internal state access via core.js
**Depends on:** Chunk 5, Chunk 6
**Why:** Assessment Finding 3 (Exposed Internals) — tests should import from domain modules and use their own internal state references.
**Entry criteria:** Chunks 5 and 6 complete and verified
**Steps:**
1. In `tests/run.js`, replace `const core = require("../src/core")` with individual module imports: `const auth = require("../src/auth")`, `const users = require("../src/users")`, `const data = require("../src/data")`, `const email = require("../src/email")`
2. Update `resetState()` to use `users._users`, `auth._sessions`, `email._emailQueue` instead of `core._users`, `core._sessions`, `core._emailQueue`
3. Update auth tests: `core.hashPassword` -> `auth.hashPassword`, `core.verifyPassword` -> `auth.verifyPassword`, `core.createSession` -> `auth.createSession`, `core.validateSession` -> `auth.validateSession`
4. Update user tests: `core.createUser` -> `users.createUser`, `core.findUser` -> `users.findUser`, `core.findUserByEmail` -> `users.findUserByEmail`, `core.updateUserRole` -> `users.updateUserRole`, `core.listUsers` -> `users.listUsers`
5. Update data tests: `core.processCSV` -> `data.processCSV`, `core.aggregateByField` -> `data.aggregateByField`
6. Update email tests: `core.queueEmail` -> `email.queueEmail`, `core.processEmailQueue` -> `email.processEmailQueue`, `core._emailQueue` -> `email._emailQueue`, `core.getEmailHistory` -> `email.getEmailHistory`
**Exit criteria:** All tests pass (17/17), no references to `core` remain in `tests/run.js`
**Commit message:** `refactor: update tests to import from domain modules directly`

## Chunk 8: Delete core.js
**Depends on:** Chunk 7
**Why:** core.js is now an empty pass-through; removing it completes the god module split.
**Entry criteria:** Chunk 7 complete and verified, no file in the project imports from `./core`
**Steps:**
1. Verify no file references `require("./core")` or `require("../src/core")`
2. Delete `src/core.js`
**Exit criteria:** All tests pass (17/17), `src/core.js` does not exist
**Commit message:** `refactor: remove empty core.js god module`

## Post-flight Checks
- [x] Full test suite passes (17/17)
- [x] No TODO/FIXME markers left from refactoring
- [x] Assessment findings are resolved
