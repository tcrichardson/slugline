import type { EditorState } from './state';
import { clampCursor, pushUndo } from './state';

export function deleteChar(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  if (text.length === 0) return s;
  const ns = pushUndo(s);
  const lines = s.lines.slice();
  lines[s.cursor.line] = text.slice(0, s.cursor.col) + text.slice(s.cursor.col + 1);
  return clampCursor({ ...ns, lines });
}

export function deleteLine(s: EditorState): EditorState {
  const ns = pushUndo(s);
  const lines = s.lines.slice();
  const removed = lines.splice(s.cursor.line, 1);
  if (lines.length === 0) lines.push('');
  const line = Math.min(s.cursor.line, lines.length - 1);
  return clampCursor({ ...ns, lines, register: removed, cursor: { line, col: 0 } });
}

export function yankLine(s: EditorState): EditorState {
  return { ...s, register: [s.lines[s.cursor.line] ?? ''], message: '1 line yanked' };
}

export function pasteBelow(s: EditorState): EditorState {
  if (s.register.length === 0) return s;
  const ns = pushUndo(s);
  const lines = s.lines.slice();
  lines.splice(s.cursor.line + 1, 0, ...s.register);
  return clampCursor({ ...ns, lines, cursor: { line: s.cursor.line + 1, col: 0 } });
}

export function pasteAbove(s: EditorState): EditorState {
  if (s.register.length === 0) return s;
  const ns = pushUndo(s);
  const lines = s.lines.slice();
  lines.splice(s.cursor.line, 0, ...s.register);
  return clampCursor({ ...ns, lines, cursor: { line: s.cursor.line, col: 0 } });
}

const TASK_RE = /^(\s*- \[)([ xX])(\] )/;

export function toggleTodo(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const m = TASK_RE.exec(text);
  if (!m) return s; // no-op on non-task lines
  const ns = pushUndo(s);
  const next = m[2] === ' ' ? 'x' : ' ';
  const lines = s.lines.slice();
  lines[s.cursor.line] = text.replace(TASK_RE, `$1${next}$3`);
  return { ...ns, lines };
}
