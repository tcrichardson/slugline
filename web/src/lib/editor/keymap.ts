import type { EditorState } from './state';
import { undo, redo } from './state';
import * as M from './motions';
import * as E from './edits';
import * as I from './insert';
import { runCommand, type AppEffect, type CommandCtx } from './commands';

export interface KeyInput {
  key: string;
  ctrl: boolean;
  meta: boolean;
  shift: boolean;
}
export interface KeyResult {
  state: EditorState;
  effect?: AppEffect;
}

export function handleKey(state: EditorState, key: KeyInput, ctx: CommandCtx): KeyResult {
  if (state.command !== null) return handleCommandMode(state, key, ctx);
  if (state.mode === 'insert') return handleInsertMode(state, key);
  return handleNormalMode(state, key, ctx);
}

function handleCommandMode(state: EditorState, key: KeyInput, ctx: CommandCtx): KeyResult {
  if (key.key === 'Escape') return { state: { ...state, command: null, message: '' } };
  if (key.key === 'Enter') return runCommand(state, ctx);
  if (key.key === 'Backspace') return { state: { ...state, command: (state.command ?? '').slice(0, -1) } };
  if (key.key.length === 1 && !key.ctrl && !key.meta) {
    return { state: { ...state, command: (state.command ?? '') + key.key } };
  }
  return { state };
}

function handleInsertMode(state: EditorState, key: KeyInput): KeyResult {
  switch (key.key) {
    case 'Escape':
      return { state: I.exitInsert(state) };
    case 'Backspace':
      return { state: I.backspace(state) };
    case 'Enter':
      return { state: I.insertNewline(state) };
    case 'Tab':
      return { state: I.insertTab(state) };
    case 'ArrowLeft':
      return { state: M.moveLeft(state) };
    case 'ArrowRight':
      return { state: M.moveRight(state) };
    case 'ArrowUp':
      return { state: M.moveUp(state) };
    case 'ArrowDown':
      return { state: M.moveDown(state) };
  }
  if (key.ctrl && (key.key === 'w' || key.key === 'W')) return { state: I.deleteWordBefore(state) };
  if (key.key.length === 1 && !key.ctrl && !key.meta) return { state: I.insertText(state, key.key) };
  return { state };
}

function handleNormalMode(state: EditorState, key: KeyInput, ctx: CommandCtx): KeyResult {
  if (state.pending === 'g') {
    const s = { ...state, pending: '' as const };
    if (key.key === 'g') return { state: M.firstLine(s) };
    if (key.key === 't') return { state: s, effect: { type: 'tabNext' } };
    if (key.key === 'T') return { state: s, effect: { type: 'tabPrev' } };
    return handleNormalMode(s, key, ctx);
  }
  if (state.pending === 'd') {
    const s = { ...state, pending: '' as const };
    if (key.key === 'd') return { state: E.deleteLine(s) };
    return handleNormalMode(s, key, ctx);
  }
  if (state.pending === 'y') {
    const s = { ...state, pending: '' as const };
    if (key.key === 'y') return { state: E.yankLine(s) };
    return handleNormalMode(s, key, ctx);
  }

  switch (key.key) {
    case 'h':
    case 'ArrowLeft':
      return { state: M.moveLeft(state) };
    case 'l':
    case 'ArrowRight':
      return { state: M.moveRight(state) };
    case 'j':
    case 'ArrowDown':
      return { state: M.moveDown(state) };
    case 'k':
    case 'ArrowUp':
      return { state: M.moveUp(state) };
    case 'w':
      return { state: M.wordForward(state) };
    case 'b':
      return { state: M.wordBackward(state) };
    case 'e':
      return { state: M.wordEnd(state) };
    case '0':
      return { state: M.lineStart(state) };
    case '$':
      return { state: M.lineEnd(state) };
    case 'G':
      return { state: M.lastLine(state) };
    case 'g':
      return { state: { ...state, pending: 'g' } };
    case 'd':
      return { state: { ...state, pending: 'd' } };
    case 'y':
      return { state: { ...state, pending: 'y' } };
    case 'x':
      return { state: E.deleteChar(state) };
    case 'p':
      return { state: E.pasteBelow(state) };
    case 'P':
      return { state: E.pasteAbove(state) };
    case 't':
      return { state: E.toggleTodo(state) };
    case 'u':
      return { state: undo(state) };
    case 'i':
      return { state: I.enterInsert(state) };
    case 'a':
      return { state: I.enterInsertAfter(state) };
    case 'A':
      return { state: I.enterInsertLineEnd(state) };
    case 'o':
      return { state: I.openBelow(state) };
    case 'O':
      return { state: I.openAbove(state) };
    case ':':
      return { state: { ...state, command: '', message: '' } };
    case 'Enter':
      return { state: M.moveDown(state) };
    case '[':
      return { state, effect: { type: 'prevDay' } };
    case ']':
      return { state, effect: { type: 'nextDay' } };
    case 'Escape':
      return { state: I.enterInsert({ ...state, pending: '', message: '' }) };
  }
  if (key.ctrl && (key.key === 'r' || key.key === 'R')) return { state: redo(state) };
  if (key.ctrl && (key.key === 't' || key.key === 'T')) return { state, effect: { type: 'today' } };
  return { state };
}
