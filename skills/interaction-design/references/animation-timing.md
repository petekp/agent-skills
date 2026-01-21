# Animation Timing Reference

Detailed timing curves, spring configurations, and animation patterns for interaction design.

## Table of Contents
- [Duration Guidelines](#duration-guidelines)
- [Easing Functions](#easing-functions)
- [Spring Physics](#spring-physics)
- [Animation Patterns by Component](#animation-patterns-by-component)
- [Choreography & Sequencing](#choreography--sequencing)
- [Reduced Motion](#reduced-motion)
- [Platform-Specific Curves](#platform-specific-curves)

---

## Duration Guidelines

### By Interaction Scale

| Scale | Duration | Use Cases |
|-------|----------|-----------|
| Micro | 50-150ms | Button press, toggle, checkbox |
| Small | 150-250ms | Tooltip, dropdown, state change |
| Medium | 250-400ms | Modal, sidebar, card flip |
| Large | 400-600ms | Page transition, onboarding reveal |
| Extra Large | 600-1000ms | Complex orchestrated sequences |

### By Distance Traveled

| Distance | Recommended Duration |
|----------|---------------------|
| < 100px | 150-200ms |
| 100-300px | 200-350ms |
| 300-500px | 300-450ms |
| > 500px | 400-600ms |

### The 100ms Rule

- **< 100ms**: Feels instant, no easing needed
- **100-300ms**: Perceptible motion, use easing
- **> 300ms**: Noticeable animation, use strong easing

### Entry vs Exit

Exit animations should be faster than entry:
- Entry: 250-350ms (welcoming, smooth)
- Exit: 150-250ms (decisive, quick)

---

## Easing Functions

### CSS Cubic Bezier Reference

```
ease:        cubic-bezier(0.25, 0.1, 0.25, 1.0)
ease-in:     cubic-bezier(0.42, 0, 1.0, 1.0)
ease-out:    cubic-bezier(0, 0, 0.58, 1.0)
ease-in-out: cubic-bezier(0.42, 0, 0.58, 1.0)
linear:      cubic-bezier(0, 0, 1, 1)
```

### Recommended Curves

**Standard (ease-out)** — Elements entering, responding to user
```
cubic-bezier(0.2, 0, 0, 1)
```
Fast start, gentle deceleration. Default choice.

**Decelerate** — Elements appearing from offscreen
```
cubic-bezier(0, 0, 0.2, 1)
```
Enters at velocity, smoothly stops.

**Accelerate** — Elements leaving the screen
```
cubic-bezier(0.4, 0, 1, 1)
```
Starts slow, exits quickly.

**Sharp** — Elements that change size or shape
```
cubic-bezier(0.4, 0, 0.6, 1)
```
Quick transition, minimal oversimplification.

**Overshoot** — Playful, bouncy feel
```
cubic-bezier(0.34, 1.56, 0.64, 1)
```
Exceeds target, settles back. Use sparingly.

### When to Use Each

| Scenario | Easing | Why |
|----------|--------|-----|
| Fade in | ease-out | Welcoming arrival |
| Fade out | ease-in | Decisive departure |
| Hover state | ease-out | Responsive to user |
| Slide in panel | ease-out | Smooth deceleration into view |
| Slide out panel | ease-in | Accelerates out of way |
| Modal backdrop | linear | Consistent dim effect |
| Bounce/playful | overshoot | Personality, delight |
| Loading pulse | ease-in-out | Continuous, rhythmic |

---

## Spring Physics

Spring animations feel natural because they model real-world physics. Key parameters:

### Spring Parameters

| Parameter | Effect | Typical Range |
|-----------|--------|---------------|
| **Mass** | Weight of element; higher = slower | 0.5-2 |
| **Stiffness** | Spring tightness; higher = snappier | 100-500 |
| **Damping** | Resistance; higher = less bounce | 10-40 |

### Common Presets

**Gentle** — Soft, flowing motion
```
mass: 1, stiffness: 120, damping: 14
```
Use for: Large panels, page transitions.

**Snappy** — Quick, responsive
```
mass: 1, stiffness: 300, damping: 20
```
Use for: Buttons, toggles, small UI elements.

**Bouncy** — Playful, energetic
```
mass: 1, stiffness: 200, damping: 10
```
Use for: Celebrations, notifications, fun interactions.

**Stiff** — Minimal oscillation
```
mass: 1, stiffness: 400, damping: 30
```
Use for: Precision controls, professional apps.

### Spring Libraries

**Framer Motion (React)**
```jsx
<motion.div animate={{ x: 100 }} transition={{ type: "spring", stiffness: 300, damping: 20 }} />
```

**React Spring**
```jsx
useSpring({ to: { x: 100 }, config: { tension: 300, friction: 20 } })
```

**SwiftUI**
```swift
.animation(.spring(response: 0.5, dampingFraction: 0.7))
```

### Converting Between Systems

| Framer Motion | React Spring | Approximate |
|---------------|--------------|-------------|
| stiffness: 100 | tension: 100 | Gentle |
| stiffness: 300 | tension: 300 | Snappy |
| damping: 10 | friction: 10 | Bouncy |
| damping: 30 | friction: 30 | Stiff |

---

## Animation Patterns by Component

### Buttons

**Hover**
- Property: background-color, box-shadow
- Duration: 150ms
- Easing: ease-out

**Press**
- Property: scale (0.95-0.98), background-color
- Duration: 50-100ms
- Easing: ease-out

**Loading**
- Spinner rotation: continuous, 1-1.5s per rotation
- Pulse: 1-2s ease-in-out infinite

### Modals / Dialogs

**Open**
```
backdrop: opacity 0 → 1, 200ms ease-out
modal: scale 0.95 → 1, opacity 0 → 1, 250ms ease-out
```

**Close**
```
modal: opacity 1 → 0, 150ms ease-in
backdrop: opacity 1 → 0, 200ms ease-in
```

### Dropdowns / Menus

**Open**
```
opacity: 0 → 1
transform: translateY(-8px) → translateY(0)
duration: 150-200ms
easing: ease-out
```

**Close**
```
opacity: 1 → 0
duration: 100-150ms
easing: ease-in
```

### Sidebars / Drawers

**Slide In (from left)**
```
transform: translateX(-100%) → translateX(0)
duration: 250-300ms
easing: ease-out (or spring: stiffness 300, damping 25)
```

**Slide Out**
```
transform: translateX(0) → translateX(-100%)
duration: 200-250ms
easing: ease-in
```

### Toasts / Notifications

**Enter (from right)**
```
transform: translateX(100%) → translateX(0)
opacity: 0 → 1
duration: 250ms
easing: ease-out
```

**Exit**
```
opacity: 1 → 0
transform: translateX(0) → translateX(50%)
duration: 150ms
easing: ease-in
```

### Cards / List Items

**Hover Lift**
```
transform: translateY(0) → translateY(-4px)
box-shadow: subtle → elevated
duration: 200ms
easing: ease-out
```

**Expand/Collapse**
```
height: auto animation with overflow: hidden
duration: 200-300ms
easing: ease-out
```

### Skeleton Loaders

**Shimmer**
```
background: linear-gradient moving left to right
animation: shimmer 1.5s infinite
```

**Pulse**
```
opacity: 0.6 → 1 → 0.6
duration: 1.5-2s
easing: ease-in-out
iteration: infinite
```

---

## Choreography & Sequencing

### Stagger Pattern

Delay each child element progressively:
```
base delay: 30-50ms per item
max total delay: 300-500ms (cap for long lists)
```

**Example (5 items, 40ms stagger):**
```
Item 1: 0ms
Item 2: 40ms
Item 3: 80ms
Item 4: 120ms
Item 5: 160ms
```

### Cascade Direction

- **Top-down**: Navigation, vertical lists
- **Center-out**: Modals, alerts, focused content
- **Origin-based**: Ripples, radial menus (from click point)

### Orchestration Principles

1. **Lead with structure**: Container appears before content
2. **Group related elements**: Animate together or with minimal stagger
3. **Important last**: Key CTAs appear after supporting content
4. **Consistent rhythm**: Maintain regular timing intervals

### Page Transitions

**Shared Element Transition**
1. Clone source element
2. Animate clone to destination position/size
3. Fade in destination content
4. Remove clone

**Crossfade**
```
outgoing: opacity 1 → 0, 200ms
incoming: opacity 0 → 1, 200ms (starts at 100ms)
```

**Slide**
```
outgoing: translateX(0) → translateX(-30%), opacity 1 → 0
incoming: translateX(30%) → translateX(0), opacity 0 → 1
duration: 300ms each, offset by 50ms
```

---

## Reduced Motion

### Detection

**CSS**
```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

**JavaScript**
```js
const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
```

### Graceful Degradation

| Full Motion | Reduced Motion Alternative |
|-------------|---------------------------|
| Slide in | Instant appear or fade |
| Bounce | No bounce, direct position |
| Parallax scroll | Static positioning |
| Auto-play video | Paused with play button |
| Loading spinner | Static loading indicator |
| Complex choreography | Simple fade |

### What to Keep

Even with reduced motion, maintain:
- Color/opacity changes (fast: 100-150ms)
- Focus indicators (immediate)
- State changes (fast transition or instant)
- Progress indicators (static or minimal)

---

## Platform-Specific Curves

### Apple (iOS, macOS)

**System Curves**
```swift
// SwiftUI
.animation(.easeInOut)  // default
.animation(.spring(response: 0.5, dampingFraction: 0.8))

// UIKit
UIView.animate(withDuration: 0.3, delay: 0, options: .curveEaseInOut)
```

**Recommended Durations**
- Navigation push: 350ms
- Modal present: 300ms
- Keyboard: 250ms
- Spring response: 0.3-0.5

### Material Design (Android, Web)

**Standard Easing**
```
cubic-bezier(0.4, 0, 0.2, 1)
```

**Decelerate**
```
cubic-bezier(0, 0, 0.2, 1)
```

**Accelerate**
```
cubic-bezier(0.4, 0, 1, 1)
```

**Duration Tokens**
```
short1: 50ms   short2: 100ms  short3: 150ms  short4: 200ms
medium1: 250ms medium2: 300ms medium3: 350ms medium4: 400ms
long1: 450ms   long2: 500ms   long3: 550ms   long4: 600ms
```

### Windows (Fluent Design)

**Standard Easing**
```
cubic-bezier(0.8, 0, 0.2, 1)
```

**Accelerate**
```
cubic-bezier(0.9, 0.1, 1, 0.2)
```

**Decelerate**
```
cubic-bezier(0.1, 0.9, 0.2, 1)
```

**Durations**
- Fast: 83ms
- Normal: 167ms
- Slow: 250ms

---

## Quick Reference Card

### Most Common Animations

| Animation | Duration | Easing |
|-----------|----------|--------|
| Button hover | 150ms | ease-out |
| Button press | 100ms | ease-out |
| Dropdown open | 200ms | ease-out |
| Dropdown close | 150ms | ease-in |
| Modal open | 250ms | ease-out |
| Modal close | 200ms | ease-in |
| Tooltip show | 150ms | ease-out |
| Tooltip hide | 100ms | ease-in |
| Toast enter | 250ms | ease-out |
| Toast exit | 150ms | ease-in |
| Page fade | 200ms | ease-in-out |
| List stagger | 40ms/item | ease-out |

### Default Spring

For most interactive elements:
```
stiffness: 300, damping: 20
```
Or in React Spring: `tension: 300, friction: 20`
