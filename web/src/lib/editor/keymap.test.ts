import { describe, it, expect } from 'vitest';
import { createEditorState } from './state';
import { handleKey, type KeyInput } from './keymap';

const k = (key: string, mods: Partial<KeyInput> = {}): KeyInput => ({
  key,
  ctrl: false,
  meta: false,
  shift: false,
  ...mods,
});
const ctx = { nowHHMM: '09:30' };

describe('handleKey', () => {
  it('j moves down in normal mode', () => {
    expect(handleKey(createEditorState(['a', 'b']), k('j'), ctx).state.cursor.line).toBe(1);
  });

  it('i enters insert, typing inserts, Escape exits', () => {
    let s = handleKey(createEditorState(['']), k('i'), ctx).state;
    expect(s.mode).toBe('insert');
    s = handleKey(s, k('x'), ctx).state;
    expect(s.lines[0]).toBe('x');
    s = handleKey(s, k('Escape'), ctx).state;
    expect(s.mode).toBe('normal');
  });

  it('dd deletes a line via the pending operator', () => {
    let s = createEditorState(['a', 'b']);
    s = handleKey(s, k('d'), ctx).state;
    s = handleKey(s, k('d'), ctx).state;
    expect(s.lines).toEqual(['b']);
  });

  it('gg jumps to the first line', () => {
    let s = { ...createEditorState(['a', 'b', 'c']), cursor: { line: 2, col: 0 } };
    s = handleKey(s, k('g'), ctx).state;
    s = handleKey(s, k('g'), ctx).state;
    expect(s.cursor.line).toBe(0);
  });

  it(': opens the command line and Enter runs it', () => {
    let s = createEditorState(['# T', '', '## To Do', '']);
    s = handleKey(s, k(':'), ctx).state;
    expect(s.command).toBe('');
    for (const ch of ['t', 'o', 'd', 'o', ' ', 'm']) s = handleKey(s, k(ch), ctx).state;
    const r = handleKey(s, k('Enter'), ctx);
    expect(r.state.lines).toContain('- [ ] m');
    expect(r.state.command).toBeNull();
  });

  it('gt emits a tabNext effect', () => {
    let s = createEditorState(['a']);
    s = handleKey(s, k('g'), ctx).state;
    expect(handleKey(s, k('t'), ctx).effect).toEqual({ type: 'tabNext' });
  });

  it('] emits nextDay; Ctrl-r redoes', () => {
    expect(handleKey(createEditorState(['a']), k(']'), ctx).effect).toEqual({ type: 'nextDay' });
    expect(handleKey(createEditorState(['a']), k('r', { ctrl: true }), ctx).state.message).toContain('newest');
  });
});
