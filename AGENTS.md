# AGENTS.md

## Project Overview

AetherInk is a lightweight desktop overlay drawing app for Windows and macOS.

Primary goals:

- fast and simple freehand drawing
- future transparent overlay mode
- future click-through mode
- usability for screen sharing, presentations, and brainstorming

Current stage:

- early MVP
- black line drawing with mouse drag
- clear canvas button
- project structure under active development

---

## Language Policy

Use English for all repository outputs, including:

- source code
- comments
- commit messages
- pull request text
- issue text
- TODOs
- documentation
- file names

The user may communicate in Japanese in chat, but repository artifacts must remain in English unless explicitly requested otherwise.

---

## Engineering Priorities

When making decisions, prioritize in this order:

1. Correctness
2. Simplicity
3. Maintainability
4. Cross-platform compatibility
5. Performance
6. Visual polish

Do not introduce unnecessary abstractions too early.
Prefer small, understandable modules over complex architectures.

---

## Product Priorities

For this project, prioritize features in this order:

1. Stable drawing behavior
2. Clean internal structure
3. Transparent canvas support
4. Always-on-top behavior
5. Click-through overlay mode
6. Global hotkeys
7. Export / persistence
8. UI polish

Do not jump ahead to advanced platform integration if the basic drawing workflow is unstable.

---

## Current Architecture

Current expected structure:

- `src/main.rs`  
  App entry point

- `src/app.rs`  
  Main app state and top-level UI

- `src/canvas.rs`  
  Drawing canvas logic

- `src/stroke.rs`  
  Stroke data model

- `src/platform/`  
  OS-specific integration code for Windows and macOS

Future modularization may introduce:

- `overlay/`
- `hotkeys/`
- `persistence/`

Do not create these modules unless the implementation actually requires them.

---

## Coding Rules

- Use Rust idiomatically.
- Keep functions small and focused.
- Prefer explicit code over clever code.
- Avoid premature generalization.
- Avoid adding dependencies unless they provide clear value.
- Keep platform-specific code isolated under `src/platform/`.
- Keep UI logic separate from drawing/state logic where practical.
- Prefer descriptive names over abbreviations.
- Avoid `unwrap()` in non-trivial runtime paths.
- Return errors cleanly when appropriate.

---

## UI Rules

- Keep the UI minimal.
- Default behavior should be easy to understand without explanation.
- Avoid cluttered toolbars.
- Do not add many controls at once.
- Add features incrementally.
- Prefer keyboard shortcuts for power features later, but do not add them before the base interaction is solid.

---

## Drawing Rules

- Preserve smooth and predictable drawing behavior.
- Do not break existing stroke behavior when adding features.
- Keep stroke data structures simple unless a clear need emerges.
- If adding smoothing or point filtering, do so conservatively.
- Drawing responsiveness matters more than advanced rendering tricks at this stage.

---

## Platform Rules

This app targets Windows and macOS desktop distribution.

Platform-specific behavior should follow these principles:

- shared logic first
- OS-specific code only where required
- isolate native window handling
- do not leak Windows/macOS implementation details into general app logic

If a feature is platform-specific, document that clearly in code comments and in the README if user-visible.

---

## Dependency Rules

Before adding a crate:

- confirm it is actively maintained
- prefer widely used crates
- avoid overlapping dependencies
- avoid large dependencies for small tasks

If introducing a new dependency, explain briefly in comments or commit message why it is needed.

---

## Refactoring Rules

Refactor only when at least one of these is true:

- the current code is blocking a feature
- complexity has become hard to reason about
- responsibilities are clearly mixed
- testability or platform separation is suffering

Do not refactor purely for style churn.

---

## Testing and Validation

For each meaningful change:

- ensure the project builds
- check for obvious warnings
- avoid leaving broken code paths
- prefer incremental, verifiable changes

If tests are added later:

- keep them focused
- prefer deterministic tests
- do not add fragile snapshot-heavy tests unless necessary

---

## File Editing Guidelines

When editing files:

- preserve existing style unless intentionally improving it
- do not rewrite unrelated sections
- keep diffs narrow
- avoid unnecessary formatting churn
- update README or TODO when behavior or roadmap meaningfully changes

---

## Documentation Rules

Keep documentation concise and practical.

Important files:

- `README.md` for project overview and usage
- `TODO.md` for roadmap and implementation phases
- `AGENTS.md` for coding-agent guidance

If behavior changes, update the relevant document in the same change whenever practical.

---

## Planning Rules

For small tasks:

- implement directly

For medium or multi-step tasks:

- briefly state the plan in working notes or task output

For large tasks:

- create or update a short plan document before major edits if needed

Do not generate long speculative plans for simple changes.

---

## What to Avoid

- overengineering
- unnecessary traits/interfaces
- introducing async without a real need
- adding persistence too early
- adding advanced export before overlay fundamentals work
- mixing platform-native hacks into core drawing code
- making broad unrelated changes in one pass

---

## Near-Term Roadmap Context

The near-term roadmap is:

1. solid MVP drawing
2. undo
3. pen width
4. transparent canvas
5. always-on-top
6. click-through
7. global hotkeys
8. export / save

Use this order unless explicitly instructed otherwise.

---

## Agent Behavior Expectations

When working on this repository:

- be conservative
- make the smallest reasonable change
- preserve cross-platform future viability
- explain tradeoffs briefly when relevant
- prefer reversible decisions
- leave the project in a buildable state whenever possible
