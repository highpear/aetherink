# AetherInk TODO

AetherInk is a lightweight overlay drawing tool for Windows and macOS.

This file tracks the current implementation status and the next practical steps for the MVP.

---

## Current Status

Drawing currently available:

- [x] Basic project structure
- [x] Freehand drawing with mouse drag
- [x] Pen and eraser tools
- [x] Stroke color selection
- [x] Stroke width adjustment
- [x] Eraser size adjustment
- [x] Undo last stroke
- [x] Redo last undone change
- [x] Clear canvas
- [x] Save drawing as PNG
- [x] Keyboard shortcuts for undo and clear

Canvas and window controls currently available:

- [x] Drawing enable / disable toggle
- [x] White / transparent canvas background toggle
- [x] Adjustable transparent canvas opacity
- [x] Transparent canvas border visibility setting
- [x] Always-on-top toggle
- [x] Borderless window toggle
- [x] Transparent window background toggle
- [x] Settings window
- [x] Persistent canvas and overlay settings

Overlay workflow currently available:

- [x] Click-through mode on Windows
- [x] Overlay toggle shortcut while click-through is active
- [x] Temporary drawing while holding `Shift`
- [x] Shared click-through controller structure for Windows and macOS
- [x] macOS shortcut monitoring implementation for overlay toggle and temporary drawing
- [x] Click-through remains disabled when reliable shortcut monitoring is unavailable

---

## Next Priorities

These are the most useful next steps based on the current codebase and project priorities.

- [x] Validate macOS click-through workflow end-to-end
- [x] Verify macOS transparent window behavior in real usage

---

## Overlay Follow-up

The overlay foundation exists, but platform behavior still needs tightening.

- [x] Verify macOS transparent window behavior in real usage
- [x] Verify macOS always-on-top / floating behavior in real usage
- [x] Keep click-through disabled if reliable shortcut monitoring is unavailable
- [ ] Review whether overlay status messaging should be visible outside the top bar

## macOS Validation Checklist

Run these checks in order when validating the current overlay workflow on macOS.

### 1. Basic Drawing

- [x] Basic drawing works with mouse drag in white canvas mode
- [x] Pen, eraser, undo, and clear actions behave as expected
- [x] Basic drawing also works in transparent canvas mode

### 2. Transparent Canvas

- [x] Transparent canvas opacity changes are reflected immediately
- [x] Transparent canvas border visibility modes remain understandable on transparent backgrounds
- [x] Transparent window background makes the app window visually blend without corrupting stroke rendering
- [x] Transparent window background does not make the top bar unreadable

### 3. Window Behavior

- [x] Always-on-top keeps the window above normal app windows during practical use
- [x] Borderless window still allows reliable window dragging from the top bar
- [x] No obvious macOS-specific issues appear across multiple desktops or fullscreen app transitions

### 4. Click-Through Overlay

- [x] Click-through mode can be enabled from settings while not actively drawing
- [x] Click-through mode actually passes pointer input through to the app behind the overlay
- [x] `Shift` temporarily restores drawing while click-through mode is active
- [x] Releasing `Shift` reliably returns the app to click-through mode
- [x] `Ctrl+Shift+O` toggles overlay click-through mode on and off without getting stuck
- [x] Overlay status text matches the actual current interaction mode
- [x] Focus returns correctly after leaving click-through mode

### 5. Restart / Persistence

- [x] Persisted overlay settings restore correctly after restarting the app on macOS

---

## Productivity Features

Useful after the core drawing workflow feels stable.

- [ ] Screenshot annotation mode
- [ ] Quick global shortcut to show or hide the overlay
- [ ] Export stroke data

---

## Performance and Stability

Keep changes incremental and measurable.

- [ ] Review stroke rendering cost as stroke count grows
- [ ] Add focused validation for drawing and overlay regressions

---

## Release Preparation

- [ ] App icon
- [ ] Windows build validation
- [ ] macOS build validation
- [ ] README usage notes for overlay features
- [ ] First packaged release

---

## Later Ideas

- [ ] Laser pointer mode
- [ ] Shape tools
- [ ] Screen recording annotation
- [ ] Multi-layer drawing
- [ ] Collaborative drawing
