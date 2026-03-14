# formal-verify Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a three-layer formal architectural verification skill that continuously checks agent-written code for structural violations, behavioral bugs, and elegance issues.

**Architecture:** The skill is a Claude Code plugin with a main SKILL.md orchestrating three verification layers: Layer 1 (Z3Py over tree-sitter AST facts for structural rules), Layer 2 (TLA+/Apalache + Z3Py for behavioral specs), and Layer 3 (complexity metrics + pattern analysis for elegance). Python scripts do the actual verification. Reference docs teach the agent how each layer works and how to author specs.

**Tech Stack:** Python 3.10+, z3-solver, tree-sitter (Rust + Swift grammars), Apalache (JVM), Bash scripts, YAML for constraint configuration.

---

## File Structure

```
skills/formal-verify/
├── SKILL.md                           # Main skill: orchestration, bootstrap flow, /verify command
├── references/
│   ├── layer1-structural.md           # How Layer 1 works: fact extraction, Z3 encoding, constraint YAML
│   ├── layer2-behavioral.md           # How Layer 2 works: TLA+/Apalache, Z3Py protocol specs
│   ├── layer3-elegance.md             # How Layer 3 works: metrics, grading, thresholds
│   ├── constraint-yaml-spec.md        # Full YAML schema reference for structural.yaml
│   ├── bootstrap-process.md           # 4-phase bootstrap: install, discover, interview, validate
│   ├── agent-feedback-loop.md         # Hook integration, escalation policy, output format
│   └── spec-authoring-guide.md        # Translating natural language intent to formal specs
├── scripts/
│   ├── install-deps.sh                # Installs Z3, tree-sitter grammars, Apalache
│   ├── extract-facts.py               # Tree-sitter to relational fact database (JSON)
│   ├── verify-structural.py           # Layer 1 engine: loads facts + YAML, runs Z3 solve
│   ├── verify-behavioral.py           # Layer 2 engine: runs Z3Py spec files
│   ├── run-apalache.sh                # Layer 2 engine: runs TLA+ specs via Apalache
│   ├── audit-elegance.py              # Layer 3 entry point: delegates to elegance/ sub-modules
│   ├── elegance/
│   │   ├── __init__.py                # Shared types (Deduction, GradeResult)
│   │   ├── complexity.py              # Cyclomatic complexity, nesting, length, params
│   │   ├── consistency.py             # Naming, error handling, style uniformity
│   │   └── craft.py                   # Duplication, unnecessary abstraction, dead code
│   └── verify.sh                      # Unified entry point: runs selected layers, formats output
└── examples/
    ├── structural-rules.yaml          # Example constraint file (Capacitor-flavored)
    ├── activation-coordinator.tla     # Example TLA+ spec
    └── snapshot-contract.py           # Example Z3Py protocol spec
```

Each file has a single clear responsibility. Scripts are independent executables that can be tested in isolation. Reference docs are progressive disclosure — SKILL.md stays under 500 lines by delegating detail to references.

---

## Chunk 1: Skill Skeleton + Layer 1 Scripts

### Task 1: Create SKILL.md

**Files:**
- Create: `skills/formal-verify/SKILL.md`

- [ ] **Step 1: Write SKILL.md with frontmatter and orchestration logic**

```yaml
---
name: formal-verify
description: >
  Continuous formal verification of architectural constraints and code quality.
  Use when asked to verify, audit, or validate codebase integrity. Runs
  automatically via hooks on every edit (structural) and pre-commit (full).
  Catches ownership violations, boundary crossings, state machine bugs,
  and code smells that grep ratchets miss. Triggers: "verify", "formal verify",
  "check architecture", "audit code quality", "run verification",
  "/verify", "/verify --bootstrap", "/verify --grade".
license: MIT
metadata:
  author: petekp
  version: "0.1.0"
---
```

The body of SKILL.md should cover:

1. **Overview** — What this skill does: three-layer verification (structural, behavioral, elegance) with tiered triggers.

2. **Quick start** — How to bootstrap: `/verify --bootstrap` runs install, discover, interview, validate.

3. **How verification runs** — Trigger model: PostToolUse hook runs Layer 1 only. Slice checkpoint runs Layers 1+2. Pre-commit runs all three. `/verify` runs all three verbose.

4. **When violations are found** — Output format: agent gets counterexample + diagnosis + fix suggestion. Human gets counterexample + diagnosis. 3-attempt escalation policy.

5. **Commands** — `/verify` (full audit), `/verify --bootstrap` (setup), `/verify --evolve` (drift check), `/verify --grade` (elegance only).

6. **Project structure created** — What `.verifier/` looks like in the target project.

7. **Reference pointers** — Brief description of each reference doc with `@references/filename.md` pointers for progressive disclosure.

Keep under 500 lines. All implementation detail belongs in references.

- [ ] **Step 2: Verify SKILL.md is under 500 lines and has valid frontmatter**

Run: `wc -l skills/formal-verify/SKILL.md && head -10 skills/formal-verify/SKILL.md`
Expected: Line count under 500, frontmatter starts with `---`

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/SKILL.md
git commit -m "feat(formal-verify): add main SKILL.md with orchestration logic"
```

### Task 2: Create install-deps.sh

**Files:**
- Create: `skills/formal-verify/scripts/install-deps.sh`

- [ ] **Step 1: Write the install script**

The script should:
- Check for Python 3.10+ and install/warn if missing
- `pip install z3-solver tree-sitter tree-sitter-rust tree-sitter-swift radon lizard`
- Check for `swiftlint` and install via Homebrew if missing (used by Layer 3 elegance auditor for Swift convention checks)
- Check for Java 17+ (needed for Apalache)
- Download and install Apalache release binary to `~/.local/bin/apalache-mc` (or detect if already installed)
- Create `.verifier/` directory structure in the current project: `structural.yaml`, `elegance.yaml`, `specs/`, `facts/`, `reports/`
- Add `.verifier/facts/` and `.verifier/reports/` to `.gitignore` if not already present
- Start Apalache daemon (`apalache-mc server`) if behavioral verification will be used
- Print summary of what was installed and what is ready

Use `set -euo pipefail`. Check each dependency before installing. Be idempotent — safe to run multiple times.

- [ ] **Step 2: Make executable and test dry-run**

Run: `chmod +x skills/formal-verify/scripts/install-deps.sh && bash -n skills/formal-verify/scripts/install-deps.sh`
Expected: No syntax errors

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/scripts/install-deps.sh
git commit -m "feat(formal-verify): add dependency installation script"
```

### Task 3: Create extract-facts.py

**Files:**
- Create: `skills/formal-verify/scripts/extract-facts.py`

- [ ] **Step 1: Write the fact extractor**

The script should:
- Accept arguments: `--project-dir <path>` (required), `--files <file1> <file2>...` (optional, for incremental), `--output <path>` (default: `.verifier/facts/facts.json`)
- For each `.rs` file: parse with tree-sitter-rust, extract:
  - `module(name, lang="rust", path)` — from file path
  - `defines_fn(module, fn_name, visibility)` — function definitions
  - `calls(caller_module, callee)` — function call expressions
  - `imports(module, imported)` — use statements
  - `has_pub_fn(module, fn_name)` — public functions
  - `contains_literal(file, literal)` — string literals matching configurable patterns
  - `type_def(module, type_name, visibility)` — struct/enum definitions
- For each `.swift` file: parse with tree-sitter-swift, extract:
  - Same fact types adapted for Swift syntax
  - `imports(module, imported)` — import statements
  - `implements(module, protocol)` — protocol conformance
  - `type_crosses_ffi(type_name)` — types that appear in both Rust and Swift
- Output as JSON: `{"facts": [...], "metadata": {"commit": "abc123", "timestamp": "...", "files_parsed": 42}}`
- Incremental mode: if `--files` provided, only parse those files and merge with existing facts.json (removing stale facts for those files)
- If `--files` not provided, do full extraction
- Changed-file detection: when `--incremental` flag is passed (no explicit `--files`), compute changed files via `git diff --name-only` against the commit hash stored in facts.json metadata. Fall back to mtime comparison if git is unavailable. This logic lives in extract-facts.py, not verify.sh — the extractor owns its own cache invalidation.

Use `tree_sitter` Python bindings. Each language gets its own extractor function. Facts are flat tuples — no nesting.

- [ ] **Step 2: Test with a synthetic file**

Create a temporary Rust file and Swift file, run the extractor, verify the output JSON has the expected facts.

Run: `python3 skills/formal-verify/scripts/extract-facts.py --project-dir /tmp/test-extract --output /tmp/test-facts.json`
Expected: Valid JSON with facts array

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/scripts/extract-facts.py
git commit -m "feat(formal-verify): add tree-sitter fact extractor for Rust and Swift"
```

### Task 4: Create verify-structural.py

**Files:**
- Create: `skills/formal-verify/scripts/verify-structural.py`

- [ ] **Step 1: Write the structural verifier**

The script should:
- Accept arguments: `--facts <path>` (default: `.verifier/facts/facts.json`), `--constraints <path>` (default: `.verifier/structural.yaml`), `--output-mode <agent|human>` (default: human), `--json` (machine-readable output)
- Load facts from JSON
- Parse constraint YAML (see constraint-yaml-spec.md for schema)
- For each constraint rule:
  - Translate to Z3 formula over the fact database
  - `only: X may: Y` means assert no module other than X has fact Y. If SAT, the model gives the violating module.
  - `modules_in: X must_not: Y except: [Z]` means assert no module in set X (excluding Z) has fact Y.
  - `modules_matching: pattern must: Y` means assert all modules matching glob pattern have fact Y.
  - `all_modules: true must_not: Y` means assert no module has fact Y.
- Collect all violations with:
  - Rule name and description
  - Counterexample: which module, which file, which line (from facts)
  - Diagnosis: why this violates the rule
  - Fix suggestion (agent mode only): specific action to take
- Output: JSON array of violations (empty = pass), or formatted text for human mode
- Exit code: 0 = pass, 1 = violations found

The Z3 encoding strategy:
- Each module is a Z3 `Const` of sort `ModuleSort`
- Each fact type is a Z3 `Function` (e.g., `calls: Module x Module -> Bool`)
- Constraints become quantified formulas
- SAT = violation exists, model gives the counterexample
- For transitive reachability (call graph): use bounded unrolling to depth N (configurable, default 10). This is simpler than Z3 fixed-point and sufficient for typical call graphs. Depth 10 means "can module A reach behavior Y through at most 10 intermediate calls?"

- [ ] **Step 2: Write a test with known violations**

Create a test facts.json with a known violation (e.g., module "Foo" references tmux literal but is not TmuxRouter). Create a test structural.yaml with the ownership rule. Run verifier and confirm it catches the violation.

Run: `python3 skills/formal-verify/scripts/verify-structural.py --facts /tmp/test-violation-facts.json --constraints /tmp/test-constraints.yaml`
Expected: Exit code 1, violation report naming "Foo" as the violating module

- [ ] **Step 3: Test with no violations**

Create clean facts + constraints. Run verifier.

Run: Same command with clean inputs
Expected: Exit code 0, no violations

- [ ] **Step 4: Commit**

```bash
git add skills/formal-verify/scripts/verify-structural.py
git commit -m "feat(formal-verify): add Z3-based structural constraint verifier"
```

### Task 5: Create verify.sh unified runner

**Files:**
- Create: `skills/formal-verify/scripts/verify.sh`

- [ ] **Step 1: Write the unified runner**

The script should:
- Accept arguments: `--layers <1|2|3|all>` (default: all), `--project-dir <path>` (default: cwd), `--output-mode <agent|human>` (default: human), `--verbose`, `--json`
- Determine which layers to run based on `--layers`
- Guard for missing scripts: before running each layer, check if the script exists. If not, emit a warning ("Layer N script not found, skipping") and continue. This allows verify.sh to work from Chunk 1 before Layer 2/3 scripts are created in Chunk 2.
- Run fact extraction first (incremental if facts cache exists, using `--incremental` flag)
- Run Layer 1: `verify-structural.py`
- Run Layer 2 (if requested and scripts exist): `verify-behavioral.py` for Z3Py specs + `run-apalache.sh` for TLA+ specs
- Run Layer 3 (if requested and script exists): `audit-elegance.py`
- Cache management: write results to `.verifier/reports/last-run.json` with timestamp and list of files verified. On subsequent runs with `--since-checkpoint`, skip layers whose cached results are still valid (no files changed since last run). Pre-commit uses `--since-checkpoint` by default.
- Progress indicator: when running all three layers, print layer name before each ("Verifying structural constraints...", "Checking behavioral specs...", "Auditing elegance...")
- Aggregate results from all layers
- Print unified report: violations grouped by layer, elegance grade (if Layer 3 ran)
- Exit code: 0 = all passed, 1 = any violations

Use `set -euo pipefail`. Each layer runs independently — a Layer 1 failure does not skip Layer 2.

- [ ] **Step 2: Make executable and syntax check**

Run: `chmod +x skills/formal-verify/scripts/verify.sh && bash -n skills/formal-verify/scripts/verify.sh`
Expected: No syntax errors

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/scripts/verify.sh
git commit -m "feat(formal-verify): add unified verification runner"
```

### Task 6: Register in marketplace.json

**Files:**
- Modify: `.claude-plugin/marketplace.json`

- [ ] **Step 1: Add formal-verify plugin entry**

Add to the `plugins` array:

```json
{
  "name": "formal-verify",
  "description": "Continuous formal verification of architectural constraints, behavioral invariants, and code elegance for agent-driven codebases",
  "source": "./",
  "strict": false,
  "skills": ["./skills/formal-verify"]
}
```

- [ ] **Step 2: Verify JSON is valid**

Run: `python3 -m json.tool .claude-plugin/marketplace.json > /dev/null`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add .claude-plugin/marketplace.json
git commit -m "feat(formal-verify): register skill in marketplace"
```

---

## Chunk 2: Layer 2 + Layer 3 Scripts

### Task 7: Create verify-behavioral.py

**Files:**
- Create: `skills/formal-verify/scripts/verify-behavioral.py`

- [ ] **Step 1: Write the behavioral verifier for Z3Py specs**

The script should:
- Accept arguments: `--specs-dir <path>` (default: `.verifier/specs/`), `--output-mode <agent|human>`, `--json`
- Find all `*.py` files in specs-dir
- For each spec file:
  - Load using `importlib.util.spec_from_file_location` + `module_from_spec` (isolated import, no sys.path pollution)
  - Each spec file defines a `verify()` function that returns a list of `Violation` objects (or empty list)
  - The `verify()` function uses z3 internally — it creates a solver, adds assertions, checks satisfiability
  - If SAT: violation found, extract model as counterexample
  - If UNSAT: property holds
- Collect all violations across all spec files
- Output: same format as verify-structural.py (JSON or formatted text)
- Exit code: 0 = all specs pass, 1 = any violations

Spec file contract (document in spec-authoring-guide.md):
```python
"""Spec name: Snapshot Immutability Contract"""
from z3 import *

def verify(facts):
    """
    Args:
        facts: dict loaded from facts.json
    Returns:
        list of Violation(rule, counterexample, diagnosis, fix_suggestion)
    """
    s = Solver()
    # ... Z3 assertions ...
    if s.check() == sat:
        model = s.model()
        return [Violation(...)]
    return []
```

- [ ] **Step 2: Test with a passing spec**

Create a trivial spec that checks a tautology (always passes). Run verifier.

Run: `python3 skills/formal-verify/scripts/verify-behavioral.py --specs-dir /tmp/test-specs/`
Expected: Exit code 0, no violations

- [ ] **Step 3: Test with a failing spec**

Create a spec that asserts an unsatisfiable property about known facts. Run verifier.

Expected: Exit code 1, violation with counterexample

- [ ] **Step 4: Commit**

```bash
git add skills/formal-verify/scripts/verify-behavioral.py
git commit -m "feat(formal-verify): add Z3Py behavioral spec verifier"
```

### Task 8: Create run-apalache.sh

**Files:**
- Create: `skills/formal-verify/scripts/run-apalache.sh`

- [ ] **Step 1: Write the Apalache runner**

The script should:
- Accept arguments: `--specs-dir <path>` (default: `.verifier/specs/`), `--json`
- Find all `*.tla` files in specs-dir
- For each TLA+ spec:
  - Run `apalache-mc check` with the spec's .cfg file if it exists, or with default bounds
  - Parse Apalache output for counterexamples
  - If violation found: extract the counterexample trace (sequence of states)
  - Format as: rule name (from MODULE name), counterexample (state sequence), diagnosis
- Collect all results
- Output: JSON array of violations (compatible with verify.sh aggregation)
- Exit code: 0 = all specs pass, 1 = any violations
- If `apalache-mc` not found: warn and skip (Layer 2 TLA+ checks optional if only Z3Py specs exist)

Use `set -euo pipefail`. Timeout per spec: 60 seconds (configurable).

- [ ] **Step 2: Make executable and syntax check**

Run: `chmod +x skills/formal-verify/scripts/run-apalache.sh && bash -n skills/formal-verify/scripts/run-apalache.sh`
Expected: No syntax errors

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/scripts/run-apalache.sh
git commit -m "feat(formal-verify): add Apalache TLA+ spec runner"
```

### Task 9: Create audit-elegance.py

**Files:**
- Create: `skills/formal-verify/scripts/audit-elegance.py` (main entry point, delegates to sub-modules)
- Create: `skills/formal-verify/scripts/elegance/__init__.py`
- Create: `skills/formal-verify/scripts/elegance/complexity.py` (cyclomatic complexity, nesting, length, params)
- Create: `skills/formal-verify/scripts/elegance/consistency.py` (naming, error handling, style uniformity)
- Create: `skills/formal-verify/scripts/elegance/craft.py` (duplication, unnecessary abstraction, over-engineering, dead code)

- [ ] **Step 1: Write the elegance auditor and sub-modules**

**audit-elegance.py** is the entry point. It loads config, delegates to sub-modules, aggregates scores, and outputs the grade. Each sub-module returns a list of deductions.

The script should:
- Accept arguments: `--project-dir <path>`, `--config <path>` (default: `.verifier/elegance.yaml`), `--output-mode <agent|human>`, `--json`
- Load thresholds from elegance.yaml:
  ```yaml
  thresholds:
    cyclomatic_complexity: 10      # per function
    nesting_depth: 4               # max nesting
    function_length: 50            # lines
    file_length: 500               # lines
    parameter_count: 5             # per function
  minimum_grade: B
  exclude:
    - "*/tests/*"
    - "*/generated/*"
  ```

**complexity.py** — For Rust files, use `radon` (via subprocess) for cyclomatic complexity. For Swift files, use `swiftlint` for convention checks and tree-sitter AST walking for complexity metrics that swiftlint does not cover (nesting depth, parameter count). Fall back to pure tree-sitter for both languages if external tools are not installed.

**consistency.py** — Check naming conventions (snake_case for Rust, camelCase for Swift), error handling patterns, comment/documentation density, mixed styles within a module. Uses tree-sitter AST walking.

**craft.py** — Detect duplicated logic (AST subtree similarity), unnecessary abstraction (wrapper functions that just delegate), over-engineering signals (generic parameters used only once, trait implementations with single implementor), dead code and unused code paths. Uses tree-sitter AST walking.
- Compute elegance grade:
  - Start at A (100 points)
  - Deduct points per violation (configurable weights)
  - A: 90-100, B: 80-89, C: 70-79, D: 60-69, F: below 60
  - Every deduction traceable to file:line
- Output: grade + deductions list (JSON or formatted text)
- Exit code: 0 = meets minimum grade, 1 = below minimum

- [ ] **Step 2: Test with a clean file**

Create a simple, well-structured source file. Run auditor.

Expected: Grade A, no deductions

- [ ] **Step 3: Test with a messy file**

Create a file with deep nesting, long functions, mixed naming. Run auditor.

Expected: Grade below A, specific deductions with file:line references

- [ ] **Step 4: Commit**

```bash
git add skills/formal-verify/scripts/audit-elegance.py
git commit -m "feat(formal-verify): add elegance auditor with complexity and craft metrics"
```

---

## Chunk 3: Reference Documentation

### Task 10: Create layer1-structural.md

**Files:**
- Create: `skills/formal-verify/references/layer1-structural.md`

- [ ] **Step 1: Write Layer 1 reference**

Cover:
- How fact extraction works (tree-sitter into AST into relational facts)
- Fact types for Rust and Swift (full list with examples)
- How constraints are translated to Z3 formulas
- Transitive reachability encoding: bounded unrolling to configurable depth
- How counterexamples are extracted from Z3 models
- Incremental extraction: `--incremental` with git-diff detection, `--files` for explicit list, mtime fallback
- Performance characteristics: why this runs in approximately 200ms

- [ ] **Step 2: Verify file is non-empty and well-formed**

Run: `wc -l skills/formal-verify/references/layer1-structural.md && head -3 skills/formal-verify/references/layer1-structural.md`
Expected: More than 30 lines, starts with a markdown heading

**Note:** Apply this same verification pattern to all reference doc tasks (11-16).

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/references/layer1-structural.md
git commit -m "docs(formal-verify): add Layer 1 structural verifier reference"
```

### Task 11: Create layer2-behavioral.md

**Files:**
- Create: `skills/formal-verify/references/layer2-behavioral.md`

- [ ] **Step 1: Write Layer 2 reference**

Cover:
- When to use TLA+/Apalache vs Z3Py (decision table from spec)
- How to write a Z3Py spec file (the `verify(facts)` contract)
- How Z3Py specs interact with the fact database
- TLA+ spec structure: MODULE, VARIABLES, Init, Next, invariants
- How Apalache bounded model checking works
- Apalache daemon mode and warm-start configuration
- How counterexample traces are formatted for state machine violations
- Cross-boundary verification: checking properties that span Rust and Swift

- [ ] **Step 2: Verify file is non-empty and well-formed**

Run: `wc -l skills/formal-verify/references/layer2-behavioral.md && head -3 skills/formal-verify/references/layer2-behavioral.md`
Expected: More than 30 lines, starts with a markdown heading

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/references/layer2-behavioral.md
git commit -m "docs(formal-verify): add Layer 2 behavioral verifier reference"
```

### Task 12: Create layer3-elegance.md

**Files:**
- Create: `skills/formal-verify/references/layer3-elegance.md`

- [ ] **Step 1: Write Layer 3 reference**

Cover:
- All metric categories: complexity, consistency, craft
- **Complexity metrics:** cyclomatic complexity, nesting depth, function length, file length, parameter count
- **Consistency metrics:** naming conventions, error handling style, comment/documentation density, pattern usage uniformity
- **Craft metrics:** duplicated logic, unnecessary abstraction, over-engineering signals, dead code and unused code paths
- How each metric is computed: external tools (radon for Python/Rust complexity, swiftlint for Swift conventions) plus custom tree-sitter AST analysis for cross-language and craft-level checks
- Sub-module architecture: complexity.py, consistency.py, craft.py
- Grading formula: points, weights, deduction rules
- elegance.yaml configuration reference
- What each grade level means in practice
- How to override thresholds for specific files/modules
- Examples of each smell type with before/after

- [ ] **Step 2: Verify file is non-empty and well-formed**

Run: `wc -l skills/formal-verify/references/layer3-elegance.md && head -3 skills/formal-verify/references/layer3-elegance.md`
Expected: More than 30 lines, starts with a markdown heading

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/references/layer3-elegance.md
git commit -m "docs(formal-verify): add Layer 3 elegance auditor reference"
```

### Task 13: Create constraint-yaml-spec.md

**Files:**
- Create: `skills/formal-verify/references/constraint-yaml-spec.md`

- [ ] **Step 1: Write YAML schema reference**

Cover:
- Full schema for structural.yaml
- All constraint types: ownership, boundaries, patterns, migration
- Distinguish **constraint structure keywords** (how rules are composed) from **fact pattern operators** (what is being checked):
  - Structure keywords: `only`, `modules_in`, `modules_matching`, `all_modules` (selectors) + `may`, `must`, `must_not` (assertions) + `except` (exclusions)
  - Fact pattern operators: `import`, `import_from`, `call_pattern`, `reference`, `implement`, `reference_tmux_literal` (and how to define custom patterns)
- Which structure keywords compose with which fact patterns
- Glob pattern syntax for module matching
- How `call_pattern` works with wildcards
- How `except` clauses work
- Complete examples for each constraint type
- How to add custom fact patterns

- [ ] **Step 2: Verify file is non-empty and well-formed**

Run: `wc -l skills/formal-verify/references/constraint-yaml-spec.md && head -3 skills/formal-verify/references/constraint-yaml-spec.md`
Expected: More than 30 lines, starts with a markdown heading

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/references/constraint-yaml-spec.md
git commit -m "docs(formal-verify): add constraint YAML schema reference"
```

### Task 14: Create bootstrap-process.md

**Files:**
- Create: `skills/formal-verify/references/bootstrap-process.md`

- [ ] **Step 1: Write bootstrap reference**

Cover:
- Phase 1 (Install): what install-deps.sh does, prerequisites, troubleshooting
- Phase 2 (Discover): how the agent scans CLAUDE.md, ARCHITECTURE.md, design docs; what it looks for; how it proposes initial constraints
- Phase 3 (Interview): the question flow — agent proposes rules, asks user to confirm/modify in plain English, translates to formal specs
- Phase 4 (Validate): full verification run, baseline report, how to interpret initial violations
- What `.verifier/` looks like after bootstrap
- How to re-run bootstrap (idempotent, additive)

- [ ] **Step 2: Verify file is non-empty and well-formed**

Run: `wc -l skills/formal-verify/references/bootstrap-process.md && head -3 skills/formal-verify/references/bootstrap-process.md`
Expected: More than 30 lines, starts with a markdown heading

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/references/bootstrap-process.md
git commit -m "docs(formal-verify): add bootstrap process reference"
```

### Task 15: Create agent-feedback-loop.md

**Files:**
- Create: `skills/formal-verify/references/agent-feedback-loop.md`

- [ ] **Step 1: Write feedback loop reference**

Cover:
- How PostToolUse hooks trigger Layer 1
- How the violation report is injected into agent context
- Agent output format: counterexample, diagnosis, fix suggestion (with examples)
- Human output format: counterexample, diagnosis (with examples)
- 3-attempt escalation policy: what happens at each attempt, what the escalation report looks like
- How slice checkpoints trigger Layers 1+2
- How pre-commit triggers all layers
- How to configure the hook in a target project

- [ ] **Step 2: Verify file is non-empty and well-formed**

Run: `wc -l skills/formal-verify/references/agent-feedback-loop.md && head -3 skills/formal-verify/references/agent-feedback-loop.md`
Expected: More than 30 lines, starts with a markdown heading

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/references/agent-feedback-loop.md
git commit -m "docs(formal-verify): add agent feedback loop reference"
```

### Task 16: Create spec-authoring-guide.md

**Files:**
- Create: `skills/formal-verify/references/spec-authoring-guide.md`

- [ ] **Step 1: Write spec authoring guide**

Cover:
- The agent-assisted workflow: user describes intent, agent generates spec, user reviews in plain English
- How to write structural constraints (YAML) — with templates for common patterns
- How to write Z3Py behavioral specs — the `verify(facts)` contract, common Z3 patterns
- How to write TLA+ specs — MODULE structure, common patterns, how Apalache config works
- Translating natural language to formal specs: worked examples
  - "Only X should do Y" becomes an ownership constraint
  - "No stale requests" becomes a TLA+ temporal invariant
  - "Data crosses boundary unchanged" becomes a Z3Py equality assertion
- Spec evolution: how to update specs when architecture changes
- Drift detection: how the skill notices CLAUDE.md changes and proposes constraint updates

- [ ] **Step 2: Verify file is non-empty and well-formed**

Run: `wc -l skills/formal-verify/references/spec-authoring-guide.md && head -3 skills/formal-verify/references/spec-authoring-guide.md`
Expected: More than 30 lines, starts with a markdown heading

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/references/spec-authoring-guide.md
git commit -m "docs(formal-verify): add spec authoring guide"
```

---

## Chunk 4: Examples

### Task 17: Create structural-rules.yaml example

**Files:**
- Create: `skills/formal-verify/examples/structural-rules.yaml`

- [ ] **Step 1: Write example constraint file**

Create a realistic example modeled on Capacitor's architecture:

```yaml
# Example structural constraints for a Rust+Swift codebase
# with a UniFFI boundary and strict ownership rules.
#
# Adapted from Capacitor's architectural rules in CLAUDE.md.

ownership:
  - rule: "exclusive_tmux_access"
    description: "Only TmuxRouter may build or execute tmux commands"
    constraint:
      only: "TmuxRouter"
      may: "reference_tmux_literal"

  - rule: "rust_owns_state_derivation"
    description: "Swift must not derive routing state — only project from Rust snapshots"
    constraint:
      modules_in: "swift"
      must_not: "call_pattern:derive_*|reduce_*"
      except: ["RuntimeClient"]

  - rule: "single_activation_coordinator"
    description: "Only TerminalActivationCoordinator may arbitrate activation requests"
    constraint:
      only: "TerminalActivationCoordinator"
      may: "call_pattern:activate_*|arbitrate_*"

boundaries:
  - rule: "uniffi_single_entry"
    description: "Only RuntimeClient may cross the FFI boundary"
    constraint:
      only: "RuntimeClient"
      may: "import:CapacitorCore"

  - rule: "no_driver_core_imports"
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

  - rule: "no_legacy_route_shapes"
    description: "All routing must use RoutingTarget, not raw strings"
    constraint:
      all_modules: true
      must_not: "reference:route_string|raw_route"
```

- [ ] **Step 2: Validate YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('skills/formal-verify/examples/structural-rules.yaml'))"`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/examples/structural-rules.yaml
git commit -m "feat(formal-verify): add example structural constraints (Capacitor-flavored)"
```

### Task 18: Create activation-coordinator.tla example

**Files:**
- Create: `skills/formal-verify/examples/activation-coordinator.tla`

- [ ] **Step 1: Write example TLA+ spec**

Model the activation coordinator state machine:
- States: idle, activating, active, deactivating
- Actions: RequestActivation, CompleteActivation, RequestDeactivation, CompleteDeactivation, StaleRequestArrives
- Invariants:
  - NoDoubleActivation: at most one session is activating/active at any time
  - NoStaleActivation: if a session is active, no pending request has a newer timestamp
  - EventualResolution: every activation request eventually resolves (liveness)
- Include CONSTANT definitions for bounded checking

This should be a valid TLA+ spec that Apalache can check.

- [ ] **Step 2: Verify TLA+ structure**

Run: `head -3 skills/formal-verify/examples/activation-coordinator.tla && tail -3 skills/formal-verify/examples/activation-coordinator.tla`
Expected: File starts with `---- MODULE ActivationCoordinator ----` and ends with `====`

If Apalache is installed, also run: `apalache-mc parse skills/formal-verify/examples/activation-coordinator.tla`
Expected: Parse succeeds (this is optional — skip if Apalache not available)

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/examples/activation-coordinator.tla
git commit -m "feat(formal-verify): add example TLA+ spec for activation coordinator"
```

### Task 19: Create snapshot-contract.py example

**Files:**
- Create: `skills/formal-verify/examples/snapshot-contract.py`

- [ ] **Step 1: Write example Z3Py spec**

Implement the snapshot immutability contract:
- Define Z3 sorts: SnapshotSort, FieldSort, ValueSort
- Define functions: rust_wrote, swift_read, swift_mutates
- Assert: for all fields, swift_read equals rust_wrote
- Assert: swift_mutates should be unsatisfiable
- The `verify(facts)` function should:
  - Load relevant facts (which types cross FFI, which Swift functions access snapshot fields)
  - Build Z3 assertions from the facts
  - Check satisfiability
  - Return violations with concrete counterexamples if mutations detected

Follow the spec file contract from verify-behavioral.py.

- [ ] **Step 2: Test the spec file is valid Python**

Run: `python3 -c "exec(open('skills/formal-verify/examples/snapshot-contract.py').read())"`
Expected: No syntax errors (may fail on z3 import if not installed — that is fine)

- [ ] **Step 3: Commit**

```bash
git add skills/formal-verify/examples/snapshot-contract.py
git commit -m "feat(formal-verify): add example Z3Py spec for snapshot immutability"
```

---

## Chunk 5: Final Integration

### Task 20: End-to-end smoke test

- [ ] **Step 1: Verify complete file structure**

Run: `find skills/formal-verify -type f | sort`
Expected output should match the file structure defined at the top of this plan.

- [ ] **Step 2: Verify all scripts are executable**

Run: `ls -la skills/formal-verify/scripts/*.sh`
Expected: All .sh files have execute permission

- [ ] **Step 3: Verify SKILL.md frontmatter**

Run: `head -15 skills/formal-verify/SKILL.md`
Expected: Valid YAML frontmatter with name, description, license, metadata

- [ ] **Step 4: Verify marketplace.json is valid**

Run: `python3 -m json.tool .claude-plugin/marketplace.json > /dev/null && echo "valid"`
Expected: "valid"

- [ ] **Step 5: Verify all reference files exist and are non-empty**

Run: `wc -l skills/formal-verify/references/*.md`
Expected: All files have content (more than 10 lines each)

- [ ] **Step 6: Run verify.sh syntax check**

Run: `bash -n skills/formal-verify/scripts/verify.sh`
Expected: No errors

- [ ] **Step 7: Run Python syntax check on all scripts**

Run: `python3 -m py_compile skills/formal-verify/scripts/extract-facts.py && python3 -m py_compile skills/formal-verify/scripts/verify-structural.py && python3 -m py_compile skills/formal-verify/scripts/verify-behavioral.py && python3 -m py_compile skills/formal-verify/scripts/audit-elegance.py && echo "all ok"`
Expected: "all ok"

- [ ] **Step 8: Check for unstaged files and commit if needed**

Run: `git status skills/formal-verify/`
Expected: Either "nothing to commit" (all files committed in prior tasks) or a small list of files missed by prior commits. If there are unstaged files, review them to ensure they are intentional (not `__pycache__`, `.pyc`, or temp files), then add them specifically by name and commit:

```bash
git add skills/formal-verify/<specific-files>
git commit -m "feat(formal-verify): add remaining files missed by prior commits"
```
