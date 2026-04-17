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
- Clear canvas button
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
- Click-through remains disabled on platforms where reliable shortcut monitoring is unavailable

## How to Use

1. Launch the app and drag on the canvas to draw.
2. Use the top bar to switch between the pen and eraser tools.
3. Adjust pen color, pen width, or eraser size from the top bar.
4. Toggle the `Draw: On` / `Draw: Off` control if you want to pause or resume editing.
5. Open `Settings` to switch between white and transparent canvas modes and adjust overlay behavior.
6. Enable `Always on top` when you want to keep the canvas above other windows.
7. On supported platforms, enable click-through mode when you want the overlay to stay visible without intercepting normal mouse input.

## Shortcuts

- `Ctrl+Z`: Undo the last stroke
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
- `Click-through mode`: Pass mouse input through the overlay when supported and safe to enable

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
