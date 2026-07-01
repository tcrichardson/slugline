# Rust/Iced Desktop Port — Implementation Roadmap

> This roadmap decomposes the Svelte→Rust/Iced port into independently testable phases.
> Design source of truth: `docs/superpowers/specs/2026-07-01-svelte-to-iced-desktop-port-design.md`.
> Per the approved authoring strategy, **Phase 0 and Phase 1 are written in full detail now**;
> Phases 2–7 are authored at full fidelity just-in-time, right before each is executed, once its
> predecessor's real Rust types exist. This avoids speculative rework.

## Working conventions (apply to every phase)

- **Branch:** all execution happens on a feature branch (e.g., `iced-port`) off `master`. The web
  version is preserved via a git tag (`web-final`) before the cutover phase deletes `web/`.
- **TDD:** the existing TypeScript tests are the parity spec. Port them to Rust `#[test]` red-green,
  alongside each module. Never port implementation without first porting (and failing) its test.
- **Workspace toolchain:** `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings` must be
  green at the end of every task. These replace `make test` / `make test-web` / vitest / prettier.
- **Iced version:** pinned in the root `Cargo.toml` (target `0.13.x`). Do not float the version.
- **Source-of-truth invariant:** raw Markdown lines are the truth; structure is derived on demand.
  The active editor line is always raw; every other line renders pretty.

## Phase plan (dependency order)

| Phase | Doc | Produces (working + testable) | Depends on |
|---|---|---|---|
| **0** | `…-phase-0-scaffold.md` | Cargo workspace (`slugline-core` + `slugline`); store/config/date moved into core with existing tests green; a minimal Iced window that reads today's note via the store and shows it as plain monospace text; embedded font; dynamic window title. | — |
| **1a** | `…-phase-1a-doc-model.md` | `doc` classify + inline-span renderer ported to `core` with tests (`Line` enum, `Vec<Span>`). | 0 |
| **1b** | `…-phase-1b-editor-engine.md` | Editor engine ported to `core` with tests: `state/motions/edits/insert/keymap` (editing subset) + `KeyInput`/`KeyResult`/`AppEffect` types. | 0 |
| **1c** | `…-phase-1c-iced-editing.md` | Iced editor pane (per-line pretty rendering + raw active line + block/beam cursor); global keyboard subscription → `handle_key`; debounced autosave to disk; flush-on-exit. The walking skeleton is complete when 1c passes. | 1a, 1b |
| **2** | *(JIT)* | Navigation & tabs: `[ ] :goto :today`, `gt/gT :tab :close`, shared yank register across tabs, flush-before-navigate, `AppEffect`→`Task` wiring, window title reflects active date. | 1 |
| **3** | *(JIT)* | Sidebar: calendar (has-note dots, click-to-open, month nav) inside a resizable/collapsible `pane_grid`. | 1, 2 |
| **4** | *(JIT)* | Agenda derivation + 7-day To Do aggregation, click-to-navigate. | 1, 2, 3 |
| **5** | *(JIT)* | Command mode → fuzzy command palette overlay; all `:` commands (`meeting/note/todo/section/scheduled/start/end/purpose/topic` + navigation); ⌘K. | 1, 2, 4 |
| **6** | *(JIT)* | Theming: light/dark palettes, `:theme` switch + comment-preserving persistence, status line, toast/error surface. | 1, 3, 5 |
| **7** | *(JIT)* | Cutover: delete `web/`, `src/app.rs`, `src/assets.rs`, and the axum/`rust-embed`/`mime_guess`/`tower` deps; tag `web-final`; finalize CLI (`--notes-dir`, `--config`); update README/docs. | all |

## "Done" definition per phase

- **0:** `cargo test` green (moved store/config/date tests pass); `cargo run -p slugline` opens a
  native window titled `Slugline — <today>` showing today's note as raw text; window closes cleanly.
- **1a/1b:** `cargo test -p slugline-core` green; the ported `doc` and `editor` modules match the
  behavior of their TypeScript `*.test.ts` counterparts.
- **1c:** A real editing session works against files on disk: enter INSERT and type, motions and edits
  behave per the ported tests, the active line shows raw markdown with a block/beam cursor while other
  lines render pretty, and edits autosave within ~750ms (verified on disk). `cargo test` green.
- **2–7:** defined in each phase's JIT plan; every phase ends with all workspace checks green and a
  manual smoke of that phase's user story.
