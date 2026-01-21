---
name: interaction-design
description: Apply interaction design principles to create intuitive, responsive interfaces. Use when designing component behaviors, micro-interactions, loading states, transitions, user flows, accessibility patterns, keyboard navigation, progressive disclosure, animation timing, direct manipulation patterns, instrument/tool design, feedback systems, touch/gesture interactions, evaluating interaction directness, analyzing modal vs modeless designs, or designing post-WIMP interfaces. Also use when reviewing UI for interaction quality or when asked about UX, responsiveness, or user control.
---

# Interaction Design

Guide for designing intuitive, responsive, and accessible user interactions.

## Output Contracts

### Interaction Specification

```markdown
## Interaction Spec: [Component Name]

### States
| State | Visual Treatment | Transition |
|-------|------------------|------------|
| Default | [appearance] | — |
| Hover | [changes] | 150ms ease-out |
| Focus | [focus ring/outline] | immediate |
| Active | [pressed appearance] | 50ms |
| Disabled | [muted appearance] | — |
| Loading | [skeleton/spinner] | [timing] |

### Keyboard
- Tab: [focus behavior]
- Enter/Space: [activation]
- Escape: [dismissal behavior]
- Arrows: [navigation, if applicable]

### Motion
- Duration: [timing in ms]
- Easing: [curve name]
- Reduced motion: [fallback behavior]

### Accessibility
- Focus indicator: [visible, high-contrast description]
- Screen reader: [announcements, aria-labels]
- Touch target: [minimum size]

### Edge Cases
- [Scenario]: [behavior]
```

### Flow Analysis

```markdown
## Flow Analysis: [Journey Name]

### Steps
1. [Step] → [Expected interaction]

### Friction Points
- [Issue]: [recommendation]

### Missing States
- [State that needs design]
```

## Decision Frameworks

### Modal vs Modeless

**Use modeless** (inspectors, inline editing, live preview) when:
- User needs to see results while adjusting
- Trial-and-error is expected
- Multiple parameters interact visually
- Frequent adjustments are needed

**Use modal** (dialogs, overlays) only when:
- Action is destructive/irreversible and needs confirmation
- Complex multi-step wizard with clear sequence
- System needs exclusive attention (auth, payments)
- Context must be isolated (compose email, edit profile)

### Activation Type Selection

| Frequency | Recommend | Rationale |
|-----------|-----------|-----------|
| Constant (scrolling, panning) | Spatial (always visible) or dedicated input | Zero activation cost |
| Frequent (formatting, tools) | Keyboard shortcut + toolbar | Fast access, learnable |
| Occasional (settings, preferences) | Menu or command palette | Saves space, discoverable |
| Rare (export, delete account) | Menu only | Prevents accidents |

### Feedback Timing

| Response Time | User Perception | Design Response |
|---------------|-----------------|-----------------|
| <100ms | Instant | Direct manipulation, no indicator needed |
| 100ms–1s | Fast | Subtle state change (opacity, cursor) |
| 1s–10s | Working | Spinner or progress indicator |
| >10s | Long | Progress bar with estimate, allow cancel |

## Instrument Design

Every interactive element is an *instrument* mediating between user and content. Evaluate instruments on three dimensions:

### Degree of Indirection

Lower indirection = more direct manipulation.

**Spatial offset** — Distance between control and effect:
- Ideal: Handles directly on the object (resize handles, drag-to-reorder)
- Good: Inspector panel alongside content
- Poor: Toolbar at screen edge
- Avoid: Modal dialog covering content

**Temporal offset** — Delay before seeing results:
- Ideal: Real-time as user drags/types (live preview)
- Good: On blur, release, or short debounce
- Poor: Requires clicking "Apply" to preview
- Avoid: Only visible after closing dialog

### Degree of Integration

Ratio of input DOF to output DOF. Higher = more efficient.

| Interaction | Input DOF | Output DOF | Integration | Verdict |
|-------------|-----------|------------|-------------|---------|
| 2D drag for position | 2 | 2 | 1.0 | Ideal |
| Scrollbar for 2D pan | 1 | 2 | 0.5 | Use panning instead |
| 3 sliders for HSL | 1 each | 3 | 0.33 | Consider unified picker |
| Rotation dial | 1 | 1 | 1.0 | Ideal |
| Text field for angle | typing | 1 | ~0.1 | Add drag or dial |

### Degree of Compatibility

Similarity between gesture and result. Higher = more intuitive.

| Compatibility | Example |
|---------------|---------|
| High | Drag down → content moves down |
| Medium | Pinch → zoom (metaphor-based) |
| Low | Drag scrollbar down → content moves up |
| Very low | Type "24" → font becomes 24pt |

**Design goal**: Maximize compatibility, especially for frequent operations.

## Core Principles

### Feedback & Responsiveness
- Every action deserves acknowledgment (visual, auditory, or haptic)
- Optimistic UI: update immediately, reconcile errors gracefully
- Skeleton screens > spinners for perceived performance
- Show system status continuously, not just on request

### Progressive Disclosure
- Show only what's needed at each step
- Reveal complexity gradually through interaction
- Use sensible defaults; make advanced options discoverable
- Chunk information to reduce cognitive load

### Direct Manipulation
- Objects should feel tangible and respond to interaction
- Maintain visible connection between action and result
- Support undo/redo for reversible actions
- Provide clear affordances for interactive elements
- Prefer manipulation over specification (drag vs. type coordinates)

### Consistency & Predictability
- Follow platform conventions (web, iOS, Android)
- Maintain internal consistency across the application
- Use familiar patterns before inventing new ones
- Same gesture = same result throughout the app

## Touch & Gesture Patterns

### Target Sizes

| Context | Minimum | Comfortable |
|---------|---------|-------------|
| iOS | 44×44pt | 48×48pt+ |
| Android | 48×48dp | 56×56dp+ |
| Web (touch) | 44×44px | 48×48px+ |
| Spacing between targets | 8pt/dp | 12pt/dp |

### Gesture Vocabulary

| Gesture | Common Use | Considerations |
|---------|------------|----------------|
| Tap | Primary action | Needs visual affordance |
| Long-press | Secondary/context menu | Needs discoverability hint (subtle animation, tooltip) |
| Swipe | Delete, navigate, reveal actions | Always provide undo; avoid for primary destructive actions |
| Pinch | Zoom | Maintain focal point under fingers |
| Two-finger drag | Pan (when pinch-zoom active) | |
| Edge swipe | System navigation | Don't override; use insets |

### Touch Feedback
- **Timing**: Highlight on touch-down, not touch-up
- **Haptic**: Light tap for selections, medium for confirmations, heavy for warnings
- **Visual**: Ripple for unbounded areas, state change for bounded buttons
- **Audio**: Subtle clicks for significant actions (optional, respect system settings)

## Anti-Patterns

### High Temporal Offset
❌ Modal dialog with Preview button
✓ Inline editing with live preview

❌ Settings that require app restart
✓ Settings applied immediately with undo option

### Low Compatibility
❌ Text field for color selection
✓ Color picker with visual feedback

❌ Dropdown for numeric range
✓ Slider with value display

### Activation Cost Traps
❌ Frequently-used tool buried in submenu
✓ Toolbar position or keyboard shortcut

❌ Tool palette requires clicking then clicking target
✓ Click-through tools or keyboard modifiers

### Modal Overuse
❌ Confirmation dialog for every action
✓ Undo support; confirm only for irreversible/destructive

❌ Modal for simple preference
✓ Toggle or inline control

### Hidden State
❌ Current mode only visible in status bar
✓ Cursor change, selection handles, visible mode indicator

❌ Unsaved changes with no indicator
✓ Dirty state in title, tab, or save button

### Breaking Direct Manipulation
❌ Must click "Apply" to see font change
✓ Font changes as user hovers/selects options

❌ Separate "Edit" mode to make changes
✓ Inline editing, click-to-edit

## Platform Considerations

### Web
- Focus management crucial for SPAs (trap focus in modals, restore on close)
- Reduced motion: respect `prefers-reduced-motion` media query
- Design for touch + mouse: avoid hover-only interactions
- Handle viewport resize and orientation changes
- Support keyboard navigation for all interactive elements

### iOS
- Respect system gestures (edge swipes, scroll bounce physics)
- Support Dynamic Type scaling (test at largest sizes)
- Use SF Symbols for consistent, adaptable iconography
- Haptic feedback via UIImpactFeedbackGenerator
- Support VoiceOver with proper trait annotations

### macOS
- Support keyboard shortcuts and menu bar integration
- Respect reduced motion and increased contrast settings
- Multi-window: maintain state across windows
- Support trackpad gestures (pinch, rotate, swipe)

### Android
- Material motion principles (container transform, shared axis)
- Predictive back gesture (Android 14+): prepare back preview
- Edge-to-edge with proper WindowInsets handling
- Support TalkBack with contentDescription

## Micro-Interactions

### State Transitions
- **Hover**: 150-200ms ease-out for color/shadow changes
- **Focus**: Immediate visible indicator (outline, ring, glow)
- **Active/Pressed**: Scale down slightly (0.95-0.98) or darken
- **Disabled**: Reduced opacity (0.5-0.6), cursor: not-allowed
- **Loading**: Pulsing skeleton or spinner

### Animation Timing
- **Micro**: 100-200ms (button states, toggles)
- **Small**: 200-300ms (dropdowns, tooltips)
- **Medium**: 300-400ms (modals, panels, cards)
- **Large**: 400-600ms (page transitions, complex reveals)

See [references/animation-timing.md](references/animation-timing.md) for detailed curves and spring configurations.

## Component Behaviors

For detailed patterns on forms, modals, menus, drag-and-drop, and other components, see [references/component-patterns.md](references/component-patterns.md).

## Accessibility Patterns

For ARIA patterns, focus management strategies, and screen reader considerations, see [references/accessibility-patterns.md](references/accessibility-patterns.md).

## Theoretical Foundations

### Direct Manipulation (Shneiderman)
Core principles: visibility of objects, rapid reversible actions, direct object manipulation replacing command syntax. See [references/direct-manipulation.md](references/direct-manipulation.md).

### Instrumental Interaction (Beaudouin-Lafon)
Extends direct manipulation to post-WIMP interfaces. Interaction mediated by instruments (tools) operating on domain objects. Provides framework for evaluating indirection, integration, and compatibility. See [references/instrumental-interaction.md](references/instrumental-interaction.md).
