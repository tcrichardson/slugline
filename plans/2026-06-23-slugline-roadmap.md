# Slugline Implementation Roadmap

> This roadmap decomposes Slugline into independently testable subsystems. Each phase
> becomes its own detailed bite-sized plan before execution. Decisions referenced here
> were settled in the design interview; see `plans/design_idea.md` for the original brief.

**Product:** A single-user, local-first, keyboard-driven (vim-modal) notetaking app for
bullet-journal-style daily notes stored as plain `YYYY-MM-DD.md` files. TypeScript + Svelte
frontend; Rust (`axum`) backend serving a thin filesystem API and the embedded SPA as one binary.

---

## Architectural invariants (apply to every phase)

- **Single-user, local-first.** Binds `127.0.0.1` only. No auth.
- **Source of truth = raw Markdown lines.** Structure is *derived on demand*, never the source of truth.
- **All Markdown/structure logic lives in the TS client.** The Rust server is a thin file store
  (`GET /api/notes`, `GET/PUT /api/notes/{date}`, `GET /api/config`). A future LLM assistant is
  *additive* (new endpoints + a Rust parser conforming to the written grammar) and does not change v1.
- **Filename is canonical `YYYY-MM-DD.md`.** The H1 title (with weekday, e.g. `# 2026-06-23-TUE`)
  is decoupled display text.
- **Editor logic is pure and DOM-free** (`(state, input) -> newState` reducers) so it is unit-testable.
- **Per-line rendering:** exactly one active "edit line" shows raw Markdown; all other lines render
  pretty. Supported subset only: ATX headings, task items, list items, `meta:` lines, paragraphs with
  inline `**bold**` / `*italic*` / `` `code` `` / `[links]`. Code fences and tables are out of scope.

---

## Phase plan (dependency order)

| Plan | Subsystem | Produces (working + testable) | Depends on |
|---|---|---|---|
| **1** | **Document model core (TS)** | Grammar spec + fixture corpus + pure TS library: line classifier, structural scanner, context resolver, inline renderer, command parser. All Vitest-tested against fixtures. Plus the `web/` Vite+Svelte+TS scaffold and root `Makefile`. | — |
| **2** | **Rust file API + server** | Runnable `axum` binary: `GET /api/notes`, `GET/PUT /api/notes/{date}` (atomic write, materialize-on-open), `GET /api/config`, strict date validation / path-traversal rejection, TOML config, startup dir validation, `rust-embed` SPA serving, `--notes-dir`/`--port`/`--no-open`. Rust unit tests over a temp dir. | — |
| **3** | **Frontend app shell** | Svelte app served by the binary: API client, tab store (open/retarget/de-dup, `gt`/`gT`, `:close`, ≥1 tab), header wall-clock, layout, config load + theme tokens applied, calendar grid rendering with has-note dots and two-way date sync (display only). | 1, 2 |
| **4** | **The editor** | Modal editor over the line-array: NORMAL/INSERT, motions (`h j k l w b e 0 $ gg G`), edits (`x dd yy p P o O i a A t`), undo/`Ctrl-R`, shared line-wise register, per-line pretty rendering with single raw edit line, centered scroll anchor with clamping, `:` command line + interpreter applying `:meeting/:note/:section/:todo/:start/:end/:scheduled/:purpose/:topic/:goto/:today/:tab/:close/:w/:theme`, autosave (debounce + flush). | 1, 2, 3 |
| **5** | **Sidebar features** | Agenda derivation (active-note scheduled meetings, click-to-jump), 7-day To Do aggregation grouped by date (read-only + click-to-navigate), calendar click-to-open/create wiring. | 1, 3, 4 |
| **6** | **Themes & packaging polish** | Built-in light/dark token maps + blue heading ramp, partial per-theme config overrides, `:theme` session switching, bundled-offline Roboto, release build (`make build`) producing the single binary, error-surfacing polish. | 3, 4 |

Plans **1** and **2** are independent and may be built in parallel. Everything else follows the order above.

---

## "Done" definition per phase

- **1:** `cd web && npm test` is green; fixtures cover valid + edge-case documents; grammar spec committed to `docs/document-grammar.md`.
- **2:** `cargo test` green; `cargo run` serves the API; manual `curl` round-trip of a note works; traversal attempts return 400.
- **3:** `make build && ./target/release/slugline` opens a browser showing the header clock, an (empty) editor pane, tabs, and a calendar with dots for existing dates; switching tabs/dates updates the calendar selection.
- **4:** A full editing session works end-to-end against real files: type notes, run every command, undo/redo, autosave verified on disk.
- **5:** Agenda and To Do panels populate correctly from real notes and navigate on click.
- **6:** Light/dark themes switch live; custom colors from config apply; `make build` yields a distributable binary.

---

## Out of scope for v1 (explicitly deferred)

IME/CJK composition input; fenced code blocks & tables in the renderer; count prefixes (`3dd`);
character-wise register; help overlay (`?` / `:help` unbound); sidebar todo toggling;
live file-watching / external-change reload (last-writer-wins); end-to-end/browser tests;
user-customizable note template; live config reload (restart to apply); search (`/` reserved).
