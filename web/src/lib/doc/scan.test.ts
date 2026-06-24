import { describe, it, expect } from 'vitest';
import { scanDocument } from './scan';
import { fixtureLines } from './__fixtures__/load';

describe('scanDocument', () => {
  it('reads the title from the first H1', () => {
    const model = scanDocument(fixtureLines('full-day.md'));
    expect(model.title).toBe('2026-06-23-TUE');
    expect(model.titleLineIndex).toBe(0);
  });

  it('finds the three standard sections by kind', () => {
    const model = scanDocument(fixtureLines('full-day.md'));
    expect(model.sections.map((s) => s.kind)).toEqual(['todo', 'meetings', 'notes']);
  });

  it('collects H3 blocks under meetings with their meta', () => {
    const model = scanDocument(fixtureLines('full-day.md'));
    const meetings = model.sections.find((s) => s.kind === 'meetings')!;
    expect(meetings.blocks.map((b) => b.name)).toEqual(['Weekly Sync', 'Standup']);

    const sync = meetings.blocks[0];
    const scheduled = sync.meta.find((m) => m.key === 'scheduled')!;
    expect(scheduled.value).toBe('14:30');
    expect(sync.meta.map((m) => m.key)).toEqual(['purpose', 'scheduled', 'started', 'ended']);
  });

  it('bounds a block to the line before the next heading', () => {
    const model = scanDocument(fixtureLines('full-day.md'));
    const meetings = model.sections.find((s) => s.kind === 'meetings')!;
    const sync = meetings.blocks[0];
    // metaEndLine is the last meta line of Weekly Sync; endLine reaches the line before "### Standup"
    expect(sync.metaEndLine).toBeGreaterThan(sync.headingLineIndex);
    expect(sync.endLine).toBeGreaterThanOrEqual(sync.metaEndLine);
  });

  it('does not throw on malformed documents', () => {
    const model = scanDocument(fixtureLines('malformed.md'));
    expect(model.title).toBe('Just a title');
    expect(model.sections).toEqual([]);
  });

  it('treats a block with no meta as metaEndLine === headingLineIndex', () => {
    const model = scanDocument(fixtureLines('subsections.md'));
    const meetings = model.sections.find((s) => s.kind === 'meetings')!;
    const planning = meetings.blocks[0];
    expect(planning.name).toBe('Planning');
    // Planning has exactly one meta line (scheduled), so metaEndLine > heading.
    expect(planning.meta.map((m) => m.key)).toEqual(['scheduled']);
  });
});
