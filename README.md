# Slugline

A single-user, local-first, keyboard-driven (vim-modal) daily notes app. Notes are stored as plain `YYYY-MM-DD.md` files on disk. The whole application ships as a single self-contained binary — no cloud, no dependencies at runtime.

## Features

- **Vim-modal editor** — NORMAL/INSERT modes, motions (`h j k l w b e 0 $ gg G`), edits (`x dd yy p P o O i a A`), undo/redo (`u` / `Ctrl-R`)
- **Per-line rendering** — one raw edit line; all other lines render pretty (headings, tasks, lists, blockquotes, bold/italic/strikethrough/highlight/code/links)
- **Tabs** — open multiple dates side-by-side (`gt` / `gT`, `:tab`, `:close`)
- **Calendar sidebar** — dots on dates that have notes; click to open or create
- **Agenda sidebar** — today's scheduled meetings and a 7-day to-do view
- **Command line** — `:w`, `:goto`, `:today`, `:tab`, `:close`, `:theme`, `:meeting`, `:note`, `:section`, `:todo`, `:start`, `:end`, `:scheduled`, `:purpose`, `:topic`
- **Themes** — built-in `light` (default) and `dark`; switch with `:theme dark` / `:theme light`, or just `:theme` to toggle. The choice is **saved to `config.toml`** (comment-preserving). Partial color overrides via `[ui.colors.<theme>]`.
- **Offline fonts** — Roboto is bundled inside the binary; no network required

## Usage

Slugline is a modal editor in the style of Vim. Keyboard input is interpreted differently depending on the current mode.

### Modes

| Mode | How to enter | How to exit |
|------|-------------|-------------|
| **Normal** | Default on open; `Escape` from Insert | — |
| **Insert** | `i`, `a`, `A`, `o`, `O` | `Escape` |
| **Command** | `:` from Normal | `Escape` to cancel, `Enter` to run |

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

Press `:` in Normal mode to open the command line at the bottom of the screen. Type a command and press `Enter`, or press `Escape` to cancel.

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
# Run from source (auto-opens browser at http://127.0.0.1:4747)
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
  --port <PORT>        Listen port (default: 4747)
  --no-open            Don't auto-open the browser
  --config <PATH>      Config file path
  -V, --version        Print version
  -h, --help           Print help
```

## Configuration

Slugline writes a default config on first launch:

- **macOS/Linux:** `~/.config/slugline/config.toml`

Example `config.toml`:

```toml
[server]
notes_dir = "~/Documents/Slugline"
port = 4747
auto_open = true

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

**Prerequisites:** Rust (stable), Node.js ≥ 18

```sh
# Install frontend dependencies
cd web && npm install

# Run backend dev server (notes in ./dev-notes, no browser open)
make dev

# Frontend dev server with HMR (proxies API to the Rust backend)
make dev-web

# Tests
make test        # Rust unit tests (cargo test)
make test-web    # TypeScript/Svelte unit tests (vitest)

# Type-check frontend
cd web && npm run check

# Format
make fmt         # cargo fmt
make fmt-web     # prettier
```

## Project Structure

```
slugline/
├── src/                  # Rust backend (axum)
│   ├── main.rs
│   ├── app.rs            # HTTP handlers
│   ├── assets.rs         # rust-embed SPA serving
│   ├── cli.rs            # CLI argument parsing
│   ├── config.rs         # TOML config loading
│   ├── date.rs           # Date validation
│   └── store.rs          # Filesystem note store
├── web/                  # Svelte 5 + Vite frontend
│   └── src/
│       ├── lib/
│       │   ├── doc/      # Document model (line classifier, scanner, renderer)
│       │   ├── editor/   # Pure editor state machine (motions, edits, commands)
│       │   ├── components/
│       │   ├── api.ts    # API client
│       │   ├── appState.svelte.ts  # App-wide store
│       │   ├── theme.ts  # Theme token maps + applyTheme
│       │   ├── agenda.ts
│       │   └── todos.ts
│       ├── App.svelte
│       └── main.ts
├── Cargo.toml
├── Makefile
└── plans/                # Implementation plans per phase
```

## API

The Rust server exposes a minimal filesystem API (all other routes serve the SPA):

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/notes` | List dates that have note files |
| `GET` | `/api/notes/{date}` | Read a note (materializes empty template if missing) |
| `PUT` | `/api/notes/{date}` | Write a note (atomic) |
| `GET` | `/api/config` | Read the UI-relevant config subset |
| `PUT` | `/api/config/theme` | Persist the active theme to `config.toml` |

Dates must be `YYYY-MM-DD`. Invalid dates and path-traversal attempts return 400.

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
