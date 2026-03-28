# AetherInk TODO

AetherInk is a lightweight overlay drawing tool for Windows and macOS.

---

## Phase 1 — Core Drawing (MVP)

Basic drawing functionality.

- [x] Create project structure
- [x] Implement canvas drawing with mouse drag
- [x] Draw black lines
- [x] Clear canvas button

Next improvements:

- [x] Undo last stroke
- [x] Adjustable pen width
- [x] Adjustable pen color
- [ ] Eraser tool
- [ ] Stroke smoothing

---

## Phase 2 — Canvas Features

Improve drawing usability.

- [x] Transparent canvas background
- [x] Toggle canvas background (white / transparent)
- [x] Adjustable transparent background opacity
- [x] Toggle drawing mode
- [x] Keyboard shortcut for clear
- [x] Keyboard shortcut for undo
- [ ] Cursor indicator for pen

---

## Phase 3 — Overlay Mode

Core feature of AetherInk.

- [x] Always-on-top window
- [x] Borderless window mode
- [x] Transparent window background
- [x] Click-through mode (Windows MVP)
- [x] Keyboard shortcut for click-through mode
- [x] Temporary drawing while holding a key

Example workflow:
Normal mode → click-through
Hold key → draw
Release key → click-through

---

## Phase 4 — Platform Integration

OS-specific behavior.

### Windows

- [x] Window transparency
- [x] Click-through window style
- [x] Always-on-top control

### macOS

- [ ] Transparent window
- [ ] Floating window level
- [ ] Click-through behavior

---

## Phase 5 — Productivity Features

Useful tools for meetings and brainstorming.

- [ ] Screenshot annotation mode
- [ ] Quick toggle overlay shortcut
- [ ] Save drawing as PNG
- [ ] Export strokes

---

## Phase 6 — UI Improvements

Improve user experience.

- [ ] Minimal toolbar
- [x] Settings window
- [x] Persistent canvas background setting
- [x] Persistent transparent background opacity setting
- [ ] Persistent default pen color setting
- [ ] Persistent default pen width setting
- [ ] Pen selection
- [x] Color palette
- [x] Pen size slider
- [x] Transparent canvas border visibility setting (always show / show near edges only)

---

## Phase 7 — Performance

- [ ] Optimize stroke rendering
- [ ] Limit point density
- [ ] GPU rendering improvements

---

## Phase 8 — Distribution

Prepare for release.

- [ ] App icon
- [ ] Windows build
- [ ] macOS build
- [ ] GitHub release
- [ ] Documentation

---

## Future Ideas

- [ ] Screen recording annotation
- [ ] Laser pointer mode
- [ ] Shape tools
- [ ] Multi-layer drawing
- [ ] Collaborative drawing
