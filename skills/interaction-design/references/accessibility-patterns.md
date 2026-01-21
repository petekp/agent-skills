# Accessibility Patterns

ARIA patterns, focus management, and screen reader considerations for interaction design.

## Table of Contents
- [Focus Management](#focus-management)
- [Keyboard Navigation](#keyboard-navigation)
- [ARIA Patterns](#aria-patterns)
- [Screen Reader Considerations](#screen-reader-considerations)
- [Color & Contrast](#color--contrast)
- [Motion & Vestibular](#motion--vestibular)
- [Touch Accessibility](#touch-accessibility)
- [Testing Checklist](#testing-checklist)

---

## Focus Management

### Focus Visibility

**Requirements**
- Focus indicator must be visible on all interactive elements
- Contrast ratio: 3:1 minimum against adjacent colors
- Size: 2px outline minimum, or equivalent visual treatment

**Recommended Styles**
```css
:focus-visible {
  outline: 2px solid #005fcc;
  outline-offset: 2px;
}

/* High contrast mode */
@media (prefers-contrast: more) {
  :focus-visible {
    outline: 3px solid currentColor;
    outline-offset: 3px;
  }
}
```

**Avoid**
- `outline: none` without replacement indicator
- Focus styles that only change background color slightly
- Relying solely on color change (add outline or border)

### Focus Order

**Principles**
1. Follow visual layout (left-to-right, top-to-bottom in LTR)
2. Group related elements logically
3. Skip decorative/non-interactive elements
4. Maintain consistent order across interactions

**Common Issues**
- Modals that don't trap focus
- Dynamically inserted content that steals focus
- Off-screen elements that receive focus
- Incorrect tabindex values creating unexpected order

### Focus Trapping

Required for:
- Modals and dialogs
- Dropdown menus (while open)
- Popovers with interactive content

**Implementation Pattern**
```
1. Identify first and last focusable elements
2. On Tab from last element → focus first element
3. On Shift+Tab from first element → focus last element
4. On Escape → close and restore focus
```

### Focus Restoration

When closing overlays or removing content:
```
1. Store reference to trigger element before opening
2. On close, return focus to stored element
3. If trigger no longer exists, focus next logical element
```

### Managing Dynamic Content

**New Content Added**
- Don't auto-focus unless user-initiated action
- Use `aria-live` regions for announcements instead
- Exception: Modals should focus first element or close button

**Content Removed**
- If focused element removed, move focus to logical successor
- Don't leave focus in void (causes browser to reset to body)

---

## Keyboard Navigation

### Standard Patterns

| Key | Action |
|-----|--------|
| Tab | Move forward through focusable elements |
| Shift + Tab | Move backward |
| Enter | Activate button, submit form, follow link |
| Space | Activate button, toggle checkbox, select option |
| Escape | Close modal, cancel action, clear selection |
| Arrow Keys | Navigate within components (tabs, menus, sliders) |
| Home / End | Jump to first / last item |
| Page Up / Down | Scroll or large jumps |

### Roving Tabindex

For composite widgets (tabs, menus, toolbars):

**Concept**
- Only one item in group has `tabindex="0"` (receives Tab focus)
- Other items have `tabindex="-1"` (not in Tab order)
- Arrow keys move focus and update tabindex values

**Example: Tab List**
```html
<div role="tablist">
  <button role="tab" tabindex="0" aria-selected="true">Tab 1</button>
  <button role="tab" tabindex="-1" aria-selected="false">Tab 2</button>
  <button role="tab" tabindex="-1" aria-selected="false">Tab 3</button>
</div>
```

**Arrow Key Behavior**
- Arrow Right/Down: Move to next tab
- Arrow Left/Up: Move to previous tab
- Home: Move to first tab
- End: Move to last tab
- Wrap around at ends (optional)

### Skip Links

Allow keyboard users to bypass repetitive content:

```html
<a href="#main-content" class="skip-link">Skip to main content</a>

<style>
.skip-link {
  position: absolute;
  top: -40px;
  left: 0;
  z-index: 100;
}
.skip-link:focus {
  top: 0;
}
</style>
```

### Keyboard Shortcuts

**Principles**
- Don't override browser/OS shortcuts
- Document shortcuts (help panel or tooltip)
- Allow customization when possible
- Single-key shortcuts require ability to disable (WCAG 2.1.4)

**Common Conventions**
| Shortcut | Action |
|----------|--------|
| / or Cmd+K | Search / Command palette |
| ? | Show keyboard shortcuts |
| Escape | Close, cancel, deselect |
| Cmd+S | Save |
| Cmd+Z | Undo |
| Cmd+Shift+Z | Redo |

---

## ARIA Patterns

### Landmark Roles

```html
<header role="banner">...</header>
<nav role="navigation">...</nav>
<main role="main">...</main>
<aside role="complementary">...</aside>
<footer role="contentinfo">...</footer>
<form role="search">...</form>
```

Note: Use semantic HTML elements; roles are implicit. Add explicit roles only when element can't be used.

### Live Regions

Announce dynamic content changes:

**Attributes**
- `aria-live="polite"`: Announces after current speech
- `aria-live="assertive"`: Interrupts current speech (use sparingly)
- `aria-atomic="true"`: Announces entire region, not just changes
- `aria-relevant`: What changes to announce (additions, removals, text)

**Common Use Cases**
```html
<!-- Status messages -->
<div role="status" aria-live="polite">3 items in cart</div>

<!-- Error alerts -->
<div role="alert" aria-live="assertive">Form submission failed</div>

<!-- Loading status -->
<div aria-live="polite" aria-busy="true">Loading results...</div>
```

### Modal Dialog

```html
<div role="dialog" aria-modal="true" aria-labelledby="dialog-title">
  <h2 id="dialog-title">Confirm Delete</h2>
  <p>Are you sure you want to delete this item?</p>
  <button>Cancel</button>
  <button>Delete</button>
</div>
```

**Requirements**
- `role="dialog"` or `role="alertdialog"` (for urgent messages)
- `aria-modal="true"` indicates background is inert
- `aria-labelledby` points to visible title
- Focus trapped within dialog
- Escape closes dialog

### Tabs

```html
<div role="tablist" aria-label="Settings">
  <button role="tab" aria-selected="true" aria-controls="panel1" id="tab1">
    General
  </button>
  <button role="tab" aria-selected="false" aria-controls="panel2" id="tab2" tabindex="-1">
    Privacy
  </button>
</div>

<div role="tabpanel" id="panel1" aria-labelledby="tab1">
  <!-- General settings content -->
</div>

<div role="tabpanel" id="panel2" aria-labelledby="tab2" hidden>
  <!-- Privacy settings content -->
</div>
```

### Menu

```html
<button aria-haspopup="menu" aria-expanded="false">
  Actions
</button>

<ul role="menu" hidden>
  <li role="menuitem">Edit</li>
  <li role="menuitem">Duplicate</li>
  <li role="separator"></li>
  <li role="menuitem">Delete</li>
</ul>
```

**Behavior**
- Toggle `aria-expanded` on trigger
- Arrow Down on trigger opens and focuses first item
- Arrow keys navigate items
- Enter/Space activates item
- Escape closes and returns focus to trigger

### Combobox (Autocomplete)

```html
<label for="search">Search</label>
<input
  type="text"
  id="search"
  role="combobox"
  aria-autocomplete="list"
  aria-expanded="false"
  aria-controls="search-results"
  aria-activedescendant=""
>

<ul role="listbox" id="search-results" hidden>
  <li role="option" id="opt1">Result 1</li>
  <li role="option" id="opt2">Result 2</li>
</ul>
```

**Behavior**
- Update `aria-expanded` when listbox shows/hides
- Update `aria-activedescendant` to currently highlighted option
- Arrow keys move through options
- Enter selects highlighted option
- Escape closes listbox

### Disclosure (Expand/Collapse)

```html
<button aria-expanded="false" aria-controls="content">
  Show details
</button>
<div id="content" hidden>
  <!-- Expandable content -->
</div>
```

### Switch/Toggle

```html
<button role="switch" aria-checked="false">
  Dark mode
</button>
```

Alternative with checkbox:
```html
<label>
  <input type="checkbox" role="switch">
  Dark mode
</label>
```

### Slider

```html
<label for="volume">Volume</label>
<input
  type="range"
  id="volume"
  min="0"
  max="100"
  value="50"
  aria-valuetext="50 percent"
>
```

For custom sliders:
```html
<div
  role="slider"
  aria-label="Volume"
  aria-valuemin="0"
  aria-valuemax="100"
  aria-valuenow="50"
  aria-valuetext="50 percent"
  tabindex="0"
>
```

---

## Screen Reader Considerations

### Text Alternatives

**Images**
- Decorative: `alt=""` (empty, not missing)
- Informative: Describe content/function
- Functional (buttons/links): Describe action, not appearance

**Icons**
```html
<!-- Icon-only button -->
<button aria-label="Close">
  <svg aria-hidden="true">...</svg>
</button>

<!-- Icon with text -->
<button>
  <svg aria-hidden="true">...</svg>
  Save
</button>
```

### Hidden Content

**Visually Hidden (Screen Reader Only)**
```css
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
```

**Hidden from Screen Readers**
```html
<div aria-hidden="true">Decorative content</div>
```

### Announcements

**Status Updates**
```html
<div role="status" class="sr-only">
  Search complete. 5 results found.
</div>
```

**Loading States**
```html
<button aria-busy="true" aria-describedby="loading-status">
  Submit
</button>
<span id="loading-status" role="status">Processing...</span>
```

### Form Errors

```html
<label for="email">Email</label>
<input
  type="email"
  id="email"
  aria-invalid="true"
  aria-describedby="email-error"
>
<span id="email-error" role="alert">
  Please enter a valid email address
</span>
```

### Tables

```html
<table>
  <caption>Monthly Sales Report</caption>
  <thead>
    <tr>
      <th scope="col">Month</th>
      <th scope="col">Revenue</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <th scope="row">January</th>
      <td>$10,000</td>
    </tr>
  </tbody>
</table>
```

---

## Color & Contrast

### Minimum Ratios (WCAG AA)

| Element | Ratio |
|---------|-------|
| Normal text (< 18pt / 14pt bold) | 4.5:1 |
| Large text (≥ 18pt / 14pt bold) | 3:1 |
| UI components (borders, icons) | 3:1 |
| Focus indicators | 3:1 |

### WCAG AAA (Enhanced)

| Element | Ratio |
|---------|-------|
| Normal text | 7:1 |
| Large text | 4.5:1 |

### Don't Rely on Color Alone

Bad:
```
Required fields are marked in red.
```

Good:
```
Required fields are marked with an asterisk (*).
```

**Patterns**
- Use icons + color for status (✓ Success, ✗ Error)
- Add patterns to charts, not just colors
- Underline links, don't rely on color difference

### High Contrast Mode

```css
@media (prefers-contrast: more) {
  :root {
    --text-color: #000;
    --bg-color: #fff;
    --border-color: #000;
  }

  button {
    border: 2px solid var(--border-color);
  }
}
```

---

## Motion & Vestibular

### Respecting User Preferences

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

### What Triggers Vestibular Issues

- Large-scale motion (parallax scrolling)
- Zoom animations covering significant screen area
- Spinning or rotating animations
- Rapid flashing or blinking
- Auto-playing video with motion

### Safe Motion

Even with reduced motion preference:
- Opacity fades (fast, < 150ms)
- Small-scale transitions (color, border)
- Static loading indicators
- User-initiated animations with pause control

### Animation Controls

- Provide pause/stop for auto-playing content
- Allow disabling animations in settings
- Don't auto-play motion on page load

---

## Touch Accessibility

### Target Sizes

| Standard | Minimum Size |
|----------|--------------|
| WCAG 2.5.5 (AAA) | 44×44 CSS pixels |
| WCAG 2.5.8 (AA, 2.2) | 24×24 CSS pixels |
| Apple HIG | 44×44 points |
| Material Design | 48×48 dp |

### Spacing

- Minimum 8px between adjacent targets
- Inline targets (text links): provide adequate padding

### Touch Alternatives

- Provide tap alternatives for gestures (swipe actions need visible buttons)
- Long-press actions should have menu alternatives
- Drag operations need keyboard alternatives

### Pointer Cancellation

Users should be able to cancel accidental touches:
- Action on up-event (touch release), not down-event
- Moving finger off target before release cancels action
- Exception: Immediate feedback for drawing, games

---

## Testing Checklist

### Keyboard Testing

- [ ] All interactive elements reachable via Tab
- [ ] Focus order matches visual layout
- [ ] Focus indicator visible on all elements
- [ ] Enter/Space activates buttons and controls
- [ ] Escape closes overlays and cancels actions
- [ ] Arrow keys work in composite widgets
- [ ] No keyboard traps (except intentional modal traps)

### Screen Reader Testing

- [ ] All images have appropriate alt text
- [ ] Form fields have associated labels
- [ ] Error messages are announced
- [ ] Dynamic content updates announced via live regions
- [ ] Modals announce on open, trap focus
- [ ] Interactive elements have accessible names
- [ ] Landmarks help navigation

### Visual Testing

- [ ] Text contrast meets 4.5:1 (AA) or 7:1 (AAA)
- [ ] UI component contrast meets 3:1
- [ ] Focus indicators visible and high contrast
- [ ] Information not conveyed by color alone
- [ ] Content readable at 200% zoom
- [ ] No horizontal scroll at 320px width

### Motion Testing

- [ ] Animations respect prefers-reduced-motion
- [ ] No content flashes more than 3 times per second
- [ ] Auto-playing motion can be paused
- [ ] Parallax and large-scale motion can be disabled

### Touch Testing

- [ ] Touch targets at least 44×44 pixels
- [ ] Adequate spacing between targets
- [ ] Gestures have tap alternatives
- [ ] Actions can be cancelled (pointer up, not down)

### Tools

**Automated**
- axe DevTools (browser extension)
- Lighthouse accessibility audit
- WAVE evaluation tool

**Manual**
- VoiceOver (macOS/iOS)
- NVDA (Windows, free)
- JAWS (Windows)
- TalkBack (Android)
- Keyboard-only navigation

**Contrast Checkers**
- WebAIM Contrast Checker
- Colour Contrast Analyser (desktop app)
- Figma plugins (Stark, Contrast)
