import { describe, it, expect } from 'vitest';
import { initTabs, retarget, openNewTab, closeTab, nextTab, prevTab, activeDate } from './tabs';

describe('tabs', () => {
  it('initializes with today as the only tab', () => {
    const s = initTabs('2026-06-23');
    expect(s.tabs).toEqual(['2026-06-23']);
    expect(activeDate(s)).toBe('2026-06-23');
  });

  it('retargets the active tab in place', () => {
    const s = retarget(initTabs('2026-06-23'), '2026-06-22');
    expect(s.tabs).toEqual(['2026-06-22']);
    expect(s.activeIndex).toBe(0);
  });

  it('focuses an existing tab instead of duplicating on retarget', () => {
    let s = openNewTab(initTabs('2026-06-23'), '2026-06-22'); // [23, 22] active 1
    s = retarget(s, '2026-06-23'); // 23 already open at index 0
    expect(s.tabs).toEqual(['2026-06-23', '2026-06-22']);
    expect(s.activeIndex).toBe(0);
  });

  it('opens new tabs appended right and focuses them', () => {
    const s = openNewTab(initTabs('2026-06-23'), '2026-06-24');
    expect(s.tabs).toEqual(['2026-06-23', '2026-06-24']);
    expect(s.activeIndex).toBe(1);
  });

  it('closes a tab and always keeps at least one', () => {
    let s = openNewTab(initTabs('2026-06-23'), '2026-06-24'); // [23, 24] active 1
    s = closeTab(s, 1, '2026-06-25');
    expect(s.tabs).toEqual(['2026-06-23']);
    expect(s.activeIndex).toBe(0);
    s = closeTab(s, 0, '2026-06-25'); // removing the last falls back to today
    expect(s.tabs).toEqual(['2026-06-25']);
    expect(s.activeIndex).toBe(0);
  });

  it('cycles tabs with wrap-around', () => {
    let s = openNewTab(initTabs('a'), 'b'); // [a, b] active 1
    s = nextTab(s);
    expect(s.activeIndex).toBe(0);
    s = prevTab(s);
    expect(s.activeIndex).toBe(1);
  });
});
