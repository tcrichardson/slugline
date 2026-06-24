import { describe, it, expect } from 'vitest';
import { scanDocument } from './scan';
import { resolveContext, nearestHeadingLevel } from './context';
import { fixtureLines } from './__fixtures__/load';

describe('resolveContext', () => {
  const lines = fixtureLines('full-day.md');
  const model = scanDocument(lines);

  it('returns the meeting when the cursor is inside an H3 under Meetings', () => {
    const meetings = model.sections.find((s) => s.kind === 'meetings')!;
    const sync = meetings.blocks[0];
    const ctx = resolveContext(model, sync.headingLineIndex + 1);
    expect(ctx.kind).toBe('meeting');
    if (ctx.kind === 'meeting') expect(ctx.block.name).toBe('Weekly Sync');
  });

  it('returns the note when the cursor is inside an H3 under Notes', () => {
    const notes = model.sections.find((s) => s.kind === 'notes')!;
    const arch = notes.blocks[0];
    const ctx = resolveContext(model, arch.headingLineIndex + 1);
    expect(ctx.kind).toBe('note');
  });

  it('returns todo when the cursor is inside the To Do section', () => {
    const todo = model.sections.find((s) => s.kind === 'todo')!;
    const ctx = resolveContext(model, todo.headingLineIndex + 1);
    expect(ctx.kind).toBe('todo');
  });

  it('returns none when the cursor is on the title line', () => {
    expect(resolveContext(model, 0).kind).toBe('none');
  });
});

describe('nearestHeadingLevel', () => {
  it('finds the level of the nearest enclosing heading', () => {
    const lines = fixtureLines('subsections.md');
    // Inside the "Mitigations" (H5) area -> nearest heading level is 5.
    const idx = lines.indexOf('Cut scope.');
    expect(nearestHeadingLevel(lines, idx)).toBe(5);
  });

  it('returns null above any heading', () => {
    expect(nearestHeadingLevel(['', 'no heading yet'], 1)).toBeNull();
  });
});
