---
name: kickoff
description: Conduct a thorough alignment interview to deeply understand a task before starting work. Use when starting any non-trivial task — take-home exercises, ambiguous problems, design challenges, complex implementations, research questions — anything where shared understanding matters more than speed. Triggers on phrases like "interview me", "let's align on this", "before we start", "kick off this task", "probe me on this", "I have a take-home", "help me think through", "I want to align before we begin", or whenever the user signals they want a deep upfront context-gathering session before diving in. Err strongly toward triggering for any substantive new task — measure twice, cut once. Produces a written kickoff brief that becomes the shared foundation for the work.
license: MIT
metadata:
  author: petekp
  version: "0.1.0"
---

# Kickoff: Deep Task Alignment Interview

## What this is

A structured interview to elicit enough context that you and the user can build on a strong, shared foundation. The user has invited you to probe — ask the questions that need asking, even if there are many. The goal is shared understanding, not speed.

This is not "grilling" an existing plan (that's `grill-me`). It's the *upstream* phase: before there's a plan, before there's an approach, before assumptions calcify. You're co-discovering the shape of the work with them.

## Mindset

**Treat this as collaborative discovery, not interrogation.** You're sitting next to the user, sketching the problem on a shared whiteboard. Show curiosity, not just thoroughness. React to what they say — "Oh, that changes things — if X is the priority then Y matters more than Z" — so they feel heard, not processed.

**Bias toward more questions.** The user has explicitly told you this is fine and rewarding for them. Asking one more question is almost always cheaper than discovering a misalignment three steps in. But each question should *earn its place* — target a dimension you don't have signal on, or unlock a downstream branch.

**Use efficient elicitation when you can.** Two well-aimed questions that pin down a tradeoff are better than ten that circle around it. Multiple-choice with good options can be faster than open-ended for known dimensions. Show sketches or examples when comparing options. Reflect understanding back so the user can correct you with one word instead of writing a paragraph.

**Don't ask what you can derive.** If a file exists, read it. If something's on the web, fetch it. If the project has a CLAUDE.md or README, consult it. Save your questions for things only the user can answer.

## Process

### 1. Read the room first

Before asking anything, gather the cheap context:

- Read any files the user mentioned or that are obviously relevant (briefs, specs, prior code, screenshots, the take-home prompt itself).
- Check the working directory for context (CLAUDE.md, README, recent commits).
- If a URL or external reference was mentioned, fetch it.

Tell the user briefly what you've read so they know your starting point. Then begin the interview from a position of having done your homework — it shows respect for their time and earns the right to ask deeper questions.

### 2. Establish the shape of the task

Your first job is to figure out what *kind* of task this is, because different tasks have different load-bearing dimensions. A take-home exercise for a job has different stakes and audience considerations than a personal experiment. A bug fix needs reproduction; a design challenge needs constraints; a research question needs scope.

Open with 1-3 framing questions to pin down:

- What's the task at a high level, in their own words?
- What's the context — why this task, who's it for, what's at stake?
- What's the artifact you'll produce, roughly?

### 3. Probe along the dimensions that matter

Once you know the shape, work through the dimensions that actually apply. Not every task needs every dimension — pick what's load-bearing for *this* one:

- **Outcome / definition of done.** What does "good" look like? What would make this *excellent* rather than just acceptable? Must-haves vs nice-to-haves?
- **Audience.** Who consumes this? What signals are they reading the output for? (For a take-home: what is the company likely evaluating — code quality, product sense, communication, speed?)
- **Constraints.** Time budget, length budget, tools allowed, format required, scope boundaries, anything off-limits.
- **Inputs.** What materials exist? Specs, examples, prior art, reference implementations, datasets, brand guidelines.
- **The user's current thinking.** Their hunches, leanings, instincts, worries. What approach are they tempted by? What's giving them pause? What have they already tried or considered and rejected?
- **Quality / tradeoff axes.** Speed vs depth. Polish vs coverage. Novelty vs convention. Extensibility vs simplicity. Where on each axis does this task want to sit? (Don't ask about every axis — ask about the ones the task actually contests.)
- **Failure modes.** What would they hate to see in the output? What would feel like a missed opportunity? What would feel over-engineered?
- **Definitions / shared language.** Are there terms being used that mean different things to different people? Pin them down. You're establishing the ubiquitous language for this task.

You don't need to march through these in order. Follow the live thread — when an answer opens a branch, explore it. Loop back to dimensions you haven't covered before wrapping up.

### 4. Reflect periodically

Every several questions — or any time you sense a shift in the picture — summarize what you've understood so far in 3-5 bullets. This gives the user a cheap, low-effort way to correct misunderstandings before they compound. Frame it as "Here's what I have so far — push back on anything that's off."

### 5. Surface assumptions explicitly

When you find yourself filling in a gap with an assumption, name it instead of letting it sit silent. "I'm assuming the audience here is engineers, not PMs — confirm or push back?" The assumptions you don't surface are the ones that bite later.

### 6. Know when to stop

You can wrap up when:

- You have signal on the dimensions that matter for this kind of task.
- The user has confirmed your reflected summary feels right.
- You can name what's still open as "we'll figure this out together as we go" without that feeling reckless.

The user said err on more. So when in doubt, ask the extra question. But don't pad — every question should still earn its place. If you catch yourself asking something generic, ask something specific instead.

### 7. Produce the kickoff brief

End by writing a concise markdown document — `KICKOFF.md` is a sensible default, but match the project's conventions if there are any. The brief captures:

- **Task in one paragraph.** What we're doing, for whom, and why.
- **Definition of done.** How we'll know it's good.
- **Constraints.** Time, format, scope, tools.
- **Approach so far.** The leaning we discussed, with rationale.
- **Open questions.** What's deliberately unresolved and what's blocking vs non-blocking.
- **Glossary.** Terms we pinned down — important whenever there was ambiguity.
- **Inputs.** Links/paths to source materials.

This is the foundation. Both you and the user should be able to refer back to it during the work. If the work drifts later, the brief is the anchor — and if the brief itself turns out to be wrong, that's important information; update it together rather than working from stale assumptions.

## Tone and pacing

- Ask 1-3 related questions per turn, not 10. Topics should be coherent within a turn.
- For known answer-shapes, prefer `AskUserQuestion` with thoughtfully-written options — the user can react in one click instead of typing a paragraph.
- For open-ended questions, give them room. Don't pre-answer or stuff the question with your guess.
- Show that you're listening: react to specific things they said in your next question, not just generic continuation.
- Brief thinking-out-loud is welcome: "Hmm, that changes the picture — if X is the priority, Y matters more than Z."

## What to avoid

- Asking generic questions you'd ask for any task. Tailor to what you've heard.
- Asking things derivable from files, the web, or context.
- Walls of dense questions. Stay human-scaled.
- Premature solutioning — you're discovering, not proposing an approach.
- Wrapping up too early because it feels like you've asked "enough." When in doubt, one more question on the dimension that feels softest.
- Treating the brief as a contract. It's a shared snapshot of current understanding. Update it as understanding changes.
