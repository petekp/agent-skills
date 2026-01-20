# pete's agent skills

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

### rust

Robust Rust patterns for file-backed data, parsing, persistence, FFI boundaries, and system integration.

**Trigger phrases:**
- "write Rust for file handling"
- "Rust subprocess integration"
- "Serde serialization patterns"
- "UniFFI boundaries"

**Covers:** UTF-8 safety, atomic writes, state machines, defensive error handling

### swiftui

Build world-class SwiftUI interfaces for iOS, iPadOS, macOS, and visionOS.

**Trigger phrases:**
- "build SwiftUI interface"
- "Liquid Glass adoption"
- "SwiftUI animations"
- "Apple-level UI quality"

**Covers:** Layout patterns, state management, design tokens, performance, accessibility

## Structure

```
petekp/agent-skills/
├── .claude-plugin/
│   └── marketplace.json
├── skills/
│   ├── tuning-panel/
│   ├── rust/
│   └── swiftui/
└── README.md
```

## License

MIT
