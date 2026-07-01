# Phase 1c — Iced Editing Shell (keyboard + editor pane + autosave) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Iced API caution:** Iced's API changes between releases. Method/function names below target
> `0.13.x`. Where a call is flagged **[verify]**, confirm the exact signature in the pinned version's
> docs (`docs.rs/iced/<pinned>`) and adjust — the *intent* is what matters, not the exact spelling.

**Goal:** Turn the read-only Phase 0 window into a real editor. A global keyboard subscription feeds `KeyInput`s into `core::editor::handle_key`; the editor pane renders the active line as raw markdown with a block/beam cursor and every other line pretty; edits autosave to disk after ~750ms idle and flush on quit.

**Architecture:** Depends on the ported `slugline-core` `editor` + `doc` modules (Phases 1a/1b). The Iced `App` is the Elm loop: `Message::Key` → `handle_key` → new `editor`; a `time` subscription drives debounced autosave via `Task::perform`; a window-close subscription performs a final synchronous flush.

**Tech Stack:** Iced `0.13.x` — `keyboard::on_key_press`, `time::every`, `window::close_requests`/`close`, `rich_text`/`span`, `scrollable`, `column`/`row`/`container`/`text`.

---

## File Structure (files added/changed in Phase 1c)

```
crates/slugline/
  src/
    main.rs                        # subscription wired in; store passed to App
    app.rs                         # REPLACES the Phase 0 stub: Model/Message/update/view/subscription
    keys.rs                        # translate iced key events -> core KeyInput
    ui/
      mod.rs                       # pub mod editor_pane; pub mod palette;
      palette.rs                   # dark-theme color constants (until Phase 6 theming)
      editor_pane.rs               # per-line view: raw active line + cursor; pretty lines
```

---

### Task 1: Translate Iced key events → `core::editor::KeyInput`

**Files:** Create `crates/slugline/src/keys.rs`; modify `crates/slugline/src/main.rs` (add `mod keys;`)

- [ ] **Step 1: Write the failing test** — `crates/slugline/src/keys.rs`:

```rust
use iced::keyboard::key::{Key, Named};

/// Map an Iced logical key to the DOM-`KeyboardEvent.key`-style string our keymap expects.
/// Returns `None` for keys we ignore (pure modifiers, unidentified).
pub fn key_string(key: &Key) -> Option<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::key::{Key, Named};
    use smol_str::SmolStr;

    #[test]
    fn named_and_character_map() {
        assert_eq!(key_string(&Key::Named(Named::Enter)).as_deref(), Some("Enter"));
        assert_eq!(key_string(&Key::Named(Named::ArrowLeft)).as_deref(), Some("ArrowLeft"));
        assert_eq!(key_string(&Key::Named(Named::Space)).as_deref(), Some(" "));
        assert_eq!(key_string(&Key::Character(SmolStr::new("h"))).as_deref(), Some("h"));
        assert_eq!(key_string(&Key::Named(Named::Shift)), None);
    }
}
```

Note **[verify]**: the `Key`/`Named` import path and the `Character` payload type (`SmolStr`) are from
iced 0.13. If the pinned version differs, fix the imports and the `Character` match accordingly.

- [ ] **Step 2: Run test to verify it fails** — `cargo test -p slugline keys::`
Expected: FAIL (`todo!()`).

- [ ] **Step 3: Implement `key_string`** (replace `todo!()`):

```rust
pub fn key_string(key: &Key) -> Option<String> {
    match key {
        Key::Named(named) => Some(
            match named {
                Named::Enter => "Enter",
                Named::Backspace => "Backspace",
                Named::Tab => "Tab",
                Named::Escape => "Escape",
                Named::ArrowLeft => "ArrowLeft",
                Named::ArrowRight => "ArrowRight",
                Named::ArrowUp => "ArrowUp",
                Named::ArrowDown => "ArrowDown",
                Named::Space => " ",
                _ => return None,
            }
            .to_string(),
        ),
        Key::Character(s) => Some(s.to_string()),
        _ => None,
    }
}
```

- [ ] **Step 4: Run test to verify it passes** — `cargo test -p slugline keys::`
Expected: PASS. (Add `smol_str` to `[dev-dependencies]` of `crates/slugline/Cargo.toml` only if the test needs to construct `SmolStr` directly: `smol_str = "0.2"`. **[verify]** the version Iced re-exports.)

- [ ] **Step 5: Commit**

```bash
git add crates/slugline/src/keys.rs crates/slugline/src/main.rs
git commit -m "feat(app): translate Iced key events to core KeyInput"
```

---

### Task 2: Theme color constants (temporary, until Phase 6)

**Files:** Create `crates/slugline/src/ui/mod.rs`, `crates/slugline/src/ui/palette.rs`; modify `main.rs` (add `mod ui;`)

- [ ] **Step 1: Write the palette** — `crates/slugline/src/ui/palette.rs` (values from `web/src/lib/theme.ts` DARK):

```rust
use iced::Color;

/// Parse a `#rrggbb` hex string into an Iced Color at compile-usage time.
pub const fn hex(rgb: u32) -> Color {
    Color::from_rgb(
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
    )
}

pub const BG: Color = hex(0x161a26);
pub const FG: Color = hex(0xe7ecf5);
pub const MUTED: Color = hex(0x97a1b3);
pub const CURSOR: Color = hex(0xe7ecf5);
pub const EDIT_BAR_BG: Color = hex(0x2a344c);
pub const RULE: Color = hex(0x2d3650);
pub const HIGHLIGHT_BG: Color = hex(0x713f12);
pub const LINK: Color = hex(0x2f6df6);
pub const TODO_DONE: Color = hex(0x8a93a3);
pub const BLOCKQUOTE_BORDER: Color = hex(0x3b82f6);

/// Heading colors h1..h6.
pub const HEADING: [Color; 6] = [
    hex(0x1d4ed8), hex(0x3b82f6), hex(0x60a5fa), hex(0x7dabfb), hex(0x9cc2fc), hex(0x9cc2fc),
];
```

- [ ] **Step 2: Declare the ui module** — `crates/slugline/src/ui/mod.rs`:

```rust
pub mod editor_pane;
pub mod palette;
```

and add `mod ui;` near the top of `crates/slugline/src/main.rs`.

- [ ] **Step 3: Verify it compiles** — `cargo build -p slugline`
Expected: compiles (unused-warnings are fine until Task 3 consumes the palette).

- [ ] **Step 4: Commit**

```bash
git add crates/slugline/src/ui/
git commit -m "feat(app): dark theme color constants (temporary pre-theming)"
```

---

### Task 3: The editor pane view

**Files:** Create `crates/slugline/src/ui/editor_pane.rs`

- [ ] **Step 1: Implement the pane** — `crates/slugline/src/ui/editor_pane.rs`. There is no unit test here (rendering side effects); it is exercised by the Task 4 manual smoke.

```rust
use iced::font::{Style as FontStyle, Weight};
use iced::widget::{column, container, rich_text, row, scrollable, span, text};
use iced::{Element, Font, Length};

use slugline_core::doc::{classify_line, render_inline, Line, Span};
use slugline_core::editor::{EditorState, Mode};

use super::palette;

const MONO: Font = Font::MONOSPACE;

pub fn view<Message: 'static>(editor: &EditorState) -> Element<'_, Message> {
    let mut col = column![].padding([16, 24]).spacing(2).width(Length::Fill);
    for (i, line) in editor.lines.iter().enumerate() {
        if i == editor.cursor.line {
            col = col.push(active_line(line, editor.cursor.col, editor.mode));
        } else {
            col = col.push(pretty_line(line));
        }
    }
    scrollable(col).height(Length::Fill).into()
}

fn active_line<'a, Message: 'a>(line: &str, col: usize, mode: Mode) -> Element<'a, Message> {
    let chars: Vec<char> = line.chars().collect();
    let col = col.min(chars.len());
    let before: String = chars[..col].iter().collect();
    let cursor_char: String = chars.get(col).map(|c| c.to_string()).unwrap_or_else(|| " ".into());
    let after: String = if col < chars.len() { chars[col + 1..].iter().collect() } else { String::new() };

    let cursor: Element<'a, Message> = match mode {
        Mode::Normal => container(text(cursor_char).font(MONO).color(palette::BG))
            .style(|_| container::Style {
                background: Some(palette::CURSOR.into()),
                ..container::Style::default()
            })
            .into(),
        Mode::Insert => row![
            container(text("")).width(2).height(Length::Fixed(18.0)).style(|_| container::Style {
                background: Some(palette::CURSOR.into()),
                ..container::Style::default()
            }),
            text(cursor_char).font(MONO),
        ]
        .into(),
    };

    let line_row = row![text(before).font(MONO), cursor, text(after).font(MONO)];
    container(line_row)
        .width(Length::Fill)
        .padding([2, 0])
        .style(|_| container::Style {
            background: Some(palette::EDIT_BAR_BG.into()),
            border: iced::Border { color: palette::RULE, width: 1.0, radius: 0.0.into() },
            ..container::Style::default()
        })
        .into()
}

fn pretty_line<'a, Message: 'a>(line: &str) -> Element<'a, Message> {
    match classify_line(line) {
        Line::Blank => text(" ").into(),
        Line::Heading { level, text: t } => {
            let color = palette::HEADING[(level as usize).clamp(1, 6) - 1];
            let size = 24.0 - (level as f32 - 1.0) * 2.0;
            inline(&render_inline(&t), Some(color), Some(size), Weight::Bold, false).into()
        }
        Line::Task { done, text: t } => {
            let box_glyph = if done { "\u{2611}" } else { "\u{2610}" }; // ☑ / ☐
            let content = inline(
                &render_inline(&t),
                if done { Some(palette::TODO_DONE) } else { None },
                None,
                Weight::Normal,
                done, // strikethrough when done
            );
            row![text(box_glyph), text(" "), content].into()
        }
        Line::List { ordered, number, depth, text: t } => {
            let prefix = if ordered {
                format!("{}. ", number.unwrap_or(1))
            } else {
                "\u{2022} ".to_string() // •
            };
            row![
                container(text("")).width(Length::Fixed(depth as f32 * 20.0)),
                text(prefix),
                inline(&render_inline(&t), None, None, Weight::Normal, false),
            ]
            .into()
        }
        Line::Blockquote { text: t } => container(
            inline(&render_inline(&t), Some(palette::MUTED), None, Weight::Normal, false),
        )
        .padding([0, 12])
        .style(|_| container::Style {
            border: iced::Border {
                color: palette::BLOCKQUOTE_BORDER,
                width: 3.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into(),
        Line::Meta { key, text: t } => row![
            text(key.to_uppercase()).size(11).color(palette::MUTED),
            text(" "),
            inline(&render_inline(&t), Some(palette::MUTED), Some(12.0), Weight::Normal, false),
        ]
        .into(),
        Line::Paragraph { text: t } => {
            inline(&render_inline(&t), None, None, Weight::Normal, false).into()
        }
    }
}

/// Build a `rich_text` from spans. `base_*` apply to every span; per-span flags layer on top.
/// **[verify]** span builder method names against the pinned iced version (`.color`, `.font`,
/// `.size`, `.strikethrough`, `.underline`; `.background` may be unavailable — see highlight note).
fn inline<'a, Message: 'a>(
    spans: &[Span],
    base_color: Option<iced::Color>,
    base_size: Option<f32>,
    base_weight: Weight,
    base_strike: bool,
) -> Element<'a, Message> {
    let built: Vec<_> = spans
        .iter()
        .map(|s| {
            let mut sp = span(s.text.clone());
            // Font: bold/italic/code.
            let mut font = Font { weight: base_weight, ..Font::DEFAULT };
            if s.bold {
                font.weight = Weight::Bold;
            }
            if s.italic {
                font.style = FontStyle::Italic;
            }
            if s.code {
                font = MONO;
            }
            sp = sp.font(font);
            if let Some(c) = base_color {
                sp = sp.color(c);
            }
            if s.code {
                sp = sp.color(palette::MUTED);
            }
            if s.link.is_some() {
                sp = sp.color(palette::LINK).underline(true);
            }
            if let Some(sz) = base_size {
                sp = sp.size(sz);
            }
            if base_strike || s.strike {
                sp = sp.strikethrough(true);
            }
            // Highlight (==text==): span background if supported by the pinned version;
            // otherwise fall back to the highlight color as foreground. See design risk note.
            if s.highlight {
                sp = sp.color(palette::HIGHLIGHT_BG);
            }
            sp
        })
        .collect();
    rich_text(built).into()
}
```

- [ ] **Step 2: Verify it compiles** — `cargo build -p slugline`
Expected: compiles. **[verify]** if `rich_text`/`span`/`container::Style`/`Border` names differ in the pinned version, adjust to the equivalent APIs; the structure (one styled span per `Span`, a three-part active row) is the contract.

- [ ] **Step 3: Commit**

```bash
git add crates/slugline/src/ui/editor_pane.rs
git commit -m "feat(app): per-line editor pane (raw active line + cursor, pretty lines)"
```

---

### Task 4: Wire editing, autosave, and flush-on-exit into the app

**Files:** Rewrite `crates/slugline/src/app.rs`; modify `crates/slugline/src/main.rs`

- [ ] **Step 1: Rewrite the app** — `crates/slugline/src/app.rs`:

```rust
use std::time::{Duration, Instant};

use iced::{keyboard, time, window, Element, Subscription, Task};

use slugline_core::editor::{create_editor_state, handle_key, EditorState, KeyInput};
use slugline_core::store::NotesStore;

use crate::keys::key_string;
use crate::ui::editor_pane;

const SAVE_DEBOUNCE: Duration = Duration::from_millis(750);

pub struct App {
    store: NotesStore,
    date: String,
    editor: EditorState,
    last_saved: String,
    dirty_since: Option<Instant>,
    saving: bool,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Key(KeyInput),
    Tick,
    Saved { content: String, res: Result<(), String> },
    CloseRequested(window::Id),
}

impl App {
    pub fn new(store: NotesStore, date: String) -> Self {
        let (content, error) = match store.read_or_create(&date) {
            Ok(c) => (c, None),
            Err(e) => (String::new(), Some(format!("Failed to load note: {e}"))),
        };
        let editor = create_editor_state(content.lines().map(str::to_string).collect(), Vec::new());
        Self { store, date, editor, last_saved: content, dirty_since: None, saving: false, error }
    }

    pub fn title(&self) -> String {
        format!("Slugline \u{2014} {}", self.date)
    }

    /// The buffer as file content, with a guaranteed trailing newline.
    fn content(&self) -> String {
        let body = self.editor.lines.join("\n");
        if body.ends_with('\n') { body } else { format!("{body}\n") }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Key(input) => {
                let before = self.editor.lines.clone();
                let result = handle_key(&self.editor, &input);
                self.editor = result.state;
                // (result.effect is always None in the walking skeleton; handled in Phase 2/5.)
                if self.editor.lines != before {
                    self.dirty_since = Some(Instant::now());
                }
                Task::none()
            }
            Message::Tick => {
                let idle = self.dirty_since.map(|t| t.elapsed() >= SAVE_DEBOUNCE).unwrap_or(false);
                if idle && !self.saving {
                    let content = self.content();
                    if content == self.last_saved {
                        self.dirty_since = None;
                        return Task::none();
                    }
                    self.saving = true;
                    let store = self.store.clone();
                    let date = self.date.clone();
                    let to_save = content.clone();
                    return Task::perform(
                        async move { store.write(&date, &to_save).map_err(|e| e.to_string()) },
                        move |res| Message::Saved { content, res },
                    );
                }
                Task::none()
            }
            Message::Saved { content, res } => {
                self.saving = false;
                match res {
                    Ok(()) => {
                        self.last_saved = content;
                        if self.content() == self.last_saved {
                            self.dirty_since = None;
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Save failed \u{2014} edits kept, will retry: {e}"));
                        // dirty_since stays set, so the next Tick retries.
                    }
                }
                Task::none()
            }
            Message::CloseRequested(id) => {
                // Final synchronous flush so nothing is lost on quit.
                let content = self.content();
                if content != self.last_saved {
                    let _ = self.store.write(&self.date, &content);
                }
                window::close(id)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        editor_pane::view(&self.editor)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::on_key_press(|key, mods| {
                key_string(&key).map(|k| {
                    Message::Key(KeyInput {
                        key: k,
                        ctrl: mods.control(),
                        meta: mods.logo(),
                        shift: mods.shift(),
                    })
                })
            }),
            time::every(Duration::from_millis(250)).map(|_| Message::Tick),
            window::close_requests().map(Message::CloseRequested),
        ])
    }
}
```

Note **[verify]**: `keyboard::on_key_press`, `mods.logo()`, `time::every`, `window::close_requests()`,
and `window::close(id)` are the 0.13 spellings — confirm against the pinned docs. `self.error` is set
but not yet displayed; the toast surface is Phase 6. (Add `#[allow(dead_code)]` on the `error` field
if clippy complains this phase.)

- [ ] **Step 2: Wire the subscription in `main.rs`** — update the `iced::application(...)` call at the end of `crates/slugline/src/main.rs`:

```rust
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run_with(move || (App::new(store.clone(), date.clone()), Task::none()))
```

(Requires `NotesStore: Clone` — it already derives `Clone`. Ensure `store` is cloned into the closure.)

- [ ] **Step 3: Build** — `cargo build -p slugline`
Expected: compiles. Resolve any **[verify]** API mismatches now.

- [ ] **Step 4: Manual smoke test — editing + autosave**

Run: `cargo run -p slugline -- --notes-dir ./dev-notes`
Verify all of:
1. Window opens titled `Slugline — <today>` showing the materialized template; non-active lines render pretty (the `#` heading is colored/bold; `## To Do` etc. are colored).
2. `j`/`k` move the active-line band; the active line shows raw markdown with a **block** cursor.
3. `o` opens a line below and switches to a **beam** cursor; type `- [ ] hello`, press `Escape`.
4. Move onto that line; press `t` — the checkbox toggles to `- [x] hello`; the non-active rendering shows ☑ with strikethrough.
5. Type more, wait ~1s, then inspect `./dev-notes/<today>.md` on disk — it reflects your edits.
6. `u` undoes; `Ctrl-r` redoes.
7. Close the window; reopen — your last edits are present (flush-on-exit worked).

- [ ] **Step 5: Commit**

```bash
git add crates/slugline/src/app.rs crates/slugline/src/main.rs
git commit -m "feat(app): modal editing wired to core, debounced autosave, flush-on-exit"
```

---

### Task 5: Workspace hygiene gate

**Files:** none (verification only)

- [ ] **Step 1: Full test run** — `cargo test --workspace`
Expected: green (core: date/store/config/dates/doc/editor; app: cli/keys).

- [ ] **Step 2: Format + clippy** — `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean. Fix and re-run if needed.

- [ ] **Step 3: Commit any fixups**

```bash
git add -A
git commit -m "chore: fmt + clippy clean for phase 1c" || echo "nothing to commit"
```

---

## Self-Review (performed while writing this plan)

- **Spec coverage:** Completes design Section 3 (per-line pretty vs. raw active line + block/beam
  cursor) and the autosave/flush parts of Section 5. Consumes `core::editor::handle_key`,
  `create_editor_state`, `EditorState`, `Mode`, `KeyInput` (Phase 1b) and `classify_line`/`Line`/
  `render_inline`/`Span` (Phase 1a) — names match exactly.
- **Deferred, on purpose:** navigation/tabs (`AppEffect` handling) → Phase 2; command palette and
  `:` commands → Phase 5; theming, OS light/dark, and the error toast surface → Phase 6; scroll-to-
  active-line refinement and Roboto embedding → their respective later phases. The `result.effect`
  from `handle_key` is intentionally ignored here (always `None` in Phase 1).
- **Placeholder scan:** the only `todo!()` is the red-phase stub in `keys.rs`, replaced in-task. All
  **[verify]** markers point at real, checkable iced-0.13 API details, not vague gaps.
- **Known risks carried from the design:** (1) `==highlight==` uses foreground color as a fallback if
  span backgrounds are unavailable in the pinned version; (2) links render styled but non-clickable
  (click-to-open deferred). Both are explicitly acceptable for the walking skeleton.
