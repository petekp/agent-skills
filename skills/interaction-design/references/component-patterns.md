# Component Interaction Patterns

Detailed interaction patterns for common UI components.

## Table of Contents
- [Forms & Inputs](#forms--inputs)
- [Modals & Dialogs](#modals--dialogs)
- [Dropdowns & Menus](#dropdowns--menus)
- [Drag & Drop](#drag--drop)
- [Lists & Tables](#lists--tables)
- [Navigation](#navigation)
- [Search](#search)
- [Notifications & Toasts](#notifications--toasts)
- [Tooltips & Popovers](#tooltips--popovers)
- [Sliders & Range Inputs](#sliders--range-inputs)
- [Date & Time Pickers](#date--time-pickers)

---

## Forms & Inputs

### Text Inputs

**States**
| State | Appearance | Behavior |
|-------|------------|----------|
| Empty | Placeholder visible, subtle border | — |
| Focused | Ring/outline, placeholder fades or moves | Cursor blinks |
| Filled | Value visible, clear button optional | — |
| Error | Red border, error message below | Focus on first error |
| Disabled | Reduced opacity, no cursor change | Ignore interaction |
| Read-only | Normal appearance, no edit affordance | Can select/copy |

**Validation Timing**
- Validate on blur (not on every keystroke)
- Re-validate on change after an error is shown
- Show inline errors immediately below the field
- Don't clear user input on error

**Label Patterns**
- Always use visible labels (not placeholder-only)
- Float label: moves above input on focus/fill
- Static label: always above input
- Inline label: inside input, left-aligned (for short labels)

**Keyboard**
- Tab: move to next field
- Shift+Tab: move to previous field
- Enter: submit form (if single-line) or move to next field
- Escape: blur field, optionally revert changes

### Textareas

- Auto-grow: expand height as content grows (with max-height)
- Character count: show remaining/total for limited fields
- Resize handle: bottom-right corner, respect min/max dimensions
- Line numbers: optional for code inputs

### Checkboxes & Radio Buttons

**Checkbox States**
- Unchecked, Checked, Indeterminate (for parent of mixed children)
- Focus ring on the control, not the label
- Click anywhere on label to toggle

**Radio Groups**
- Arrow keys navigate between options
- Only one radio in group receives tab focus
- Selection follows focus (or requires explicit activation)

### Form Layout

**Single Column** (default for most forms)
- Labels above inputs for scanning
- Group related fields with subtle dividers or spacing
- Primary action button aligned left (right in RTL)

**Multi-Column** (dense data entry)
- Use CSS Grid for alignment
- Keep related fields in same row
- Ensure logical tab order (left-to-right, top-to-bottom)

**Inline Forms** (search, filters)
- Label as placeholder or icon
- Submit on Enter or explicit button
- Compact spacing

---

## Modals & Dialogs

### Opening

- Animate from trigger element (scale + fade) or center (fade + slide up)
- Duration: 200-300ms ease-out
- Focus first interactive element or close button
- Prevent body scroll (use `overflow: hidden` or scroll lock)

### Focus Management

```
1. Save previous focus location
2. Move focus into modal (first focusable or autofocus element)
3. Trap focus within modal (Tab cycles through modal elements)
4. On close, restore focus to trigger element
```

### Closing

- Close button (top-right or within footer)
- Escape key (unless destructive action in progress)
- Backdrop click (configurable; avoid for important confirmations)
- Animate out: reverse of open animation, slightly faster (150-250ms)

### Dialog Types

| Type | Use Case | Close Behavior |
|------|----------|----------------|
| Alert | System message, acknowledgment | Single button, no backdrop close |
| Confirm | Destructive action confirmation | Two buttons, no backdrop close |
| Prompt | Single input collection | Submit/Cancel, Enter submits |
| Form | Multi-field data entry | Submit/Cancel, warn on unsaved changes |
| Drawer | Side panel for details/settings | Backdrop close, swipe to dismiss (mobile) |

### Stacking Modals

Avoid when possible. If necessary:
- Each modal gets its own backdrop (dimmer for each layer)
- Escape closes topmost modal only
- Maintain focus trap in topmost modal

### Responsive Behavior

- Full-screen on mobile (< 480px width)
- Centered with max-width on tablet/desktop
- Bottom sheet variant for mobile-first actions

---

## Dropdowns & Menus

### Activation

- Click to open (not hover—accessibility requirement)
- Enter/Space also opens when trigger is focused
- Arrow Down opens and selects first item

### Keyboard Navigation

| Key | Action |
|-----|--------|
| Arrow Down | Move to next item |
| Arrow Up | Move to previous item |
| Home | Move to first item |
| End | Move to last item |
| Enter/Space | Select current item |
| Escape | Close menu, return focus to trigger |
| Type character | Jump to item starting with that character |

### Positioning

- Open below trigger by default
- Flip to above if insufficient space below
- Flip horizontally for edge collision
- Maintain minimum distance from viewport edges (8px)

### Menu Item States

| State | Appearance |
|-------|------------|
| Default | Normal text |
| Hover/Focus | Background highlight |
| Selected | Checkmark or filled indicator |
| Disabled | Reduced opacity, skip in keyboard nav |
| Destructive | Red text (delete, remove) |

### Nested Menus (Submenus)

- Open on hover with 200ms delay (prevents accidental triggers)
- Arrow Right opens submenu
- Arrow Left closes submenu, returns to parent
- Maintain safe triangle for diagonal mouse movement

### Select Dropdowns

- Show current selection in trigger
- Searchable: add input at top for filtering
- Multi-select: checkboxes, keep open after selection
- Clear all: button to reset selection

---

## Drag & Drop

### Affordances

- Drag handle: grip icon (⋮⋮) or entire item
- Cursor: `grab` on hover, `grabbing` during drag
- Slight lift on drag start (shadow + scale 1.02-1.05)

### During Drag

**Ghost/Preview**
- Semi-transparent copy following cursor
- Or: empty placeholder at original position

**Drop Zones**
- Highlight valid drop targets
- Show insertion indicator (line between items)
- Dim/disable invalid drop zones

**Scrolling**
- Auto-scroll when dragging near container edges
- Accelerate scroll speed near very edge

### Drop Feedback

- Animate item into new position (200-300ms ease-out)
- Remove ghost/preview immediately
- Briefly highlight dropped item (flash or pulse)

### Keyboard Alternative

Required for accessibility:
- Select item with Space
- Arrow keys to move through positions
- Space/Enter to drop at current position
- Escape to cancel

### Multi-Item Drag

- Show count badge on ghost (e.g., "3 items")
- Maintain visual stack of selected items
- Apply action to all selected items on drop

---

## Lists & Tables

### Selection

**Single Select**
- Click to select (deselects previous)
- Arrow keys move selection
- Enter activates/opens selected item

**Multi-Select**
- Click toggles selection
- Shift+Click for range selection
- Cmd/Ctrl+Click for non-contiguous selection
- Checkbox column for explicit multi-select

### Sorting

- Click column header to sort
- Click again to reverse sort direction
- Visual indicator (arrow) shows sort column and direction
- Tertiary click resets to default sort

### Filtering

- Filter controls above or beside list
- Instant filtering (debounce input: 150-300ms)
- Show count of filtered results
- Clear all filters button when filters active

### Infinite Scroll / Virtualization

- Load more items when 2-3 screens from bottom
- Show loading indicator at bottom during fetch
- Virtualize: only render visible items + buffer
- Maintain scroll position on data updates

### Empty States

| State | Content |
|-------|---------|
| Initial empty | Illustration + "No items yet" + CTA to add |
| No results (filter) | "No matches" + suggest clearing filters |
| Error | Error message + retry button |

### Row Actions

- Hover actions: appear on row hover (icon buttons)
- Context menu: right-click or overflow (⋮) button
- Swipe actions: reveal on horizontal swipe (mobile)
- Inline edit: click cell to edit

---

## Navigation

### Tab Navigation

- Tabs as `role="tablist"` with `role="tab"` children
- Arrow keys move between tabs
- Tab/Shift+Tab exits tab list
- Active tab: `aria-selected="true"`, visual indicator (underline, background)

### Breadcrumbs

- Current page not a link (text only)
- Separator between items (/ or >)
- Truncate middle items on overflow with "..."
- Click any item to navigate to that level

### Pagination

- Show current page and total pages
- Previous/Next buttons (disabled at bounds)
- Direct page input for large page counts
- Jump to first/last buttons optional

### Sidebar Navigation

- Collapsible: icon-only or full labels
- Nested items: expand/collapse with disclosure arrow
- Current item: strong visual indicator (background, border)
- Keyboard: Arrow keys navigate, Enter activates

---

## Search

### Search Input

- Magnifying glass icon (left side)
- Clear button (right side, appears when filled)
- Escape clears input and closes results

### Instant Search

- Debounce input (150-300ms)
- Show loading indicator in results area
- Minimum characters before searching (2-3) to avoid noise
- Highlight matching text in results

### Search Results

- Show result count
- Group by category if multiple types
- Recent searches / suggestions when input empty
- No results: suggestions for different queries

### Command Palette

- Open with Cmd/Ctrl+K
- Fuzzy matching on command names
- Recent commands at top
- Category grouping with headers
- Keyboard-first: arrows + Enter to execute

---

## Notifications & Toasts

### Positioning

- Top-right or bottom-right (most common)
- Top-center for important alerts
- Stack vertically with gap (8-12px)
- New notifications push others down

### Auto-Dismiss

| Type | Duration | Auto-dismiss |
|------|----------|--------------|
| Success | 3-5 seconds | Yes |
| Info | 5-7 seconds | Yes |
| Warning | 7-10 seconds | Optional |
| Error | Persistent | No (require dismiss) |

### Interaction

- Hover pauses auto-dismiss timer
- Close button always visible
- Action button (e.g., "Undo", "View")
- Swipe to dismiss (mobile)

### Animation

- Enter: slide in from edge + fade (200-300ms)
- Exit: fade out + slide out (150-200ms)
- Stack reflow: animate position changes

---

## Tooltips & Popovers

### Tooltips

- Show on hover after 300-500ms delay
- Hide immediately on mouse leave
- Position: above element by default, flip if clipped
- Content: text only, 1-2 short lines
- Keyboard: show on focus for focusable elements
- Touch: show on long-press (or avoid tooltips, use explicit labels)

### Popovers

- Trigger: click (not hover)
- Can contain interactive elements
- Dismiss: click outside, Escape, or explicit close
- Arrow pointing to trigger element
- Focus trap if contains form elements

---

## Sliders & Range Inputs

### Single Value Slider

- Thumb: 20-24px touch target minimum
- Track: filled portion shows progress
- Value label: above thumb or in tooltip during drag
- Tick marks: optional for discrete values

### Range Slider (Two Thumbs)

- Both thumbs independently draggable
- Thumbs cannot cross each other
- Filled track between thumbs

### Keyboard

- Arrow Left/Right: decrease/increase by step
- Page Up/Down: larger jumps (10 steps)
- Home/End: min/max values
- Focus indicator on thumb

### Accessibility

- `role="slider"` with `aria-valuemin`, `aria-valuemax`, `aria-valuenow`
- `aria-valuetext` for human-readable value (e.g., "$500")
- Announce value changes to screen readers

---

## Date & Time Pickers

### Date Picker

**Input States**
- Text input with date format hint (MM/DD/YYYY)
- Calendar icon triggers picker
- Support manual text entry with validation

**Calendar View**
- Month/year header with navigation arrows
- Day grid with weekday headers
- Today highlighted
- Selected date(s) filled
- Disabled dates dimmed (past dates, unavailable)

**Keyboard**
- Arrow keys navigate days
- Page Up/Down: previous/next month
- Home/End: first/last day of month
- Enter: select date and close

### Date Range Picker

- Two inputs or single input with separator
- Visual range highlight in calendar
- Click start date, then end date
- Presets: "Last 7 days", "This month", etc.

### Time Picker

- Input with format hint (HH:MM AM/PM)
- Dropdown with time options (15-30 min intervals)
- Allow manual entry for precise times
- Clock face picker optional (higher touch accuracy)

### Combined DateTime

- Date picker followed by time picker
- Or unified calendar with time selection below
- Clear indication of timezone if relevant
