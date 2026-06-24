export type Mode = 'normal' | 'insert';

export interface Cursor {
  line: number;
  col: number;
}

export interface Snapshot {
  lines: string[];
  cursor: Cursor;
}

export type Pending = '' | 'g' | 'd' | 'y';

export interface EditorState {
  lines: string[];
  cursor: Cursor;
  mode: Mode;
  register: string[]; // line-wise; shared across tabs by the store
  pending: Pending;
  command: string | null; // command-line buffer (text after ':'), null when inactive
  message: string;
  undo: Snapshot[];
  redo: Snapshot[];
}

export function createEditorState(lines: string[], register: string[] = []): EditorState {
  return {
    lines: lines.length > 0 ? lines.slice() : [''],
    cursor: { line: 0, col: 0 },
    mode: 'normal',
    register,
    pending: '',
    command: null,
    message: '',
    undo: [],
    redo: [],
  };
}

export function snapshot(s: EditorState): Snapshot {
  return { lines: s.lines.slice(), cursor: { ...s.cursor } };
}

/** Snapshot the pre-mutation state and clear redo. Call BEFORE applying a mutation. */
export function pushUndo(s: EditorState): EditorState {
  return { ...s, undo: [...s.undo, snapshot(s)], redo: [] };
}

export function undo(s: EditorState): EditorState {
  if (s.undo.length === 0) return { ...s, message: 'Already at oldest change' };
  const prev = s.undo[s.undo.length - 1];
  return {
    ...s,
    lines: prev.lines.slice(),
    cursor: { ...prev.cursor },
    undo: s.undo.slice(0, -1),
    redo: [...s.redo, snapshot(s)],
    message: '',
  };
}

export function redo(s: EditorState): EditorState {
  if (s.redo.length === 0) return { ...s, message: 'Already at newest change' };
  const next = s.redo[s.redo.length - 1];
  return {
    ...s,
    lines: next.lines.slice(),
    cursor: { ...next.cursor },
    redo: s.redo.slice(0, -1),
    undo: [...s.undo, snapshot(s)],
    message: '',
  };
}

/** Clamp cursor to valid bounds for the current mode. */
export function clampCursor(s: EditorState): EditorState {
  const line = Math.max(0, Math.min(s.cursor.line, s.lines.length - 1));
  const text = s.lines[line] ?? '';
  const maxCol = s.mode === 'insert' ? text.length : Math.max(0, text.length - 1);
  const col = Math.max(0, Math.min(s.cursor.col, maxCol));
  return { ...s, cursor: { line, col } };
}
