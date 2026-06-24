import { describe, it, expect } from 'vitest';
import { createEditorState } from './state';
import { deleteChar, deleteLine, yankLine, pasteBelow, pasteAbove, toggleTodo } from './edits';

const at = (lines: string[], line = 0, col = 0) => ({
  ...createEditorState(lines),
  cursor: { line, col },
});

describe('normal edits', () => {
  it('x deletes the char under the cursor', () => {
    expect(deleteChar(at(['abc'], 0, 1)).lines[0]).toBe('ac');
  });

  it('dd deletes and yanks the line', () => {
    const s = deleteLine(at(['a', 'b'], 0, 0));
    expect(s.lines).toEqual(['b']);
    expect(s.register).toEqual(['a']);
    expect(s.undo.length).toBe(1);
  });

  it('yy yanks; p pastes below', () => {
    let s = yankLine(at(['a', 'b'], 0, 0));
    s = { ...s, cursor: { line: 1, col: 0 } };
    s = pasteBelow(s);
    expect(s.lines).toEqual(['a', 'b', 'a']);
  });

  it('P pastes above', () => {
    let s = yankLine(at(['a', 'b'], 1, 0));
    s = pasteAbove({ ...s, cursor: { line: 0, col: 0 } });
    expect(s.lines).toEqual(['b', 'a', 'b']);
  });

  it('t toggles a task item only', () => {
    expect(toggleTodo(at(['- [ ] x'], 0, 0)).lines[0]).toBe('- [x] x');
    expect(toggleTodo(at(['- [x] x'], 0, 0)).lines[0]).toBe('- [ ] x');
    expect(toggleTodo(at(['plain'], 0, 0)).lines[0]).toBe('plain');
  });
});
