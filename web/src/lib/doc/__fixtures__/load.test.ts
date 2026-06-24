import { describe, it, expect } from 'vitest';
import { fixtureLines } from './load';

describe('fixture loader', () => {
  it('reads the empty template', () => {
    const lines = fixtureLines('empty-template.md');
    expect(lines[0]).toBe('# 2026-06-23-TUE');
    expect(lines).toContain('## Meetings');
  });

  it('loads all four fixture files without error', () => {
    const fixtures = ['empty-template.md', 'full-day.md', 'subsections.md', 'malformed.md'];
    for (const name of fixtures) {
      expect(() => fixtureLines(name)).not.toThrow();
    }
  });
});
