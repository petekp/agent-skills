# Ready-to-Use Prompts

Copy and adapt these prompts for each phase.

## Research Prompts

```
Read [this folder / this system / this feature] in depth. Understand how it works
deeply — all its specificities, edge cases, and integration points. Write a detailed
report of your findings in .claude/research.md
```

```
Study the [notification / auth / payment] system in great detail. Understand the
intricacies of how it works end-to-end. Write a detailed .claude/research.md
document with everything there is to know about it.
```

```
Go through the [task scheduling / data sync / caching] flow. Understand it deeply
and look for potential bugs. Keep researching until you find all the issues. Write
a detailed report in .claude/research.md
```

## Planning Prompts

```
I want to build [feature name and description] that [business outcome]. Write a
detailed .claude/plan.md outlining how to implement this. Include code snippets.
Read source files before suggesting changes.
```

```
The [endpoint/component/system] should [desired change]. Write a detailed
.claude/plan.md for how to achieve this. Base the plan on the actual codebase.
```

```
Here's how [reference project] implements [feature] — [paste code]. Write a
.claude/plan.md explaining how we can adopt a similar approach in our codebase.
```

## Annotation Prompts

```
I added notes to the plan. Address all the notes and update the document
accordingly. Don't implement yet.
```

```
Review .claude/plan.md — I added inline annotations. Address every note,
update the plan, and remove my annotations once addressed. Don't implement yet.
```

## Todo List Prompt

```
Add a detailed todo list to the plan with all phases and individual tasks
necessary to complete it. Don't implement yet.
```

## Implementation Prompts

```
Implement it all. When you're done with a task or phase, mark it as completed
in the plan document. Do not stop until all tasks and phases are completed.
Do not add unnecessary comments or jsdocs, do not use any or unknown types.
Continuously run typecheck to make sure you're not introducing new issues.
```

## Feedback Examples

```
You didn't implement the [function name].
```

```
Move this to [correct location], not [wrong location].
```

```
This should look exactly like [reference component] — same [specific attributes].
```

```
I reverted everything. Now all I want is [narrowed scope] — nothing else.
```
