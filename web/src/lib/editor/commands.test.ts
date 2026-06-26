import { describe, it, expect } from 'vitest';
import { scanDocument } from '../doc/scan';
import { ensureSection, appendBlock, appendLineToSection, upsertMeta, appendMeta, endOfEnclosingSection, runCommand } from './commands';
import { createEditorState } from './state';

const TEMPLATE = ['# 2026-06-23-TUE', '', '## To Do', '', '## Meetings', '', '## Notes', ''];

describe('command helpers', () => {
  it('appendBlock adds an H3 at the end of a section', () => {
    const meetings = scanDocument(TEMPLATE).sections.find((s) => s.kind === 'meetings')!;
    const { lines, headingIndex } = appendBlock(TEMPLATE, meetings, '### Sync');
    expect(lines[headingIndex]).toBe('### Sync');
  });

  it('appendLineToSection adds after the last non-blank line', () => {
    const todo = scanDocument(TEMPLATE).sections.find((s) => s.kind === 'todo')!;
    const { lines, index } = appendLineToSection(TEMPLATE, todo, '- [ ] Buy milk');
    expect(lines[index]).toBe('- [ ] Buy milk');
    // it stays within the To Do section (before ## Meetings)
    expect(lines.indexOf('## Meetings')).toBeGreaterThan(index);
  });

  it('upsertMeta inserts then updates in place', () => {
    let lines = ['## Meetings', '### Sync', ''];
    let block = scanDocument(lines).sections[0].blocks[0];
    ({ lines } = upsertMeta(lines, block, 'scheduled', '14:30'));
    expect(lines).toContain('meta:scheduled 14:30');
    block = scanDocument(lines).sections[0].blocks[0];
    ({ lines } = upsertMeta(lines, block, 'scheduled', '15:00'));
    expect(lines.filter((l) => l.startsWith('meta:scheduled')).length).toBe(1);
    expect(lines).toContain('meta:scheduled 15:00');
  });

  it('ensureSection recreates a missing section in canonical order', () => {
    const { lines, section } = ensureSection(['# T', '', '## Notes', ''], 'meetings');
    const kinds = scanDocument(lines).sections.map((s) => s.kind);
    expect(kinds).toContain('meetings');
    expect(kinds.indexOf('meetings')).toBeLessThan(kinds.indexOf('notes'));
    expect(section.kind).toBe('meetings');
  });

  it('endOfEnclosingSection finds the next same/shallower heading', () => {
    expect(endOfEnclosingSection(['### A', 'body', '### B'], 1, 3)).toBe(2);
  });
});

describe('appendMeta', () => {
  it('inserts meta:people when no prior value exists', () => {
    const lines = ['## Meetings', '### Sync', ''];
    const block = scanDocument(lines).sections[0].blocks[0];
    const { lines: out } = appendMeta(lines, block, 'people', 'Alice');
    expect(out).toContain('meta:people Alice');
  });

  it('appends comma-separated to an existing meta:people value', () => {
    const lines = ['## Meetings', '### Sync', 'meta:people Alice', ''];
    const block = scanDocument(lines).sections[0].blocks[0];
    const { lines: out } = appendMeta(lines, block, 'people', 'Bob');
    expect(out).toContain('meta:people Alice, Bob');
    expect(out.filter((l) => l.startsWith('meta:people')).length).toBe(1);
  });

  it('trims whitespace from the new value before appending', () => {
    const lines = ['## Meetings', '### Sync', 'meta:people Alice', ''];
    const block = scanDocument(lines).sections[0].blocks[0];
    const { lines: out } = appendMeta(lines, block, 'people', '  Bob  ');
    expect(out).toContain('meta:people Alice, Bob');
  });
});

const TPL = ['# 2026-06-23-TUE', '', '## To Do', '', '## Meetings', '', '## Notes', ''];
const withCmd = (lines: string[], cmd: string, line = 0) => ({
  ...createEditorState(lines),
  command: cmd,
  cursor: { line, col: 0 },
});
const ctx = { nowHHMM: '09:30' };

describe('runCommand', () => {
  it('reports unknown commands and clears the command line', () => {
    const r = runCommand(withCmd(TPL, 'meetng x'), ctx);
    expect(r.state.command).toBeNull();
    expect(r.state.message).toContain('Unknown command');
  });

  it(':meeting adds a heading and moves the cursor to it', () => {
    const r = runCommand(withCmd(TPL, 'meeting Daily Standup'), ctx);
    expect(r.state.lines[r.state.cursor.line]).toBe('### Daily Standup');
  });

  it(':todo appends to To Do and keeps the cursor', () => {
    const r = runCommand(withCmd(TPL, 'todo Buy milk'), ctx);
    expect(r.state.lines).toContain('- [ ] Buy milk');
    expect(r.state.cursor.line).toBe(0);
  });

  it(':todo inside a meeting tags the meeting name', () => {
    const lines = ['# T', '', '## To Do', '', '## Meetings', '### Sync', '', '## Notes', ''];
    const r = runCommand({ ...createEditorState(lines), command: 'todo Prep', cursor: { line: 5, col: 0 } }, ctx);
    expect(r.state.lines.some((l) => l === '- [ ] Prep _(Sync)_')).toBe(true);
  });

  it(':scheduled errors when not in a meeting', () => {
    const r = runCommand(withCmd(TPL, 'scheduled 14:30', 2), ctx);
    expect(r.state.message).toBe('Not in a meeting');
  });

  it(':start records the current time on the enclosing meeting', () => {
    const lines = ['# T', '', '## Meetings', '### Sync', '', '## Notes', ''];
    const r = runCommand({ ...createEditorState(lines), command: 'start', cursor: { line: 4, col: 0 } }, ctx);
    expect(r.state.lines).toContain('meta:started 09:30');
  });

  it(':section nests one level deeper than the enclosing heading', () => {
    const lines = ['## Meetings', '### Sync', 'body', '## Notes', ''];
    const r = runCommand({ ...createEditorState(lines), command: 'section Risks', cursor: { line: 2, col: 0 } }, ctx);
    expect(r.state.lines.some((l) => l === '#### Risks')).toBe(true);
  });

  it(':goto emits an effect without mutating the buffer', () => {
    const r = runCommand(withCmd(TPL, 'goto 2026-06-01'), ctx);
    expect(r.effect).toEqual({ type: 'goto', date: '2026-06-01' });
    expect(r.state.lines).toEqual(TPL);
  });
});

describe(':people command', () => {
  // Cursor at line 4 = body of "### Sync" meeting
  const meetingLines = ['# T', '', '## Meetings', '### Sync', '', '## Notes', ''];
  // Cursor at line 6 = body of "### Retro" note
  const noteLines = ['# T', '', '## Meetings', '', '## Notes', '### Retro', '', ''];

  it('sets meta:people in a meeting block', () => {
    const r = runCommand(
      { ...createEditorState(meetingLines), command: 'people Alice', cursor: { line: 4, col: 0 } },
      ctx,
    );
    expect(r.state.lines).toContain('meta:people Alice');
    expect(r.state.message).toBe('');
  });

  it('appends to existing meta:people in a meeting block', () => {
    const lines = ['# T', '', '## Meetings', '### Sync', 'meta:people Alice', '', '## Notes', ''];
    const r = runCommand(
      { ...createEditorState(lines), command: 'people Bob', cursor: { line: 5, col: 0 } },
      ctx,
    );
    expect(r.state.lines).toContain('meta:people Alice, Bob');
  });

  it('sets meta:people in a note block', () => {
    const r = runCommand(
      { ...createEditorState(noteLines), command: 'people Alice', cursor: { line: 6, col: 0 } },
      ctx,
    );
    expect(r.state.lines).toContain('meta:people Alice');
    expect(r.state.message).toBe('');
  });

  it('errors when not in a meeting or note block', () => {
    // TPL cursor line 2 = "## To Do" section heading, not inside any block
    const r = runCommand(withCmd(TPL, 'people Alice', 2), ctx);
    expect(r.state.message).toContain('meeting or note');
  });

  it(':p shortcut works end-to-end through runCommand', () => {
    const r = runCommand(
      { ...createEditorState(meetingLines), command: 'p Alice', cursor: { line: 4, col: 0 } },
      ctx,
    );
    expect(r.state.lines).toContain('meta:people Alice');
  });
});
