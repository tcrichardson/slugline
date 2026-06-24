import { describe, it, expect } from 'vitest';
import { createEditorState, pushUndo, undo, redo, clampCursor } from './state';

describe('editor state', () => {
  it('never has zero lines', () => {
    expect(createEditorState([]).lines).toEqual(['']);
  });

  it('undo/redo round-trips a line change', () => {
    let s = createEditorState(['a', 'b']);
    s = pushUndo(s);
    s = { ...s, lines: ['a', 'b', 'c'] };
    s = undo(s);
    expect(s.lines).toEqual(['a', 'b']);
    s = redo(s);
    expect(s.lines).toEqual(['a', 'b', 'c']);
  });

  it('clamps cursor col to length-1 in normal mode', () => {
    const s = clampCursor({ ...createEditorState(['abc']), cursor: { line: 0, col: 99 } });
    expect(s.cursor.col).toBe(2);
  });

  it('allows cursor col at length in insert mode', () => {
    const base = createEditorState(['abc']);
    const s = clampCursor({ ...base, mode: 'insert', cursor: { line: 0, col: 99 } });
    expect(s.cursor.col).toBe(3);
  });

  it('preserves a provided register (shared across tabs)', () => {
    expect(createEditorState(['x'], ['line']).register).toEqual(['line']);
  });
});
