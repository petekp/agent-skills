# agent-skills

[Agent Skills](https://agentskills.io) for UI development, parameter tuning, and design iteration.

## Installation

```bash
# Install all skills
npx add-skill petekp/agent-skills

# Or install individually via Claude Code
/plugin install petekp/agent-skills
```

## Available Skills

### tuning-panel

Create bespoke parameter tuning panels for iterating on animations, layouts, colors, typography, physics, or any visual parameters.

**Trigger phrases:**
- "create a tuning panel"
- "add parameter controls"
- "tweak parameters visually"
- "fine-tune values"

**Supports:** leva (React), tweakpane (vanilla JS), dat.GUI, SwiftUI native controls

```
skills/tuning-panel/
├── SKILL.md
├── references/
│   ├── platform-libraries.md
│   └── parameter-categories.md
└── examples/
    ├── react-leva-animation.tsx
    └── export-format.md
```

## Structure

```
petekp/agent-skills/
├── .claude-plugin/
│   └── marketplace.json
├── skills/
│   └── tuning-panel/
└── README.md
```

## License

MIT
