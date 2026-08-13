# Release Process

This document tracks the manual release process for AetherInk.

Keep the process small and repeatable. Do not add release-only feature work here; use this checklist to confirm that the current app is ready to package and publish.

## Pre-Release Checks

- Confirm the version in `Cargo.toml`.
- Confirm `Cargo.lock` is up to date.
- Run `cargo fmt --check`.
- Run `cargo test`.
- Run `cargo build --release`.
- Confirm the working tree only contains intended release changes.
- Review `README.md` for user-facing accuracy.
- Review `TODO.md` for any release-blocking items.
- Install `cargo-about` with `cargo install --locked cargo-about --features cli` if needed.
- Regenerate third-party notices with `cargo about generate --locked --fail -o THIRD_PARTY_LICENSES.html about.hbs`.
- Confirm the regenerated `THIRD_PARTY_LICENSES.html` has no unexpected changes.

## Manual Validation

Run these checks on each supported release platform when practical.

- Launch the app from a release build.
- Draw with the pen tool.
- Change pen color and pen width.
- Erase part of a stroke.
- Undo and redo changes.
- Clear the canvas.
- Toggle ink visibility.
- Toggle drawing on and off.
- Save a PNG with `Save PNG`.
- Save a PNG with `Quick Save`.
- Copy the canvas image to the clipboard.
- Enable background-aware copying, then copy a transparent canvas image with the screen background included.
- Save a transparent canvas with `Save Background PNG`.
- Switch between white and transparent canvas backgrounds.
- Adjust transparent canvas opacity.
- Toggle always-on-top mode.
- Toggle borderless window mode.
- Toggle transparent window background mode.
- Enable click-through mode.
- Confirm enabling click-through also enables always-on-top mode.
- Recover from click-through mode with the keyboard shortcut.
- Hold `Shift` to draw temporarily while click-through mode is active.
- Restart the app and confirm persisted settings restore correctly while click-through starts disabled.

## Platform Checks

Windows:

- Confirm the packaged app launches on Windows.
- Confirm click-through mode passes pointer input to the window behind AetherInk.
- Confirm transparent window and canvas behavior remains visually usable.
- Confirm background PNG export captures the expected screen area.
- Confirm background-aware clipboard copy captures the expected screen area.

macOS:

- Confirm the packaged app launches on macOS.
- Confirm click-through mode passes pointer input to the window behind AetherInk.
- Confirm `Shift` temporary drawing works while click-through mode is active.
- Confirm Screen Recording permission handling for background PNG export.
- Confirm background-aware clipboard copy uses the same permitted capture workflow.
- Confirm transparent window and canvas behavior remains visually usable.

## Packaging

- Build release artifacts for each supported platform.
- Package the app in the simplest suitable format for the release.
- Launch the app from the packaged output, not only from `cargo run`.
- Confirm packaged artifacts use the intended version.
- Include `LICENSE` and `THIRD_PARTY_LICENSES.html` in every packaged artifact.
- Confirm both license files are readable from the packaged output.
- Keep platform-specific packaging notes in this document as the process becomes clearer.

## GitHub Release

- Create a version tag that matches `Cargo.toml`.
- Upload the packaged release artifacts.
- Include short release notes with:
  - notable features
  - known limitations
  - supported platforms
  - basic usage notes or a link to `README.md`
- Mark the release as a prerelease if the app is still considered early MVP quality.

## Post-Release

- Download and launch the published artifact.
- Confirm the release notes and assets are visible.
- Create follow-up issues or TODO entries for any release findings.
