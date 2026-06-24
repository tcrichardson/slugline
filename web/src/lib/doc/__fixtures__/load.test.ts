import { describe, it, expect } from 'vitest';
import { fixtureLines } from './load';

describe('fixture loader', () => {
  it('reads the empty template', () => {
    const lines = fixtureLines('empty-template.md');
    expect(lines[0]).toBe('# 2026-06-23-TUE');
    expect(lines).toContain('## Meetings');
  });
});
