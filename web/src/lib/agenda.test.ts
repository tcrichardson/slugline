import { describe, it, expect } from 'vitest';
import { deriveAgenda } from './agenda';
import { fixtureLines } from './doc/__fixtures__/load';

describe('deriveAgenda', () => {
  it('lists scheduled meetings sorted by time', () => {
    const items = deriveAgenda(fixtureLines('full-day.md'));
    expect(items.map((i) => i.name)).toEqual(['Standup', 'Weekly Sync']);
    expect(items[0].time).toBe('09:00');
  });

  it('captures started/ended status when present', () => {
    const sync = deriveAgenda(fixtureLines('full-day.md')).find((i) => i.name === 'Weekly Sync')!;
    expect(sync.ended).toBe('15:02');
  });

  it('omits meetings without a scheduled time', () => {
    const lines = ['## Meetings', '### A', 'meta:scheduled 10:00', '### B', ''];
    expect(deriveAgenda(lines).map((i) => i.name)).toEqual(['A']);
  });

  it('returns empty when there is no Meetings section', () => {
    expect(deriveAgenda(['# T', '', '## Notes', ''])).toEqual([]);
  });
});
