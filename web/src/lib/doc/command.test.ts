import { describe, it, expect } from 'vitest';
import { parseCommandLine, validateCommand, isValidDate } from './command';

describe('parseCommandLine', () => {
  it('splits name and rest-of-line argument', () => {
    expect(parseCommandLine('meeting Daily Standup')).toEqual({ name: 'meeting', arg: 'Daily Standup' });
  });

  it('lowercases the name and handles no-arg commands', () => {
    expect(parseCommandLine('Today')).toEqual({ name: 'today', arg: '' });
  });
});

describe('validateCommand', () => {
  it('accepts a valid text command', () => {
    expect(validateCommand('meeting Weekly Sync')).toEqual({ ok: true, command: 'meeting', arg: 'Weekly Sync' });
  });

  it('rejects unknown commands', () => {
    const r = validateCommand('meetng x');
    expect(r.ok).toBe(false);
    if (!r.ok) expect(r.error).toContain('Unknown command');
  });

  it('requires arguments where mandated', () => {
    const r = validateCommand('todo');
    expect(r.ok).toBe(false);
  });

  it('validates HH:MM for scheduled', () => {
    expect(validateCommand('scheduled 14:30').ok).toBe(true);
    expect(validateCommand('scheduled 25:00').ok).toBe(false);
  });

  it('validates YYYY-MM-DD for goto', () => {
    expect(validateCommand('goto 2026-06-23').ok).toBe(true);
    expect(validateCommand('goto 2026-13-01').ok).toBe(false);
  });

  it('validates theme values', () => {
    expect(validateCommand('theme dark').ok).toBe(true);
    expect(validateCommand('theme neon').ok).toBe(false);
  });

  it('accepts no-arg commands', () => {
    expect(validateCommand('start').ok).toBe(true);
    expect(validateCommand('close').ok).toBe(true);
  });
});

describe('isValidDate', () => {
  it('rejects impossible calendar dates', () => {
    expect(isValidDate('2026-02-30')).toBe(false);
    expect(isValidDate('2026-02-28')).toBe(true);
  });
});
