---
name: write-goal
description: Compile a plain-language task into a concise, auditable Codex or Claude Code `/goal`, or explain why a normal prompt fits better. Use when the user asks to draft, formulate, rewrite, tighten, or create a goal for multi-step work that needs a durable objective, transcript-visible proof, constraints, bounded stop conditions, host-aware operation, and risk-based review depth.
---

# Write Goal

## Overview

Act as a host-aware goal compiler, not a generic prompt rewriter. Turn the user's request into a compact `/goal` that has a clear definition of done, operating instructions, proof the evaluator can see, and a bounded stop policy. Draft the goal; do not activate it unless the user explicitly asks you to start or set the goal.

## Workflow

1. Extract the real task from the prompt that invoked this skill. Ignore the skill mention itself.
2. Identify the target host:
   - Claude Code: optimize for its `/goal` completion-condition evaluator.
   - Codex: optimize for a durable Codex Goal and long-running validation loop.
   - Unspecified: produce a portable `/goal` and call out host assumptions only if they matter.
3. Decide whether a goal is appropriate. Use a goal for durable, multi-step work with a verifiable end state. For one-line edits, simple explanations, or vague improvement requests with no checkable finish line, return a tightened normal prompt instead.
4. Compile the goal in two layers:
   - Definition of done: one primary end state, the proof surface, required constraints, scope boundaries, and any turn/time/cost fuse.
   - Operating instructions: how the agent should choose the next action, what evidence to surface each iteration, what completion receipt to print, and when to stop as blocked.
5. Choose review depth from the risk tiers below. Do not blindly attach the heaviest review loop to every goal.
6. Keep the final `/goal` short enough to be readable. Prefer one compact block, but use labels like `Definition of done:` and `Operating instructions:` when they make evaluator behavior clearer.
7. If details are missing, make conservative assumptions inline. Ask only when missing information would make the goal unsafe or impossible to verify.

## Host Semantics

Codex goals are for long-running work that should keep progressing across turns toward a verifiable stopping condition. Good Codex goals name the durable objective, validation loop, checkpoint evidence, scope boundary, and pause/blocked behavior.

Claude Code `/goal` sets a session-scoped completion condition. A small evaluator model checks the condition after each turn. It does not run commands or read files independently, so the goal must require Claude to surface proof in the transcript. A strong Claude Code condition has one measurable end state, a stated check such as a passing command or empty queue, and constraints that matter. Include a turn, time, or cost bound when runaway work is a risk. Claude Code goals are limited to 4,000 characters and require Claude Code v2.1.139 or later.

## Goal Anatomy

Every compiled goal should answer these questions:

- End state: what is true when the work is done?
- Proof: what exact command result, artifact, log line, review output, source citation, or checklist must appear in the transcript?
- Constraints: what behavior, APIs, files, style rules, budgets, or safety boundaries must not be violated?
- Scope: which repos, files, tools, data, and resources are allowed?
- Operating loop: after each result, how should the agent inspect evidence and choose the next action?
- Stop policy: when should the agent stop instead of continuing, and what blocker report should it print?
- Receipt: what changed files, exact checks, remaining risks, and review outcome must be printed before completion?

Prefer this shape:

```text
/goal Definition of done: <one measurable end state>, verified by <specific proof surfaced in the transcript>, while preserving <constraints and scope>. Operating instructions: use <allowed inputs/tools/files>, inspect <latest evidence> between iterations, make the smallest defensible next move, and print a completion receipt with <changed files/artifacts, exact check results, review outcome, and remaining risks>. Review depth: <risk tier and required review loop>. Stop if <blocked condition, no defensible path, or turn/time/cost bound> with <attempted paths, evidence gathered, blocker, unresolved findings, and next input needed>.
```

Do not prescribe every implementation step unless the user already did. Leave enough room for the agent to adapt.

## Risk-Based Review Depth

Use the lightest review loop that protects the user:

- Low risk: docs, small content edits, read-only research, or local cleanup with no runtime behavior. Require one self-review or adversarial review, resolve medium-or-above findings, then complete.
- Medium risk: normal code changes, refactors, tests, generated artifacts, or docs that describe command or behavior claims without changing them. Require one adversarial review plus focused verification; resolve all medium, high, and critical findings before completion.
- High risk: changes to activation, packaging, command behavior, public APIs, migrations, security, data loss, permissions, release/publish flows, or broad cross-file behavior. Require two consecutive adversarial reviews with no medium-or-above findings. If a follow-up review finds a medium-or-above issue, resolve it and restart the two-clean-review count.

Treat severity as impact on the goal:

- Critical or high: the objective is not met, evidence is false or missing, scope is unsafe, or a stated constraint likely regresses.
- Medium: the result may pass superficially but leaves a meaningful gap in evidence, scope, maintainability, or user-facing quality.
- Low: polish or optional improvement that does not block the goal.

## Task Fit

- Coding or refactoring: include behavior or code state, focused tests/build commands, regression checks, changed-file receipt, and what must not regress.
- Debugging or flaky tests: include reproduction evidence, hypothesis updates, focused verification, repeated pass criteria, and the point where missing evidence becomes a blocker.
- Research or audits: require a claim inventory, evidence mapping, confidence labels, source links or file citations, and a final report that separates confirmed, supported, blocked, and uncertain claims.
- Docs or content: name the artifact, reader outcome, source-of-truth checks, build/link checks, terminology constraints, and review depth based on whether command semantics are involved.
- Vague requests: narrow with explicit assumptions if there is a plausible proof surface. If there is not, return a tightened normal prompt instead.
- One-off tasks: do not force a goal. Provide the better normal prompt and one sentence explaining why.

## Output

Return:

1. `Recommended Goal`: a ready-to-use `/goal ...` block.
2. `Assumptions`: only if you filled gaps that matter.
3. `Compiler Notes`: one or two bullets naming the host, proof surface, stop bound, and review tier.

If a goal is not appropriate, return:

1. `Better As A Prompt`: a concise normal prompt.
2. `Why Not A Goal`: one sentence explaining the missing durable objective or verification surface.

Keep the answer concise. The user came for the goal, not a lecture about goals.

## Examples

User prompt:

```text
Use $write-goal for Claude Code: keep working on this flaky checkout test until it is fixed or we know exactly why it cannot be fixed.
```

Recommended Goal:

```text
/goal Definition of done: the flaky checkout test either passes reliably or has a transcript-supported blocker, verified by reproducing the failure when possible and then surfacing exact focused test results plus the relevant regression command output, while preserving public checkout behavior and existing test coverage. Operating instructions: use the checkout code, fixtures, local logs, and test commands; after each failure, update the hypothesis, make the smallest defensible change, and rerun focused verification. Print a completion receipt with changed files, exact commands and exits, review outcome, and remaining risks. Review depth: medium, with one adversarial review and all medium-or-above findings resolved. Stop after 20 turns, if the failure cannot be reproduced after reasonable attempts, or if no valid path remains; report attempted reproductions, evidence gathered, blocker, unresolved findings, and next input needed.
```

User prompt:

```text
Use $write-goal for Codex: migrate the billing module to the new API.
```

Recommended Goal:

```text
/goal Definition of done: the billing module uses the new API at every intended call site, verified by source inspection surfaced in the transcript plus passing focused billing tests and the relevant build/typecheck command, while preserving public billing behavior, existing data contracts, and unrelated files. Operating instructions: use the billing code, migration docs, tests, and local commands; work in small slices, inspect failures after each run, and keep a progress receipt of migrated call sites. Print a completion receipt with changed files, exact validation results, unresolved risks, and review outcome. Review depth: high because this is a behavior-affecting migration; require two consecutive adversarial reviews with no medium-or-above findings before completion. Stop if verification cannot run, API behavior cannot be confirmed, or no defensible path remains; report attempted paths, evidence, blocker, and next input needed.
```

Bad goal:

```text
/goal Improve the docs.
```

Better goal:

```text
/goal Definition of done: the Goals docs clearly explain when to use `/goal`, how to set/check/clear one, and two realistic examples, verified by a successful docs build and transcript-visible source checks against the current host docs, while preserving existing terminology and avoiding unrelated docs churn. Operating instructions: edit only the relevant docs, compare each claim to source docs, and print changed files, exact build result, review outcome, and remaining risks. Review depth: medium because command semantics are described; run one adversarial review and resolve medium-or-above findings. Stop if source behavior cannot be confirmed or validation cannot run, with attempted checks and next input needed.
```

Bad goal:

```text
/goal Make the app better and faster.
```

Better As A Prompt:

```text
Inspect the app and propose the three highest-impact improvements for performance or UX, with evidence for each and a suggested verification command.
```

Why Not A Goal: The request has multiple undefined outcomes and no single proof surface.

Bad goal:

```text
/goal All tests pass.
```

Better goal:

```text
/goal Definition of done: the currently failing test suite passes without hiding or deleting coverage, verified by transcript-visible output from the failing command rerun plus a clean `git diff` review showing no unrelated test weakening. Operating instructions: identify the first failure, fix the smallest root cause, rerun focused checks before broad checks, and print changed files, exact commands/exits, review outcome, and remaining risks. Review depth: medium; run one adversarial review and resolve medium-or-above findings. Stop after 15 turns, if the suite cannot be run, or if failures require product decisions; report evidence, blocker, and next input needed.
```

User prompt:

```text
Use $write-goal: explain this error message.
```

Better As A Prompt:

```text
Explain this error message in plain English, identify the likely cause, and suggest the next command or file to inspect.
```

Why Not A Goal: This is a one-off explanation request without a durable objective that needs continued work.
