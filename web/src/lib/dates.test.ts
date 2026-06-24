import { describe, it, expect } from 'vitest';
import { isValidDate, addDays, todayISO, monthGrid, yearMonth } from './dates';

describe('dates', () => {
  it('validates ISO calendar dates', () => {
    expect(isValidDate('2026-06-23')).toBe(true);
    expect(isValidDate('2026-02-30')).toBe(false);
    expect(isValidDate('2026-6-23')).toBe(false);
  });

  it('adds days across month/year boundaries', () => {
    expect(addDays('2026-12-31', 1)).toBe('2027-01-01');
    expect(addDays('2026-03-01', -1)).toBe('2026-02-28');
  });

  it('formats today from a fixed date', () => {
    expect(todayISO(new Date(2026, 5, 23, 9, 0))).toBe('2026-06-23');
  });

  it('builds a 6x7 month grid with the first of month and out-of-month days', () => {
    const g = monthGrid(2026, 6);
    expect(g.length).toBe(6);
    expect(g[0].length).toBe(7);
    const flat = g.flat();
    expect(flat.find((c) => c.date === '2026-06-01')!.inMonth).toBe(true);
    expect(flat.some((c) => !c.inMonth)).toBe(true);
  });

  it('extracts year/month', () => {
    expect(yearMonth('2026-06-23')).toEqual({ year: 2026, month: 6 });
  });
});
