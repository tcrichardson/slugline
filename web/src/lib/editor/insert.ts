import type { EditorState } from './state';
import { clampCursor, pushUndo } from './state';

// Mode entry: each pushes ONE undo snapshot for the whole insert session.
export function enterInsert(s: EditorState): EditorState {
  return { ...pushUndo(s), mode: 'insert' };
}
export function enterInsertAfter(s: EditorState): EditorState {
  const ns = pushUndo(s);
  const text = ns.lines[ns.cursor.line] ?? '';
  const col = Math.min(ns.cursor.col + (text.length > 0 ? 1 : 0), text.length);
  return { ...ns, mode: 'insert', cursor: { ...ns.cursor, col } };
}
export function enterInsertLineEnd(s: EditorState): EditorState {
  const ns = pushUndo(s);
  const text = ns.lines[ns.cursor.line] ?? '';
  return { ...ns, mode: 'insert', cursor: { ...ns.cursor, col: text.length } };
}
export function openBelow(s: EditorState): EditorState {
  const ns = pushUndo(s);
  const lines = ns.lines.slice();
  lines.splice(ns.cursor.line + 1, 0, '');
  return { ...ns, lines, mode: 'insert', cursor: { line: ns.cursor.line + 1, col: 0 } };
}
export function openAbove(s: EditorState): EditorState {
  const ns = pushUndo(s);
  const lines = ns.lines.slice();
  lines.splice(ns.cursor.line, 0, '');
  return { ...ns, lines, mode: 'insert', cursor: { line: ns.cursor.line, col: 0 } };
}

export function exitInsert(s: EditorState): EditorState {
  return clampCursor({ ...s, mode: 'normal' });
}

// In-session edits: NO undo push (the session snapshot was taken on entry).
export function insertText(s: EditorState, ch: string): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const lines = s.lines.slice();
  lines[s.cursor.line] = text.slice(0, s.cursor.col) + ch + text.slice(s.cursor.col);
  return { ...s, lines, cursor: { ...s.cursor, col: s.cursor.col + ch.length } };
}
export function insertTab(s: EditorState): EditorState {
  return insertText(s, '  ');
}
export function insertNewline(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const lines = s.lines.slice();
  lines.splice(s.cursor.line, 1, text.slice(0, s.cursor.col), text.slice(s.cursor.col));
  return { ...s, lines, cursor: { line: s.cursor.line + 1, col: 0 } };
}
export function backspace(s: EditorState): EditorState {
  const { line, col } = s.cursor;
  if (col > 0) {
    const text = s.lines[line];
    const lines = s.lines.slice();
    lines[line] = text.slice(0, col - 1) + text.slice(col);
    return { ...s, lines, cursor: { line, col: col - 1 } };
  }
  if (line > 0) {
    const prev = s.lines[line - 1];
    const lines = s.lines.slice();
    lines.splice(line - 1, 2, prev + s.lines[line]);
    return { ...s, lines, cursor: { line: line - 1, col: prev.length } };
  }
  return s;
}
export function deleteWordBefore(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  let i = s.cursor.col;
  while (i > 0 && /\s/.test(text[i - 1])) i--;
  while (i > 0 && !/\s/.test(text[i - 1])) i--;
  const lines = s.lines.slice();
  lines[s.cursor.line] = text.slice(0, i) + text.slice(s.cursor.col);
  return { ...s, lines, cursor: { ...s.cursor, col: i } };
}
