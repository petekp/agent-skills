# Refactoring Plan

**Source:** ASSESSMENT.md
**Created:** 2026-03-04
**Target:** Split core.js god module into focused domain modules

## Pre-flight Checks
- [x] All tests pass before starting
- [x] No uncommitted changes
- [x] Working branch created

## Chunk 1: Extract auth module
**Why:** Assessment Finding 1 — God Module
**Entry criteria:** All tests pass, no prior chunks pending
**Steps:**
1. Create `src/auth.js` with session storage and auth functions (hashPassword, verifyPassword, createSession, validateSession)
2. Remove auth functions and sessions object from `src/core.js`
3. Update `src/core.js` to import from `src/auth.js` and re-export auth functions for backwards compatibility
4. Verify all tests pass
**Exit criteria:** All tests pass, auth logic isolated in own module
**Commit message:** `refactor: extract auth module from core`

## Chunk 2: Extract email module
**Why:** Assessment Finding 1 — God Module
**Entry criteria:** Chunk 1 complete and verified
**Steps:**
1. Create `src/email.js` with email queue and functions (queueEmail, processEmailQueue, getEmailHistory)
2. Remove email functions and emailQueue from `src/core.js`
3. Update `src/core.js` to import from `src/email.js` and re-export
4. Verify all tests pass
**Exit criteria:** All tests pass, email logic isolated
**Commit message:** `refactor: extract email module from core`

## Chunk 3: Extract data processing module
**Why:** Assessment Finding 1 — God Module
**Entry criteria:** Chunk 2 complete and verified
**Steps:**
1. Create `src/data.js` with processCSV, aggregateByField, generateReport
2. Remove data functions from `src/core.js`
3. Update `src/core.js` to import and re-export
4. Verify all tests pass
**Exit criteria:** All tests pass, data processing isolated
**Commit message:** `refactor: extract data processing module from core`

## Chunk 4: Extract users module and update consumers
**Why:** Assessment Finding 1 + Finding 3 — complete the split
**Entry criteria:** Chunk 3 complete and verified
**Steps:**
1. Create `src/users.js` with users array and user functions
2. Update `src/api.js` to import from individual modules instead of core
3. Update `src/workers.js` to import from individual modules
4. Update tests to import from individual modules
5. Delete `src/core.js`
6. Verify all tests pass
**Exit criteria:** All tests pass, core.js is gone, all imports use focused modules
**Commit message:** `refactor: extract users module, update all consumers, remove core.js`
