# tuning-panel

An [Agent Skill](https://agentskills.io) for creating bespoke parameter tuning panels that give users visual control over values they're iterating on.

## Installation

```bash
npx add-skill petepetrash/tuning-panel
```

## What it does

When you're iterating on animations, layouts, colors, typography, physics, or any visual parameters, this skill creates a debug panel that:

- Surfaces **all tunable parameters** for the current task
- Uses **platform-appropriate libraries** (leva for React, tweakpane for vanilla JS, native controls for Swift)
- Wraps panels in **debug mode** so they never appear in production
- Includes **"Copy for LLM" export** that formats tuned values for easy handoff

## Trigger phrases

- "create a tuning panel"
- "add parameter controls"
- "build a debug panel"
- "tweak parameters visually"
- "fine-tune values"
- "dial in the settings"

Also activates when you mention `leva`, `dat.GUI`, `tweakpane`, or similar libraries.

## Example

```
User: I'm tuning the hover animation on this card component

Claude: [Creates a leva panel with spring physics, transform values,
         shadow parameters, and a "Copy for LLM" button that exports
         only the changed values]
```

## Structure

```
tuning-panel/
├── SKILL.md                           # Main skill instructions
├── references/
│   ├── platform-libraries.md          # Setup guides for leva, tweakpane, dat.GUI, SwiftUI
│   └── parameter-categories.md        # Exhaustive parameter lists by domain
└── examples/
    ├── react-leva-animation.tsx       # Complete React tuning panel
    └── export-format.md               # LLM export template
```

## License

MIT
