# Slugline

A single-user, local-first, keyboard-driven (vim-modal) daily notes app. Notes are stored as plain
`YYYY-MM-DD.md` files on disk. Slugline is a native desktop app (Rust + [Iced](https://iced.rs))
that ships as a single self-contained binary — no cloud, no browser, no runtime dependencies.

## Features

- **Vim-modal editor** — NORMAL/INSERT modes, motions (`h j k l w b e 0 $ gg G`), edits (`x dd yy p P o O i a A`), undo/redo (`u` / `Ctrl-R`)
- **Per-line rendering** — one raw edit line; all other lines render pretty (headings, tasks, lists, blockquotes, bold/italic/strikethrough/highlight/code/links)
- **Tabs** — open multiple dates side-by-side (`gt` / `gT`, `:tab`, `:close`)
- **Resizable, collapsible sidebar** — calendar, agenda, and 7-day to-do view, dragged wider/narrower or collapsed to a slim rail
- **Calendar** — dots on dates that have notes; click to open or create; month navigation
- **Agenda** — today's scheduled meetings, click to jump to the meeting
- **To Do** — a 7-day rolling view of open/done items across notes, click to jump to the item
- **Command palette** — press `:` (or `⌘K` / `Ctrl+K`) for a fuzzy-searchable list of every command
- **Themes** — built-in `light` (default) and `dark`; switch with `:theme dark` / `:theme light`, or just `:theme` to toggle. The choice is **saved to `config.toml`** (comment-preserving). Partial color overrides via `[ui.colors.<theme>]`.
- **Offline fonts** — Roboto is bundled inside the binary; no network required

## Usage

Slugline is a modal editor in the style of Vim. Keyboard input is interpreted differently depending on the current mode.

### Modes

| Mode | How to enter | How to exit |
|------|-------------|-------------|
| **Normal** | Default on open; `Escape` from Insert | — |
| **Insert** | `i`, `a`, `A`, `o`, `O` | `Escape` |
| **Command** | `:` from Normal, or `⌘K` / `Ctrl+K` from anywhere | `Escape` to cancel, `Enter` to run |

The cursor changes shape: a block in Normal mode, an I-beam in Insert mode.

### Normal Mode

Normal mode is for navigation and editing commands. Keystrokes are not inserted as text.

#### Motions

| Key | Action |
|-----|--------|
| `h` / `←` | Move left |
| `l` / `→` | Move right |
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `w` | Next word start |
| `b` | Previous word start |
| `e` | Word end |
| `0` | Line start |
| `$` | Line end |
| `gg` | First line |
| `G` | Last line |

#### Editing

| Key | Action |
|-----|--------|
| `x` | Delete character under cursor |
| `dd` | Delete current line (saved to register) |
| `yy` | Yank (copy) current line to register |
| `p` | Paste register below current line |
| `P` | Paste register above current line |
| `t` | Toggle task checkbox (`[ ]` ↔ `[x]`) |
| `u` | Undo |
| `Ctrl-r` | Redo |

#### Entering Insert Mode

| Key | Enters insert at… |
|-----|-------------------|
| `i` | Cursor position |
| `a` | After cursor |
| `A` | End of line |
| `o` | New line below |
| `O` | New line above |

#### Tab & Day Navigation

| Key | Action |
|-----|--------|
| `gt` | Next tab |
| `gT` | Previous tab |
| `[` | Previous day |
| `]` | Next day |
| `Ctrl-t` | Today |

### Insert Mode

Insert mode behaves like a standard text editor.

| Key | Action |
|-----|--------|
| `Escape` | Return to Normal mode |
| `Enter` | Insert newline (splits line at cursor) |
| `Backspace` | Delete character before cursor; at column 0, merges with line above |
| `Tab` | Insert 2 spaces |
| `Ctrl-w` | Delete word before cursor |
| `←` `→` `↑` `↓` | Move cursor |
| any text character | Insert at cursor position |

### Command Mode

Press `:` in Normal mode (or `⌘K` / `Ctrl+K` from either mode) to open the command palette: a
fuzzy-searchable overlay of every command below. Type to filter, press `Enter` to run the top
match (or a fully-typed command line), or press `Escape` to cancel.

#### Navigation

| Command | Description |
|---------|-------------|
| `:goto YYYY-MM-DD` | Open a specific date |
| `:today` | Open today's note |
| `:tab YYYY-MM-DD` | Open a date in a new tab |
| `:close` | Close the active tab |

#### File

| Command | Description |
|---------|-------------|
| `:w` | Save the current note to disk |

#### Content

| Command | Description |
|---------|-------------|
| `:meeting <name>` | Append a `### <name>` block under `## Meetings` (creates section if absent) |
| `:note <name>` | Append a `### <name>` block under `## Notes` (creates section if absent) |
| `:todo <text>` | Append `- [ ] <text>` to `## To Do`; if inside a meeting block, tags it with the meeting name |
| `:section <name>` | Insert a sub-heading one level deeper than the heading at the cursor |
| `:people <names>` / `:p <names>` | Add people to the meeting or note block at the cursor |

#### Meeting Metadata

These commands must be run with the cursor inside a `### meeting` block.

| Command | Description |
|---------|-------------|
| `:scheduled HH:MM` | Set the scheduled time for the meeting |
| `:start` | Record the actual start time (uses the current clock time) |
| `:end` | Record the actual end time (uses the current clock time) |
| `:purpose <text>` | Set the meeting purpose |

#### Note Metadata

These commands must be run with the cursor inside a `### note` block.

| Command | Description |
|---------|-------------|
| `:topic <text>` | Set the topic for the note block |

#### UI

| Command | Description |
|---------|-------------|
| `:theme` | Toggle between light and dark |
| `:theme light` | Switch to the light theme |
| `:theme dark` | Switch to the dark theme |

---

## Quick Start

```sh
# Run from source (opens a native window)
make dev

# Production build → single binary
make dist
./target/release/slugline
```

### Command-line options

```
slugline [OPTIONS]

Options:
      --notes-dir <PATH>   Notes directory (default: ~/Documents/Slugline)
      --config <PATH>      Config file path
  -V, --version            Print version
  -h, --help               Print help
```

## Configuration

Slugline writes a default config on first launch:

- **macOS/Linux:** `~/.config/slugline/config.toml`

Example `config.toml`:

```toml
[notes]
notes_dir = "~/Documents/Slugline"

[ui]
theme = "dark"
edit_line_position = 0.35   # 0.0–1.0, fraction from top

[ui.colors.dark]
"--accent" = "#e0af68"
"--rule" = "#2d3650"        # hairline under header / around the edit bar
"--edit-bar-bg" = "#2a344c" # active-line band
```

Most config changes take effect on restart; the active theme is also written back to `config.toml` whenever you switch it in-app.

## Development

**Prerequisites:** Rust (stable)

```sh
# Run with a throwaway notes dir
make dev

# Tests (whole workspace)
make test

# Format
make fmt

# Lint
cargo clippy --workspace --all-targets -- -D warnings
```

## Project Structure

```
slugline/
├── crates/
│   ├── slugline-core/       # headless domain logic, no UI dependency
│   │   └── src/
│   │       ├── agenda.rs    # scheduled-meeting derivation
│   │       ├── config.rs    # TOML config loading + comment-preserving theme writes
│   │       ├── date.rs      # date validation + weekday helper
│   │       ├── dates.rs     # calendar month-grid math
│   │       ├── doc/         # line classifier, inline-span renderer, scanner, commands
│   │       ├── editor/      # vim-modal state machine (motions, edits, insert, keymap)
│   │       ├── store.rs     # filesystem note store (atomic writes, materialize-on-open)
│   │       ├── tabs.rs      # open-tabs state
│   │       ├── theme.rs     # light/dark color token resolution
│   │       └── todos.rs     # 7-day to-do aggregation
│   └── slugline/            # the Iced desktop app
│       └── src/
│           ├── main.rs      # entry point, CLI wiring
│           ├── app.rs       # Model/Message/update/view/subscription
│           ├── cli.rs       # CLI argument parsing (clap)
│           ├── keys.rs      # Iced key event -> keymap-string mapping
│           ├── theme_iced.rs # theme tokens -> iced::Color
│           └── ui/          # editor_pane, sidebar, calendar, agenda, todo_list,
│                             # tab_strip, command_palette, status_line, toast
├── docs/superpowers/         # design docs and implementation plans
├── fixtures/                 # sample note files used by tests
├── Cargo.toml                # workspace manifest
└── Makefile
```

## Notes Format

Notes are plain Markdown files named `YYYY-MM-DD.md`. The supported subset:

```markdown
# 2026-06-24-TUE

## Morning

- [ ] Task item
- [x] Done task
- Regular list item
  - Indented list item
  1. Ordered list item

> Blockquote text

meta: value

Paragraph with **bold**, *italic*, ~~strikethrough~~, ==highlight==, `code`, and [link text](https://example.com).

## Meetings

::: meeting 09:00–10:00 | Topic | Purpose
scheduled: 09:00
start: 09:00
end: 10:00
topic: Standup
purpose: Sync
:::
```

### Inline markup summary

| Syntax | Renders as |
|--------|-----------|
| `**text**` | **Bold** |
| `*text*` or `_text_` | *Italic* |
| `~~text~~` | ~~Strikethrough~~ |
| `==text==` | Highlighted background |
| `` `code` `` | Inline code |
| `[label](url)` | Hyperlink (http/https/mailto only) |

### Block elements

| Syntax | Renders as |
|--------|-----------|
| `# Heading` … `###### Heading` | H1–H6 headings |
| `- [ ] text` / `- [x] text` | Open / done task checkbox |
| `- text` / `* text` / `+ text` | Unordered list item |
| `1. text` | Ordered list item |
| `  - text` (2-space indent) | Nested list item (depth increases per 2 spaces) |
| `> text` | Blockquote (left border, muted italic) |
| `meta:key value` | Metadata field (used by agenda and meeting blocks) |
