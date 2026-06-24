# Slugline

A web-based keyboard-driven notetkaing UI for bullet-journal-style daily notes. Notes are stored on disk as plain Markdown — one file per day — so they can be downloaded and used with any editor, version control, or sync tool.

## Layout

The UI is a single page tabbed UI with a header containing the application name on the left-hand side and the current date formatted YYYY-MM-DD with the current time immediately below the data formated as HH:MM.  The header will also contain tabs for open notes.

Below the header on the left hand side is the note taking pane.  To the right, a sidebar containing a calendar widget at the top with a "Agenda" and "To Do" section below it.

Upon load, the app opens today's note, creating it from a template if it doesn't exist yet.

### Right-hand Sidebar

#### The Calendar Widget
The calendar widget should default to the current date and provide an indication of dates that have daily notes associated with them.  Clicking on a date should open the note for that date, if a note exists, and create a new daily note, if not.

#### Agenda
The agenda section displays the schedule of meetings for the selected note.  The agenda is populated from `scheduled` metadata within meeting notes.  More on note metadata below.

#### To Do
The todo list is populated from the To Do note section and should show to dos for the most recent 7 days divided into sections based on the note titles from which they come. 

### Note Taking Pane
The note taking pane allows users to enter markdown notes inspired by bullet journaling, one note per day with the following sections:

```
# 2026-06-23-TUE

## To Do

- [ ] <To Do 1>
- [ ] <To Do 2>
- [X] <To do 3>

## Meetings

### <Meeting 1>
meta:purpose <text>
meta:scheduled HH:MM
meta:started HH:MM
meta:ended HH:MM

<Markdown Content>

### <Meeting 2>

## Notes

### <Note 1>
meta:topic <text>

### <Note 2>
```

- The tile is alway the first Heading 1 and the note should be named <title>.md when saved
- The standard notes sections (To Do, Meetings, Note) existing at Heading 2
- Indivudal meetings and notes existing at Heading 3
- Ad Hoc sections may be added to individual meetings and notes with the `/section <name>` command, adding sections at Heading 4-6, one level deeper than its parent heading. 


The note taking pane have two modes, similar to vim.  The `esc` key will be used to toggle modes.

#### Vim normal mode

A status line at the bottom shows `-- NORMAL --` and the current context. The current line is highlighted and the cursor is shown as a solid block.  The user can take the following actions in Vim Normal Mode:

| Key | Action |
|---|---|
| `j` / `k` or `↑` / `↓` | Move cursor up/down |
| `h` / `l` or `←` / `→` | Move cursor left/right |
| `w` / `b` | Move to start of next / previous word |
| `e` | Move to end of current/next word |
| `0` / `$` | Jump to start / end of line |
| `gg` / `G` | Jump to first / last line |
| `t` | Toggle a to-do done / not done |
| `x` | Delete character under cursor |
| `dd` | Delete the current line (also yanks it) |
| `yy` | Yank (copy) the current line |
| `p` | Paste yanked line below cursor |
| `P` | Paste yanked line above cursor |
| `u` | Undo last edit |
| `i` | Enter insert mode at cursor |
| `a` | Enter insert mode after cursor |
| `A` | Enter insert mode at end of line |
| `o` | Insert new line below and enter insert mode |
| `O` | Insert new line above and enter insert mode |
| `Enter` | Open the current line for editing in the capture box |
| `?` | Open help overlay |
| `Esc` | Return to capture mode |

The use can also exefute the following "slash commands" in Vim normal mode:

| Command | What it does |
|---|---|
| `/meeting "Name"` | Adds a meeting in the Meetings section `### Name` and sets the context to the new meeting |
| `/note "Name"` | Adds in the Notes section `### Name` and sets the context to the new note |
| `/section "Name"` | Add a sub-section one heading level deeper (max `######`).  Context remains with the meeting or note |
| `/todo Buy milk` | Add a to-do to the central To-dos list. If you're in a meeting, it gets tagged `_(Meeting Name)_`. |
| `/start` | Record the meeting start time (current HH:MM) as metadata. |
| `/end` | Record the meeting end time (current HH:MM) |
| `/scheduled HH:MM` | Record the scheduled start time for the current meeting. |
| `/purpose text` | Record the purpose of the current meeting as metadata. |
| `/topic text` | Record the topic of the current note block as metadata. |
| `/goto 2026-06-05` | Jump to a specific date. |
| `/today` | Jump to today's note. |
| `/help` | Show the help overlay. |

#### Vim insert mode

Press `esc` from normal mode to edit the document directly at the cursor position. The status line shows `-- INSERT --` and the cursor is shown as a vertical bar.

| Key | Action |
|---|---|
| `←` / `→` / `↑` / `↓` | Move cursor |
| `Backspace` | Delete character before cursor |
| `Enter` | Insert newline |
| `Tab` | Insert two spaces |
| `Ctrl+W` | Delete word before cursor |
| Any character | Insert at cursor |
| `Esc` | Return to normal mode |



### Global shortcuts

| Key | Action |
|---|---|
| `Ctrl-T` | Jump to today |
| `[` / `]` | Previous / next day |


### Themes

Slugline ships with two built-in themes:

Font: Roboto

| Theme | Description |
|---|---|
| `light` (default) | Calm blue heading ramp on a clean near-white canvas |
| `dark` | Calm blue heading ramp on a deep slate-indigo canvas |

Light and dark colors should be configurable.

### Architecture
This should be web front-end talking to a Rust API and writing notes to the file system.