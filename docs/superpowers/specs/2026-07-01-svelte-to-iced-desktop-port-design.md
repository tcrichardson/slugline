# Design: Port Slugline from Svelte web app to Rust/Iced desktop app

**Date:** 2026-07-01
**Status:** Approved

## Summary

Slugline is currently a hybrid app: a Rust/axum backend (~960 LOC) that serves a filesystem API and an embedded Svelte 5 SPA (~1,591 LOC of pure logic + ~465 LOC of components + ~951 LOC of tests). This design replaces the web frontend and HTTP layer with a native **Rust/Iced** desktop application, eliminating the Node/web toolchain and shipping a single self-contained binary.

The port is tractable because the frontend is already architected as a **pure, framework-agnostic state machine** (`handle_key(state, input) -> { state, effect }`) — the Elm architecture, which maps directly onto Iced's Model–Message–update–view. The domain/store/config logic is already Rust and is reused nearly wholesale.

### Decisions

- **Motivation:** kill the Node/web toolchain, ship a true native desktop app as a single binary, improve resource use, and as a Rust/Iced learning vehicle.
- **North star UI:** "Native-refined" — same information architecture as today, upgraded with desktop-grade interactions (resizable/collapsible panes, a fuzzy command palette, dynamic window title, OS light/dark following). Not a carbon copy; not a radical redesign.
- **First milestone:** a walking skeleton (open today's note → vim-modal edit → autosave), then widen via vertical slices.
- **Execution approach:** walking-skeleton vertical slices, TDD throughout, in a Cargo workspace. The first real slice ports the cohesive editor engine (with its tests) as a unit.
- **Coexistence:** replace in place on a feature branch. Gut `web/` and the axum/HTTP layer as Iced reaches parity; the web version stays recoverable via a git tag.

---

## Section 1 — Workspace & crate structure

A Cargo workspace with two members.

**`slugline-core`** (library, **no Iced dependency**) — the headless brain, fully unit-tested:

```
editor/    state, motions, edits, insert, keymap, commands   (port of web/src/lib/editor/*)
doc/       classify, scan, context, command, render          (render emits a structured
                                                               line/span model, NOT an HTML string)
tabs, todos, agenda, dates                                    (port of the matching .ts files)
theme                                                         (token structs + light/dark palettes)
store, config, date                                           (REUSED almost as-is from today's src/)
```

**`slugline`** (binary — the Iced app), depends on `slugline-core`:

```
main.rs            Iced entry + clap CLI
app/               the Iced program: Model, Message, update, view, subscription (port of appState)
ui/                editor_pane, sidebar, calendar, agenda, todo_list, status, command_palette
theme_iced.rs      adapts core theme tokens -> iced::Theme / styling
fonts.rs           Roboto + a mono face embedded via include_bytes!
```

**Deleted at cutover:** `src/app.rs`, `src/assets.rs`, the entire `/api/*` HTTP layer, `web/`, and deps `axum`, `rust-embed`, `mime_guess`, `tower`. `store.rs`/`config.rs`/`date.rs` move into `slugline-core` unchanged. The store is synchronous filesystem I/O on tiny files, so `tokio` is likely dropped entirely (Iced runs its own async executor).

**CLI:** `clap` stays. Keep `--notes-dir` and `--config`; drop `--port` and `--no-open` (no server, no browser). Retain `-V/--version`, `-h/--help`.

**Iced version:** pin to a specific release (targeting `0.13.x`) in the workspace `Cargo.toml` — Iced's API churns between versions, so pinning avoids surprise breakage.

---

## Section 2 — Iced application architecture (the Elm loop)

The current `AppStore` maps almost mechanically onto Iced's Model–Message–update–view because the editor is already an Elm-style `(state, input) -> { state, effect }` reducer.

**Model** — one struct mirroring `appState.svelte.ts` fields: `tabs_state`, `editor: EditorState`, `notes_with_files`, `config`, `now`, `calendar`, `todo_groups`, `error`, plus native-refined UI state (`panes`, `palette`, panel-collapsed flags) and autosave bookkeeping (`last_saved`, `dirty_since`).

**Message** — the union of every input:

```rust
enum Message {
    Key(KeyInput),                 // from a global keyboard subscription
    Tick(SystemTime),              // 30s clock + autosave check
    ConfigLoaded(Result<UiConfig, E>),
    NoteLoaded { date: String, body: Result<String, E> },
    NotesListed(Vec<String>),
    TodosRefreshed(Vec<TodoGroup>),
    Saved(Result<(), E>),
    ThemePersisted { target: String, prev: String, res: Result<(), E> },
    OpenDate(String), OpenDateAndLine(String, usize),
    SwitchTab(usize), CloseTab(usize),
    PrevMonth, NextMonth,
    TogglePanel(Panel), PaneResized(pane_grid::ResizeEvent),  // native-refined
    DismissError,
}
```

**update** — `Message::Key(k)` calls `core::editor::handle_key(&editor, k, ctx) -> KeyResult`, stores the new `editor`, marks dirty if lines changed, and converts the returned `AppEffect` into an `iced::Task`. The existing `AppEffect` variants (`goto/today/tab/close/save/prevDay/nextDay/tabNext/tabPrev/theme`) each become a `Task` that runs a store op and emits a follow-up `Message` — a direct translation of `runEffect`.

**view** — `view(&self) -> Element<Message>` rebuilt each update (replaces Svelte reactivity). Detailed in Sections 3–4.

**subscription** — two sources:
- `keyboard::on_key_press` → translate `iced::keyboard::Key` + `Modifiers` into the existing `KeyInput { key, ctrl, meta, shift }`. Global (not tied to a focused input), which fits a modal editor perfectly.
- `time::every(30s)` → `Tick` for the clock, also used to drive debounced autosave (Section 5).

Mouse actions (calendar day, tab, todo item) emit `OpenDate` / `SwitchTab` / `OpenDateAndLine`, mirroring today's click handlers.

---

## Section 3 — The editor pane (the crux)

The editor `view` is a `scrollable` containing a `column` of one row per line, rendered one of two ways — mirroring today's `EditorPane.svelte`.

**Active line (raw).** A `row` of three text segments — `before` | `cursor char` | `after` — in a monospace font, inside a `container` with the edit-bar background band and hairline top/bottom borders. The cursor is drawn, not native:
- NORMAL → block: the cursor char gets an inverted background (`--cursor` bg, `--bg` fg).
- INSERT → beam: a 2px left rule before the cursor char.

The active line stays **raw markdown** (markers visible), so we never style *and* place a cursor on the same run — sidestepping all the hard "cursor inside a rich span" problems.

**All other lines (pretty).** Driven by a structured model, not HTML. In `core`, `classify_line` returns a `Line` enum (`Heading{level,text}`, `Task{done,text}`, `Meta{key,text}`, `List{ordered,depth,number,text}`, `Blockquote{text}`, `Paragraph{text}`, `Blank`). The UI maps each to widgets: headings = sized/colored `text`; task = checkbox glyph + inline content; list = indented row with bullet/number; blockquote = left-border `container`, muted italic; meta = small uppercase key + value.

**Inline markup → spans (not HTML).** Replace `renderInline`'s HTML output with `core::render_inline(&str) -> Vec<Span>`, where `Span { text, bold, italic, strike, highlight, code, link: Option<String> }`. Parsing and its tests stay in `core`; the UI maps `Vec<Span>` → `iced::widget::rich_text` with styled `text::Span`s (color, bold via font weight, italic, strikethrough, `Span::link`).

**Known risk points:**
1. **`==highlight==` background.** Per-span *background* may be limited in the pinned Iced release. Plan: use span background if supported; otherwise fall back to a distinct highlight foreground color and revisit. This is the single lowest-fidelity spot.
2. **Scroll-to-active-line.** Today it's `scrollTop = activeEl.offsetTop − height·position`. Pretty lines have varying heights, so pixel-perfect needs measured heights. Plan: start with `scrollable` + a computed `RelativeOffset`, refine later if it feels off.

---

## Section 4 — The shell & native-refined features

**Layout via `pane_grid`.** The window splits into a resizable left **sidebar pane** (calendar → agenda → todos, stacked) and a **main pane** (tab strip on top, editor below, status line at bottom). Dragging the divider emits `PaneResized`; pane ratios persist in the Model.

**Collapsible panels.** Each sidebar section has a chevron header → `TogglePanel(Panel)` toggles a bool; `view` conditionally renders the body. The whole sidebar can collapse too.

**Command palette overlay.** The signature native-refined touch, and it costs almost nothing in `core`: pressing `:` already puts the state machine into command mode (`editor.command = Some(..)`). We present that state as a floating overlay via `stack![ base_view, palette ]` — a top-centered box showing the typed text plus a **fuzzy-filtered list** of known commands. `Enter` runs it through the exact same `run_command` path, so behavior is identical to typing `:cmd`. Add a `Cmd/Ctrl-K` shortcut that seeds command mode. The old bottom command-line is retired; core logic is untouched.

**Theming.** `core` owns the token structs (light/dark + `config` overrides). The UI builds an `iced::Theme::Custom` palette plus a styling module that reads tokens for per-widget colors/borders. `:theme` flows effect → `update` swaps `config.theme` → next `view` uses the new theme, and persists via the existing comment-preserving `toml_edit` writer (now a direct `core` call).

**Window title.** Iced's `title(&self)` returns `"Slugline — {active_date}"`, updating as you navigate.

**Fonts.** Roboto (already bundled) + a monospace face embedded with `include_bytes!`, registered at startup; monospace used for the raw active line.

**Deferred (need external crates — YAGNI for now):**
- **OS light/dark following** — needs a helper like `dark-light`; add a `:theme system` mode later. Explicit light/dark ships first.
- **Native menu bar / system tray** — not in Iced core (`muda`/`tray-icon`). Skip; the vim keymap + palette cover interaction. In-app shortcuts (⌘K) live in the keyboard subscription.

Tabs stay in-app (multi-window was the rejected "reimagined" direction); a simple tab-strip `row` of buttons driving `SwitchTab`/`CloseTab`.

---

## Section 5 — Data flow & persistence

No more HTTP — the store is a direct `core` dependency. `core::store` reuses today's `store.rs`: `list_notes()`, `read_note(date)` (materializes the empty template when missing), `write_note(date, content)` (atomic). These run via `Task::perform` so results arrive as Messages and the UI thread never blocks.

**Startup** (`init()` today) fires initial Tasks: load config → `ConfigLoaded`, then read active note → `NoteLoaded`, plus `NotesListed` and `TodosRefreshed`.

**Opening a date** (`OpenDate`). Today's code `await flush()` *before* retargeting. Since `update` can't await, the invariant is modeled as one composed future: `Task::perform(async { flush_if_dirty(cur).await; read_note(next).await }, |body| NoteLoaded { date, body })`. On `NoteLoaded`, `update` rebuilds `editor` via `create_editor_state(lines, shared_register)`, resets the calendar month, and chains `NotesListed` + `TodosRefreshed`.

**Debounced autosave.** Today: a 750ms `setTimeout` after each mutating key. In Iced: `update` sets `dirty_since = Some(Instant::now())` whenever `editor.lines` changed; the `time::every(250ms)` subscription's `Tick` checks `dirty_since` and, once >750ms idle, spawns a flush `Task` and clears it. Flush compares normalized content vs `last_saved`, writes atomically, and on success updates `last_saved` + refreshes todos.

**Flush on exit.** Subscribe to Iced's window close-request, run a final synchronous flush, then allow the window to close — so nothing is lost on quit.

**Config.** `ConfigLoaded` reads the UI subset; `:theme` persistence calls `core::config::persist_theme` (comment-preserving `toml_edit`) via a Task → `ThemePersisted`, with rollback-on-failure.

**Shared register.** The Model holds `shared_register: Vec<String>`, fed into `create_editor_state` on load and updated from `editor.register` after each key — mirroring today's cross-tab yank register.

---

## Section 6 — Error handling & testing

**Error handling** (mirrors today's `setError` + `Toast`, auto-dismiss 5s):
- Model holds `error: Option<String>` + an expiry; rendered as a toast via `stack`. `Tick` clears it once expired.
- **Load failure** → keep the current buffer, toast `"Failed to load note {date}"`.
- **Save failure** → toast `"Save failed — edits kept in memory, will retry"`; buffer retained and `dirty_since` stays set, so the next autosave tick retries.
- **Config load failure** → fall back to defaults + toast.
- **Theme-persist failure** → roll back `config.theme` in the Model, re-render + toast.
- **Validation** (dates, path-traversal) moves into `core` — `date` validation runs before any store call, replacing the axum layer's 400s. Store I/O returns `Result`; `core` functions are otherwise total, so no panics expected.

**Testing** (TDD throughout):
- The existing **951 LOC of TS tests port to Rust `#[test]`** in `slugline-core`, near 1:1 — editor, doc, tabs, todos, agenda, dates, theme. They are the parity spec and regression net, ported red-green alongside each module.
- `render_inline` tests get *better*: assert the `Vec<Span>` structure instead of HTML strings.
- The existing Rust `store`/`config` tests (with `tempfile`) move into `core` unchanged.
- The **`update` reducer** gets light unit coverage (Message → Model transitions), ignoring the returned `Task`.
- The **Iced `view` is not unit-tested** (rendering side effects) — same boundary as today's untested `EditorPane`/`applyTheme`. Covered by manual smoke per slice.
- CI becomes `cargo test` + `cargo fmt --check` + `cargo clippy`, workspace-wide. Drop vitest / svelte-check / prettier / npm.

---

## Milestone roadmap (vertical slices)

Expanded by the implementation plan.

0. **Scaffold** — workspace + Iced window rendering today's note read-only (pretty lines). Proves Iced + rendering.
1. **Walking skeleton** — port `editor/*` + `doc/*` with tests; keyboard subscription; NORMAL/INSERT editing with block/beam cursor; autosave to disk.
2. **Navigation & tabs** — `[ ] :goto :today`, `gt/gT :tab :close`, shared register, flush-before-navigate, window title.
3. **Sidebar** — calendar (dots, click, month nav) in a resizable `pane_grid`.
4. **Agenda & todos** — 7-day window.
5. **Commands & palette** — command mode → fuzzy overlay, all `:` commands, ⌘K.
6. **Theming & polish** — light/dark, `:theme` + persistence, status line, toasts.
7. **Cutover** — delete `web/` + axum layer + deps, tag the web version, finalize CLI + docs.

---

## Effort estimate

Rough, for reaching feature parity:

| Work | Effort | Risk |
|---|---|---|
| Reuse backend logic, drop server glue | ~0 (delete) | none |
| Port pure logic (~1,200 LOC TS→Rust) + tests | 1–2 wk | low, mechanical |
| Build Iced shell + simple components + theming | 2–3 wk | low |
| Editor: dual per-line render + cursor + rich inline | ~1 wk | medium (the crux) |

~4–6 weeks focused solo, less if fluent in Iced. Add ramp-up if Iced is unfamiliar; pin the version.
