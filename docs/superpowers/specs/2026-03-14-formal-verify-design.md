# formal-verify — Formal Architectural Verification Skill

**Date:** 2026-03-14
**Status:** Design approved
**Target project:** ~/Code/capacitor (and any agent-driven codebase)

## Problem

Coding agents produce code that compiles and passes tests but violates architectural intent. Partial migrations, semantic boundary violations, and inelegant code survive because existing verification (grep ratchets, manual smoke matrices) operates at the syntax level. An agent can satisfy a grep ratchet by wrapping a violation in a helper function with a different name.

The core failure mode: an agent claims "migration complete" but leaves behind coupling violations, stale patterns, or mixed-architecture code because it can't hold the full codebase in context.

## Goal

A continuously-running formal verification skill that ensures all code — new and existing — is provably sound, architecturally disciplined, and elegant. No code lands unless verified. Any engineer reviewing the codebase should be impressed with how clean and disciplined it is.

## Architecture

Three-layer verification pipeline with tiered triggers.

```
Triggers                    Verification Engine                Output
─────────                   ────────────────────               ──────
PostToolUse hook ──→ Layer 1 only (~200ms)         ──→ Violation report
Slice checkpoint ──→ Layer 1 + 2 (~5-30s)              (tiered by audience)
Pre-commit gate  ──→ Layer 1 + 2 + 3 (full)       ──→ Pass verdict +
/verify command  ──→ Layer 1 + 2 + 3 (verbose)         elegance grade
```

### Trigger Model: Tiered Adaptive

- **Every edit (PostToolUse hook on Write/Edit):** Layer 1 structural checks only. Sub-second. Catches ownership violations, boundary crossings, import leaks immediately.
- **Slice checkpoints:** Layers 1 + 2. Structural + behavioral verification. Runs at logical work boundaries.
- **Pre-commit gate:** All three layers. Full audit including elegance grade. Commit blocked unless all layers pass and minimum elegance grade met.
- **Manual `/verify`:** All three layers with verbose output. For human review.

### Output: Tiered by Audience

- **Agent output:** Counterexample + diagnosis + specific fix suggestion. Maximally actionable for automated fix loops.
- **Human output:** Counterexample + diagnosis. Enough to understand and judge without being patronized.
- **Pass output:** Verdict + elegance grade (A/B/C/D/F). Every deduction traceable to a specific file and line.

### Escalation Policy

3 failed agent fix attempts → escalate to human. Escalation includes: original violation, all 3 attempted fixes, why each was insufficient, and the agent's hypothesis about what's blocking it.

## Layer 1: Structural Verifier

**Engine:** Z3Py constraints over tree-sitter AST facts
**Speed:** ~200ms
**Runs:** Every edit

### Fact Extraction

Tree-sitter parses Rust and Swift source files into ASTs. A fact extractor walks each AST and emits a relational fact database:

```
module("TmuxRouter", lang="swift", path="Models/")
calls("TerminalLauncher", "TmuxRouter.route")
imports("SessionCard", "CapacitorCore")
type_crosses_ffi("RoutingTarget")
contains_literal("foo.swift", "tmux")
defines_fn("TmuxRouter", "buildCommand")
has_pub_fn("capacitor_core", "derive_route")
```

Facts are extracted incrementally — only changed files are re-parsed. Fact database is cached between runs in `.verifier/facts/`.

### Constraint Engine

Facts are loaded into Z3 as assertions. Constraints from `.verifier/structural.yaml` become Z3 formulas. The solver checks satisfiability — SAT means a violation exists (with a concrete counterexample), UNSAT means all constraints hold.

### Constraint YAML Schema

```yaml
ownership:
  - rule: "exclusive_tmux_access"
    description: "Only TmuxRouter may build or execute tmux commands"
    constraint:
      only: "TmuxRouter"
      may: "reference_tmux_literal"

  - rule: "rust_owns_state_derivation"
    description: "Swift must not derive routing state"
    constraint:
      modules_in: "swift"
      must_not: "call_pattern:derive_*|reduce_*"
      except: ["RuntimeClient"]

boundaries:
  - rule: "uniffi_single_entry"
    description: "Only RuntimeClient crosses the FFI boundary"
    constraint:
      only: "RuntimeClient"
      may: "import:CapacitorCore"

  - rule: "no_cross_layer_imports"
    description: "Terminal drivers must not import from core/"
    constraint:
      modules_matching: "*TerminalDriver"
      must_not: "import_from:core/"

patterns:
  - rule: "driver_protocol_conformance"
    description: "All terminal drivers must implement TerminalDriver protocol"
    constraint:
      modules_matching: "*TerminalDriver"
      must: "implement:TerminalDriver"

migration:
  - rule: "no_legacy_activation"
    description: "No references to deprecated parallel activation paths"
    constraint:
      all_modules: true
      must_not: "reference:target_kind|target_value|parallel_activation"
```

### Constraint Categories

- **Ownership:** "Only X may do Y," "X must not do Y," "X owns all Z operations"
- **Boundaries:** "Only X may cross boundary Y," "Modules in A must not import from B"
- **Pattern conformance:** "All X must implement Y," "Every public type must have matching test"
- **Migration completeness:** "No references to deprecated pattern X," "Legacy shim Z must have zero callers"

### Why Z3 Over Grep Ratchets

Grep checks string presence. Z3 reasons over the call graph. An agent could satisfy a grep ratchet by wrapping a tmux call in a helper with a different name. Z3 catches this because it checks reachability — "does any path from module X lead to behavior Y, even transitively through helper functions?"

Z3 also provides unsat cores when constraints conflict, which immediately identifies which rules are in tension — useful when the architecture is evolving and rules need updating.

## Layer 2: Behavioral Verifier

**Engine:** TLA+/Apalache for state machines, Z3Py for protocol constraints
**Speed:** ~5-30s
**Runs:** Slice checkpoints and pre-commit

### TLA+ / Apalache (State Machines + Temporal Properties)

For properties involving sequences of states, liveness, concurrent interleavings, and state machine correctness. Specs live in `.verifier/specs/*.tla`.

Example — Activation Coordinator:

```tla
---- MODULE ActivationCoordinator ----
VARIABLES state, pending, active

Init ==
  /\ state = "idle"
  /\ pending = {}
  /\ active = NULL

Activate(session) ==
  /\ state' = "activating"
  /\ pending' = pending \ {session}
  /\ active' = session

\* INVARIANT: never process stale request
NoStaleActivation ==
  active /= NULL =>
    \A s \in pending : s.timestamp < active.timestamp

\* INVARIANT: no double-activation
NoDoubleActivation ==
  Cardinality({s \in Sessions : s.state = "activating"}) <= 1
====
```

Apalache performs bounded symbolic model checking — translates TLA+ to SMT constraints and uses Z3 as the backend. Faster than TLC for large state spaces, and finds bugs TLC misses in symbolic exploration.

### Z3Py (Protocol Constraints + Cross-Boundary Contracts)

For properties about data relationships, input/output contracts, cross-boundary type compatibility, and arithmetic constraints. Specs live in `.verifier/specs/*.py`.

Example — Snapshot Immutability Contract:

```python
# Rust produces snapshot → crosses FFI → Swift must never mutate fields
snapshot = Const('snapshot', SnapshotSort)
swift_read = Function('swift_read', SnapshotSort, FieldSort, ValueSort)
rust_wrote = Function('rust_wrote', SnapshotSort, FieldSort, ValueSort)

# For all fields, Swift reads must equal what Rust wrote
s.add(ForAll([f], swift_read(snapshot, f) == rust_wrote(snapshot, f)))

# Check if any Swift code path mutates
# unsat = safe, sat = violation with concrete field + value as counterexample
```

### When TLA+ vs Z3Py

| Use TLA+ / Apalache | Use Z3Py |
|---|---|
| Property involves sequences of states | Property is about data relationships |
| Liveness ("eventually X happens") | Contract: input → output invariants |
| Concurrent interleavings | Cross-boundary type compatibility |
| State machine correctness | Arithmetic / bounded constraints |

### Capacitor-Specific Behavioral Specs

- **Activation flow correctness:** Coordinator never activates stale session, never double-activates, always resolves (TLA+)
- **Snapshot immutability:** Data from Rust is never mutated or reinterpreted by Swift (Z3Py)
- **State reducer validity:** Reducer never produces invalid state from valid input (Z3Py)
- **Terminal driver mutual exclusion:** No two drivers act on the same terminal simultaneously (TLA+)

## Layer 3: Elegance Auditor

**Engine:** Complexity metrics + pattern analysis + code smell detection
**Speed:** ~1-3s
**Runs:** Pre-commit and manual `/verify`

### Metrics

**Complexity:**
- Cyclomatic complexity per function (threshold configurable)
- Nesting depth limits
- Function and file length limits
- Parameter count limits

**Consistency:**
- Naming convention conformance
- Pattern usage consistency across similar modules
- Error handling style uniformity
- Comment/documentation density

**Craft:**
- Unnecessary abstraction detection
- Duplicated logic identification
- Over-engineering signals (premature generalization, unused configurability)
- Dead code and unused code paths

### Elegance Grade

Each verification run produces a letter grade (A/B/C/D/F) as a composite of all metrics. Every deduction is traceable to a specific file and line. Configurable thresholds in `.verifier/elegance.yaml`. Pre-commit gate requires minimum grade (default: B).

## Bootstrap Process

One-time setup, front-loads investment for autonomous operation afterward.

### Phase 1: Install (~5 min, automated)

- Install Z3Py and tree-sitter grammars (Rust, Swift)
- Install Apalache (JVM)
- Create `.verifier/` directory structure in target project
- Configure Claude Code hooks (PostToolUse, pre-commit)

### Phase 2: Discover (~10 min, agent-driven)

- Scan CLAUDE.md, ARCHITECTURE.md, design docs, audit history
- Analyze codebase structure, import graph, and existing patterns
- Propose initial set of structural constraints
- Identify state machines and protocols needing behavioral specs

### Phase 3: Interview (~20-30 min, collaborative)

- Walk through proposed constraints with the user
- Ask about ambiguities and missing rules in natural language
- Translate user's intent into formal specs (YAML + TLA+ + Z3Py)
- User reviews each rule in plain English, never raw TLA+

### Phase 4: Validate (~5 min, automated)

- Run full verification against current codebase
- Surface any existing violations as baseline
- Establish baseline elegance grade
- Confirm all specs are satisfiable (no contradictory constraints)

## Spec Evolution

### Drift Detection

When CLAUDE.md or ARCHITECTURE.md changes, the skill diffs new rules against existing constraints. New uncovered rules → auto-generates candidate constraints queued for review. Removed rules → flags potentially obsolete constraints.

### Re-Interview on Conflict

If a code change intentionally violates an existing constraint (e.g., during a migration), the skill asks: "This violates [rule X]. Is this intentional? Should we update the constraint?" Single question, not a blocking gate.

## Skill File Structure

```
skills/formal-verify/
├── SKILL.md                        # Main skill — orchestration + bootstrap
├── references/
│   ├── layer1-structural.md        # Fact extraction + Z3 constraint encoding
│   ├── layer2-behavioral.md        # TLA+/Apalache + Z3Py protocol specs
│   ├── layer3-elegance.md          # Complexity metrics + code smell rules
│   ├── constraint-yaml-spec.md     # Full YAML schema for structural rules
│   ├── bootstrap-process.md        # Install/discover/interview/validate
│   ├── agent-feedback-loop.md      # Hook integration + escalation policy
│   └── spec-authoring-guide.md     # How to translate intent → formal specs
├── scripts/
│   ├── install-deps.sh             # Z3, tree-sitter, Apalache install
│   ├── extract-facts.py            # Tree-sitter → fact database
│   ├── verify-structural.py        # Layer 1 engine
│   ├── verify-behavioral.py        # Layer 2 engine (Z3Py specs)
│   ├── run-apalache.sh             # Layer 2 engine (TLA+ specs)
│   ├── audit-elegance.py           # Layer 3 engine
│   └── verify.sh                   # Unified runner (all layers)
└── examples/
    ├── structural-rules.yaml       # Example constraint file
    ├── activation-coordinator.tla  # Example TLA+ spec
    └── snapshot-contract.py        # Example Z3Py spec
```

### In the Target Project

```
.verifier/
├── structural.yaml          # Project-specific structural constraints
├── elegance.yaml            # Elegance thresholds + overrides
├── specs/
│   ├── activation.tla       # Activation coordinator spec
│   ├── snapshot-contract.py # Cross-boundary snapshot spec
│   ├── reducer.py           # State reducer invariants
│   └── terminal-safety.tla  # Terminal driver mutual exclusion
├── facts/                   # Cached fact database (gitignored)
└── reports/                 # Verification reports (gitignored)
```

## Invocation

### Automatic (hooks)

- **PostToolUse (Write/Edit):** Layer 1 structural checks
- **Pre-commit:** Layers 1 + 2 + 3 full audit
- Agent receives violations injected into context transparently

### Manual

- `/verify` — Full audit, verbose output
- `/verify --bootstrap` — Run bootstrap process
- `/verify --evolve` — Check for spec drift
- `/verify --grade` — Elegance grade only

## Implementation Notes

**Layer 3 engine:** The elegance auditor will use a combination of existing tools (`radon`/`lizard` for Python-side complexity metrics, `swiftlint` for Swift conventions) and custom analysis for cross-language consistency checks and craft-level heuristics (unnecessary abstraction, over-engineering signals). Specific tooling to be finalized during planning.

**Fact extraction invalidation:** Changed files detected via `git diff --name-only` against the cached fact database's commit hash. On cache miss or renamed files, full re-extraction for affected modules. Fast path: mtime comparison for mid-session incremental checks.

**Apalache latency:** Apalache should run in daemon mode (`apalache-mc server`) to avoid JVM cold-start on every invocation. The 5-30s budget assumes a warm daemon. The bootstrap install phase starts the daemon and configures it to auto-launch.

**Pre-commit UX:** Layer 1 and Layer 2 results from the most recent slice checkpoint are cached. Pre-commit re-runs only if files changed since the last checkpoint. Full cold pre-commit (no cache) shows a progress indicator. Typical cached pre-commit: <5s.

## Dependencies

- **Python 3.10+** (Z3Py, tree-sitter bindings)
- **z3-solver** (pip package)
- **tree-sitter** + **tree-sitter-rust** + **tree-sitter-swift** (pip packages)
- **Apalache** (JVM, for TLA+ bounded model checking)
- **Java 17+** (Apalache runtime)

## Design Decisions

1. **Hybrid Z3Py + TLA+/Apalache** over pure Z3Py: Activation coordinator and terminal driver safety need temporal logic that Z3Py can't express without painful manual encoding.

2. **Tiered adaptive triggers** over per-edit full verification: Behavioral checks (5-30s) would grind the agent to a halt if run on every edit. Structural checks are fast enough for every edit.

3. **Agent-assisted spec authoring** over manual TLA+: The user shouldn't need to learn TLA+ to get formal verification. The skill translates natural language intent to formal specs.

4. **Elegance auditor as Layer 3** over correctness-only verification: The goal is code that impresses engineers, not just code that's correct. Formal verification catches bugs; the elegance auditor catches craft-level issues.

5. **3-attempt escalation** over infinite retry: Prevents the agent from churning endlessly on violations it can't resolve, while giving it enough attempts to fix straightforward issues.

6. **Cross-boundary verification** over per-language isolation: The most dangerous bugs in a Rust+Swift codebase live at the UniFFI boundary. Verifying each side independently would miss contract violations.

7. **Declarative YAML for structural rules** over code-level constraints: YAML is readable, diffable, and maintainable. The Z3 encoding is an implementation detail the user never sees.

8. **Spec evolution via drift detection** over manual maintenance: As CLAUDE.md and ARCHITECTURE.md evolve, constraints must keep pace automatically or they become stale and misleading.
