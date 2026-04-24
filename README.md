# AetherInk

A lightweight desktop overlay drawing app for Windows and macOS.

## Goals

- Fast, simple freehand drawing for screen annotation
- A clean overlay canvas for presentations, screen sharing, and brainstorming
- Transparent and always-on-top workflows that stay out of the way
- Cross-platform desktop support for Windows and macOS

## Current Status

AetherInk is currently an early MVP focused on stable freehand drawing and the first usable overlay workflow.

## Current Features

Drawing

- Freehand drawing with mouse drag
- Pen and eraser tools
- Tool-specific canvas cursor feedback
- Adjustable pen color
- Adjustable pen width
- Adjustable eraser size
- Basic color palette
- Undo last stroke
- Redo last undone change
- Clear canvas button
- Save the current canvas as a PNG file
- Quick save the current canvas as a PNG file
- Keyboard shortcuts for undo and clear

Canvas and window

- Toggle drawing mode
- White / transparent canvas background toggle
- Adjustable transparent background opacity
- Transparent canvas border visibility setting
- Always-on-top window toggle
- Borderless window toggle
- Transparent window background toggle
- Settings window
- Persistent canvas and overlay settings

Overlay workflow

- Click-through mode on Windows
- Overlay toggle shortcut while click-through is active
- Temporary drawing while click-through is enabled
- Floating overlay status while click-through is active
- Click-through remains disabled on platforms where reliable shortcut monitoring is unavailable

## How to Use

1. Launch the app and drag on the canvas to draw.
2. Use the top bar to switch between the pen and eraser tools.
3. Adjust pen color, pen width, or eraser size from the top bar.
4. Toggle the `Draw: On` / `Draw: Off` control if you want to pause or resume editing.
5. Open `Settings` to switch between white and transparent canvas modes and adjust overlay behavior.
6. Enable `Always on top` when you want to keep the canvas above other windows.
7. Use `Save PNG` to choose where to export the current canvas as a PNG file.
8. Set a quick save folder in `Settings`, then use `Quick Save` to export without opening a save dialog.
9. On supported platforms, enable click-through mode when you want the overlay to stay visible without intercepting normal mouse input.

## Shortcuts

- `Ctrl+Z`: Undo the last stroke
- `Ctrl+S` or `Cmd+S`: Open the PNG save dialog
- `Ctrl+Shift+S` or `Cmd+Shift+S`: Quick save a PNG to the selected quick save folder
- `Ctrl+Shift+Z` or `Ctrl+Y`: Redo the last undone change
- `Ctrl+Shift+C`: Clear the canvas
- `Ctrl+Delete`: Clear the canvas
- `Ctrl+Shift+O`: Toggle overlay click-through mode while shortcut monitoring is available
- `Shift` hold: Temporarily draw while click-through mode is active

## Settings Overview

- `Background`: Switch between a white canvas and a transparent canvas
- `Opacity`: Adjust visible white tint when the canvas background is transparent
- `Border`: Control when the transparent canvas border is shown
- `Enable drawing`: Pause or resume mouse drawing
- `Always on top`: Keep the window above other windows
- `Borderless window`: Hide standard window decorations
- `Transparent window background`: Blend the window chrome and panel background into the desktop
- `Quick save folder`: Choose where one-click PNG exports are written
- `Click-through mode`: Pass mouse input through the overlay when supported and safe to enable

## PNG Export

- `Save PNG` exports the current canvas area only, without the top bar or cursor preview.
- `Quick Save` writes a timestamped PNG to the selected quick save folder without opening a dialog.
- The PNG background follows the current canvas background setting, including transparent canvas opacity.
- The save dialog suggests a timestamped name such as `aetherink-canvas-20260418-173015.png`.
- If you omit the file extension when saving, AetherInk automatically appends `.png`.
- After a successful export, AetherInk remembers that folder and uses it as the default save location next time.

## Platform Notes

- Windows currently has the most complete overlay workflow, including click-through mode and keyboard-driven return to drawing.
- macOS click-through, temporary drawing, transparent window behavior, and persistence have been validated in real use.
- Click-through is intentionally kept disabled on platforms where shortcut monitoring is unavailable, because the app must always provide a reliable way to return from pointer passthrough.
- The project is intentionally keeping the feature set small until the base drawing and overlay interactions feel reliable.

## Near-Term Focus

- Conservative stroke smoothing or point filtering
- Improved pen selection UX beyond the basic color palette
- macOS overlay behavior validation
- Export and save features after the drawing workflow is stable

## Tech Stack

- Rust
- egui / eframe
