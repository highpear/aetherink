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
- [x] Simplified practical pen color palette
- [x] Stroke width adjustment
- [x] Eraser size adjustment
- [x] Undo last stroke
- [x] Redo last undone change
- [x] Clear canvas
- [x] Save drawing as PNG
- [x] Quick save drawing as PNG
- [x] Copy drawing image to clipboard
- [x] Copy transparent canvas images over the captured screen background
- [x] Keyboard shortcuts for undo and clear
- [x] Conservative pen point filtering
- [x] Undo history capped at 100 snapshots

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
- [x] Persistent quick save and export preferences

Overlay workflow currently available:

- [x] Click-through mode on Windows
- [x] Overlay toggle shortcut while click-through is active
- [x] Temporary drawing while holding `Shift`
- [x] Shared click-through controller structure for Windows and macOS
- [x] macOS shortcut monitoring implementation for overlay toggle and temporary drawing
- [x] Click-through remains disabled when reliable shortcut monitoring is unavailable
- [x] Always-on-top is enabled automatically with click-through mode
- [x] Click-through starts disabled after launch for safe recovery

---

## Next Priorities

The overlay and export foundations are implemented. The next practical work is:

1. Add focused validation for overlay regressions.
2. Investigate canvas-only click-through while keeping overlay controls interactive.
3. Add a quick global shortcut to show or hide the overlay.
4. Define screenshot annotation mode around the existing background capture workflow.

---

## Decisions To Make Soon

These do not all require immediate implementation, but clarifying them early should reduce rework.

- [x] Define undo history behavior for stroke, clear, and erase actions
- [ ] Define undo history scope for future canvas actions
- [x] Establish conservative pen point filtering that preserves turns and limits redundant points
- [x] Keep the top bar readable and show overlay state in a floating status banner
- [x] Define the interaction model while click-through mode is active
- [ ] Investigate canvas-only click-through while keeping overlay controls interactive
- [ ] Decide how much platform-specific behavior should be normalized across Windows and macOS

---

## Overlay Follow-up

The overlay foundation exists, but platform behavior still needs tightening.

- [x] Verify macOS transparent window behavior in real usage
- [x] Verify macOS always-on-top / floating behavior in real usage
- [x] Keep click-through disabled if reliable shortcut monitoring is unavailable
- [x] Investigate Windows transparent window edge / shadow visibility in real usage
- [x] Define fallback behavior if global shortcut monitoring is unavailable
- [x] Review whether overlay status messaging should be visible outside the top bar

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
- [x] `Ctrl+Shift+O` enables click-through while focused and disables it globally without getting stuck
- [x] Overlay status text matches the actual current interaction mode
- [x] Focus returns correctly after leaving click-through mode

### 5. Restart / Persistence

- [x] Persisted overlay settings restore correctly after restarting the app on macOS, with click-through safely disabled

---

## Productivity Features

Useful after the core drawing workflow feels stable.

- [x] Reorganize the top bar so drawing, export, and overlay controls stay easy to scan
- [x] Toggle ink visibility without clearing strokes
- [ ] Screenshot annotation mode beyond the existing background PNG export
- [x] Define transparent canvas background PNG capture behavior
- [x] Investigate canvas rect to screen coordinate mapping while excluding the top bar
- [x] Add a tested helper for mapping the canvas rect to screen pixels
- [x] Add a shared image composition path for drawing ink over a captured background
- [x] Add the shared app and platform call path for background capture
- [x] Add an initial Windows screen-region capture backend
- [x] Add an initial macOS screen-region capture backend
- [x] Prototype background PNG export for the transparent canvas area
- [x] Validate whether AetherInk strokes are captured twice on Windows and macOS
- [x] Quick save directory setting for PNG export
- [x] One-click quick save to the selected export directory
- [ ] Quick global shortcut to show or hide the overlay
- [ ] Export stroke data

---

## Performance and Stability

Keep changes incremental and measurable.

- [x] Review stroke rendering cost as stroke count grows
- [x] Add focused unit validation for core drawing behavior
- [ ] Add focused validation for overlay regressions

---

## Release Preparation

- [ ] App icon
- [x] Windows build validation
- [x] macOS release build validation
- [x] README usage notes for overlay features
- [ ] First packaged release

---

## Later Ideas

- [ ] Laser pointer mode
- [ ] Shape tools
- [ ] Screen recording annotation
- [ ] Multi-layer drawing
- [ ] Collaborative drawing
