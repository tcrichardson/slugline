import { describe, it, expect } from 'vitest';
import { extractTodos, windowDates } from './todos';
import { fixtureLines } from './doc/__fixtures__/load';

describe('extractTodos', () => {
  it('extracts task items with done state and line indices', () => {
    const todos = extractTodos(fixtureLines('full-day.md'));
    expect(todos.map((t) => t.text)).toEqual([
      'Buy milk',
      'Send invoice',
      'Prep deck _(Weekly Sync)_',
    ]);
    expect(todos.map((t) => t.done)).toEqual([false, true, false]);
    expect(todos[0].lineIndex).toBe(4);
  });

  it('returns empty without a To Do section', () => {
    expect(extractTodos(['# T', '', '## Notes', ''])).toEqual([]);
  });
});

describe('windowDates', () => {
  it('returns 7 dates, most-recent first, ending on the active date', () => {
    const d = windowDates('2026-06-23');
    expect(d.length).toBe(7);
    expect(d[0]).toBe('2026-06-23');
    expect(d[6]).toBe('2026-06-17');
  });
});
