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
- [x] Clear canvas
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

---

## Next Priorities

These are the most useful next steps based on the current codebase and project priorities.

- [ ] Add conservative stroke smoothing or point filtering
- [ ] Improve pen selection UX beyond the basic color palette

---

## Overlay Follow-up

The overlay foundation exists, but platform behavior still needs tightening.

- [ ] Verify macOS transparent window behavior in real usage
- [ ] Verify macOS always-on-top / floating behavior in real usage
- [ ] Validate macOS click-through workflow end-to-end
- [ ] Define fallback behavior if global shortcut monitoring is unavailable
- [ ] Review whether overlay status messaging should be visible outside the top bar

---

## Productivity Features

Useful after the core drawing workflow feels stable.

- [ ] Screenshot annotation mode
- [ ] Quick global shortcut to show or hide the overlay
- [ ] Save drawing as PNG
- [ ] Export stroke data

---

## Performance and Stability

Keep changes incremental and measurable.

- [ ] Review stroke rendering cost as stroke count grows
- [ ] Limit unnecessary point density during fast dragging
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
