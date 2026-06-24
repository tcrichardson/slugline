import { describe, it, expect } from 'vitest';
import { createEditorState } from './state';
import {
  enterInsert,
  enterInsertAfter,
  openBelow,
  exitInsert,
  insertText,
  insertNewline,
  backspace,
  deleteWordBefore,
} from './insert';

const at = (lines: string[], line = 0, col = 0) => ({
  ...createEditorState(lines),
  cursor: { line, col },
});
const ins = (lines: string[], line = 0, col = 0) => ({ ...at(lines, line, col), mode: 'insert' as const });

describe('insert ops', () => {
  it('enterInsert switches mode and pushes one undo snapshot', () => {
    const s = enterInsert(at(['abc'], 0, 1));
    expect(s.mode).toBe('insert');
    expect(s.undo.length).toBe(1);
  });

  it('enterInsertAfter moves one past the cursor', () => {
    const s = enterInsertAfter(at(['abc'], 0, 1));
    expect(s.cursor.col).toBe(2);
  });

  it('insertText inserts at the cursor and advances', () => {
    const s = insertText(ins(['ac'], 0, 1), 'b');
    expect(s.lines[0]).toBe('abc');
    expect(s.cursor.col).toBe(2);
  });

  it('insertNewline splits the line', () => {
    const s = insertNewline(ins(['ab'], 0, 1));
    expect(s.lines).toEqual(['a', 'b']);
    expect(s.cursor).toEqual({ line: 1, col: 0 });
  });

  it('backspace joins lines at col 0', () => {
    const s = backspace(ins(['a', 'b'], 1, 0));
    expect(s.lines).toEqual(['ab']);
    expect(s.cursor).toEqual({ line: 0, col: 1 });
  });

  it('openBelow inserts a blank line and enters insert', () => {
    const s = openBelow(at(['a'], 0, 0));
    expect(s.lines).toEqual(['a', '']);
    expect(s.mode).toBe('insert');
    expect(s.cursor).toEqual({ line: 1, col: 0 });
  });

  it('deleteWordBefore removes the previous word', () => {
    const s = deleteWordBefore(ins(['foo bar'], 0, 7));
    expect(s.lines[0]).toBe('foo ');
  });

  it('exitInsert clamps the cursor back into normal bounds', () => {
    const s = exitInsert(ins(['abc'], 0, 3));
    expect(s.mode).toBe('normal');
    expect(s.cursor.col).toBe(2);
  });
});
