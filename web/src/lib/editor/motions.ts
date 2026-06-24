import type { EditorState } from './state';
import { clampCursor } from './state';

export function moveLeft(s: EditorState): EditorState {
  return clampCursor({ ...s, cursor: { ...s.cursor, col: s.cursor.col - 1 } });
}
export function moveRight(s: EditorState): EditorState {
  return clampCursor({ ...s, cursor: { ...s.cursor, col: s.cursor.col + 1 } });
}
export function moveUp(s: EditorState): EditorState {
  return clampCursor({ ...s, cursor: { line: Math.max(0, s.cursor.line - 1), col: s.cursor.col } });
}
export function moveDown(s: EditorState): EditorState {
  const line = Math.min(s.lines.length - 1, s.cursor.line + 1);
  return clampCursor({ ...s, cursor: { line, col: s.cursor.col } });
}
export function lineStart(s: EditorState): EditorState {
  return { ...s, cursor: { ...s.cursor, col: 0 } };
}
export function lineEnd(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const maxCol = s.mode === 'insert' ? text.length : Math.max(0, text.length - 1);
  return { ...s, cursor: { ...s.cursor, col: maxCol } };
}
export function firstLine(s: EditorState): EditorState {
  return clampCursor({ ...s, cursor: { line: 0, col: s.cursor.col } });
}
export function lastLine(s: EditorState): EditorState {
  return clampCursor({ ...s, cursor: { line: s.lines.length - 1, col: s.cursor.col } });
}

// vim "word": a run of word chars (\w) OR a run of punctuation, separated by whitespace.
function classOf(ch: string | undefined): number {
  if (ch === undefined || /\s/.test(ch)) return 0;
  if (/\w/.test(ch)) return 1;
  return 2;
}

export function wordForward(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const n = text.length;
  let i = s.cursor.col;
  if (i >= n) return s;
  const startClass = classOf(text[i]);
  if (startClass !== 0) while (i < n && classOf(text[i]) === startClass) i++;
  while (i < n && classOf(text[i]) === 0) i++;
  return { ...s, cursor: { ...s.cursor, col: Math.min(i, Math.max(0, n - 1)) } };
}

export function wordBackward(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  let i = s.cursor.col;
  if (i <= 0) return s;
  i--;
  while (i > 0 && classOf(text[i]) === 0) i--;
  const cl = classOf(text[i]);
  while (i > 0 && classOf(text[i - 1]) === cl && cl !== 0) i--;
  return { ...s, cursor: { ...s.cursor, col: Math.max(0, i) } };
}

export function wordEnd(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const n = text.length;
  let i = s.cursor.col;
  if (i >= n - 1) return s;
  i++;
  while (i < n && classOf(text[i]) === 0) i++;
  const cl = classOf(text[i]);
  while (i < n - 1 && classOf(text[i + 1]) === cl && cl !== 0) i++;
  return { ...s, cursor: { ...s.cursor, col: Math.min(i, n - 1) } };
}
