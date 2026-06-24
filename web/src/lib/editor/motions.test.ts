import { describe, it, expect } from 'vitest';
import { createEditorState } from './state';
import { moveRight, moveDown, lineEnd, lastLine, wordForward, wordBackward, wordEnd } from './motions';

const at = (lines: string[], line: number, col: number) => ({
  ...createEditorState(lines),
  cursor: { line, col },
});

describe('motions', () => {
  it('moveRight stops at the last char in normal mode', () => {
    expect(moveRight(at(['ab'], 0, 1)).cursor.col).toBe(1);
  });

  it('moveDown stays within bounds', () => {
    expect(moveDown(at(['a', 'b'], 1, 0)).cursor.line).toBe(1);
  });

  it('lineEnd goes to the last char (normal)', () => {
    expect(lineEnd(at(['hello'], 0, 0)).cursor.col).toBe(4);
  });

  it('lastLine jumps to the final line', () => {
    expect(lastLine(at(['a', 'b', 'c'], 0, 0)).cursor.line).toBe(2);
  });

  it('wordForward jumps to the next word start', () => {
    expect(wordForward(at(['foo bar'], 0, 0)).cursor.col).toBe(4);
  });

  it('wordBackward jumps to the previous word start', () => {
    expect(wordBackward(at(['foo bar'], 0, 4)).cursor.col).toBe(0);
  });

  it('wordEnd jumps to the end of the next word', () => {
    expect(wordEnd(at(['foo bar'], 0, 0)).cursor.col).toBe(2);
  });
});
