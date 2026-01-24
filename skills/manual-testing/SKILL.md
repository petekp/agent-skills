---
name: manual-testing
description: Guide users step-by-step through manually testing whatever is currently being worked on. Use when asked to "test this", "verify it works", "let's test", "manual testing", "QA this", "check if it works", or after implementing a feature that needs verification before proceeding.
---

# Manual Testing

Guide users through systematic manual verification using selectable outcome options to minimize typing.

## Workflow

### 1. Analyze Current Context

Examine recent work to identify what needs testing:
- Review recent file changes and conversation history
- Identify the feature, fix, or change to verify
- Determine testable behaviors and expected outcomes

### 2. Generate Testing Plan

Create a focused list of verification steps:
- Break down into discrete, observable actions
- Order from most critical to least critical
- Keep steps atomic (one thing to verify per step)

### 3. Execute Steps Sequentially

For each step, use AskUserQuestion with predicted outcomes:

```
Step N of M: [Brief description of what to test]

**Action:** [Specific instruction - what to do]

**Expected:** [What should happen if working correctly]
```

Then immediately ask for the outcome using AskUserQuestion with:
- 2-4 most likely outcomes as selectable options
- Always include a free-text "Other" option (provided automatically)

**Option design:**
- First option: The expected/success outcome (label it clearly)
- Remaining options: Common failure modes or edge cases
- Keep labels short (3-7 words), use descriptions for detail

### 4. Handle Results

**On success:** Move to next step with brief acknowledgment.

**On failure/unexpected:**
- Note the issue
- Ask if user wants to investigate now or continue testing
- If continuing, track the issue for summary

### 5. Summarize

After all steps complete:
- List passed/failed steps
- Summarize any issues found
- Recommend next actions

## Example Flow

```
Step 1 of 3: Verify the button renders

**Action:** Navigate to the settings page and look for the "Export" button in the top-right corner.

**Expected:** Blue button with "Export" label should be visible.
```

Then call AskUserQuestion:
```json
{
  "questions": [{
    "question": "What do you see?",
    "header": "Step 1",
    "options": [
      {"label": "Button visible (Recommended)", "description": "Blue 'Export' button appears in top-right"},
      {"label": "Button missing", "description": "No export button visible anywhere"},
      {"label": "Button wrong style", "description": "Button exists but looks different than expected"},
      {"label": "Page error", "description": "Page failed to load or shows error"}
    ],
    "multiSelect": false
  }]
}
```

## Guidelines

- Prefer 3-4 outcome options per step (not too few, not overwhelming)
- Make the success case obvious (first option, marked "Recommended")
- Predict realistic failures based on what's being tested
- Keep the pace brisk—don't over-explain between steps
- If a step requires setup, include setup in the action instructions
