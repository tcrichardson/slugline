# Slugline

A single-user, local-first, keyboard-driven (vim-modal) daily notes app. Notes are stored as plain `YYYY-MM-DD.md` files on disk. The whole application ships as a single self-contained binary — no cloud, no dependencies at runtime.

## Features

- **Vim-modal editor** — NORMAL/INSERT modes, motions (`h j k l w b e 0 $ gg G`), edits (`x dd yy p P o O i a A`), undo/redo (`u` / `Ctrl-R`)
- **Per-line rendering** — one raw edit line; all other lines render pretty (headings, tasks, lists, bold/italic/code/links)
- **Tabs** — open multiple dates side-by-side (`gt` / `gT`, `:tab`, `:close`)
- **Calendar sidebar** — dots on dates that have notes; click to open or create
- **Agenda sidebar** — today's scheduled meetings and a 7-day to-do view
- **Command line** — `:w`, `:goto`, `:today`, `:tab`, `:close`, `:theme`, `:meeting`, `:note`, `:section`, `:todo`, `:start`, `:end`, `:scheduled`, `:purpose`, `:topic`
- **Themes** — built-in `light` (default) and `dark`; live-switch with `:theme dark`; partial color overrides via `config.toml`
- **Offline fonts** — Roboto is bundled inside the binary; no network required

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
```

Config changes take effect on restart.

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

Dates must be `YYYY-MM-DD`. Invalid dates and path-traversal attempts return 400.

## Notes Format

Notes are plain Markdown files named `YYYY-MM-DD.md`. The supported subset:

```markdown
# 2026-06-24-TUE

## Morning

- [ ] Task item
- [x] Done task
- Regular list item

meta: value

Paragraph with **bold**, *italic*, `code`, and [link text].

## Meetings

::: meeting 09:00–10:00 | Topic | Purpose
scheduled: 09:00
start: 09:00
end: 10:00
topic: Standup
purpose: Sync
:::
```
