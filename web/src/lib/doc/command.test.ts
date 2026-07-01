import { describe, it, expect } from 'vitest';
import { parseCommandLine, validateCommand } from './command';

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

  it('allows :theme with no argument (toggle)', () => {
    const r = validateCommand('theme');
    expect(r.ok).toBe(true);
    if (r.ok) expect(r.arg).toBe('');
  });
});

describe('ALIASES and :people command', () => {
  it(':p Alice resolves to command people via validateCommand', () => {
    const r = validateCommand('p Alice Smith');
    expect(r.ok).toBe(true);
    if (r.ok) {
      expect(r.command).toBe('people');
      expect(r.arg).toBe('Alice Smith');
    }
  });

  it(':p with no argument fails validation', () => {
    const r = validateCommand('p');
    expect(r.ok).toBe(false);
  });

  it(':people resolves directly', () => {
    const r = validateCommand('people Bob Jones');
    expect(r.ok).toBe(true);
    if (r.ok) {
      expect(r.command).toBe('people');
      expect(r.arg).toBe('Bob Jones');
    }
  });
});
