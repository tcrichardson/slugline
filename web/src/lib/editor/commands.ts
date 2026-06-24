import { scanDocument } from '../doc/scan';
import type { Section, Block } from '../doc/types';
import type { EditorState } from './state';
import { pushUndo, clampCursor } from './state';
import { resolveContext, nearestHeadingLevel } from '../doc/context';
import { validateCommand } from '../doc/command';

const TITLES: Record<'todo' | 'meetings' | 'notes', string> = {
  todo: '## To Do',
  meetings: '## Meetings',
  notes: '## Notes',
};
const ORDER: ('todo' | 'meetings' | 'notes')[] = ['todo', 'meetings', 'notes'];

/** Append an H3 (or any heading text) at the end of a section's content. */
export function appendBlock(
  lines: string[],
  section: Section,
  heading: string,
): { lines: string[]; headingIndex: number } {
  const idx = section.endLine + 1;
  const out = lines.slice();
  out.splice(idx, 0, heading, '');
  return { lines: out, headingIndex: idx };
}

/** Append a single line after the last non-blank line of a section (or right after its heading). */
export function appendLineToSection(
  lines: string[],
  section: Section,
  text: string,
): { lines: string[]; index: number } {
  let insertAt = section.startLine + 1;
  for (let i = section.startLine + 1; i <= section.endLine; i++) {
    if ((lines[i] ?? '').trim() !== '') insertAt = i + 1;
  }
  const out = lines.slice();
  out.splice(insertAt, 0, text);
  return { lines: out, index: insertAt };
}

/** Insert or update a `meta:key value` line within a block's meta region. */
export function upsertMeta(
  lines: string[],
  block: Block,
  key: string,
  value: string,
): { lines: string[]; index: number } {
  const metaLine = `meta:${key} ${value}`;
  const existing = block.meta.find((m) => m.key === key);
  const out = lines.slice();
  if (existing) {
    out[existing.lineIndex] = metaLine;
    return { lines: out, index: existing.lineIndex };
  }
  const insertAt = block.metaEndLine + 1; // metaEndLine === heading when no meta yet
  out.splice(insertAt, 0, metaLine);
  return { lines: out, index: insertAt };
}

/** Ensure a standard section exists; if missing, insert it in canonical order. */
export function ensureSection(
  lines: string[],
  kind: 'todo' | 'meetings' | 'notes',
): { lines: string[]; section: Section } {
  let model = scanDocument(lines);
  const found = model.sections.find((s) => s.kind === kind);
  if (found) return { lines, section: found };

  const orderIdx = ORDER.indexOf(kind);
  let insertAt = lines.length;
  let placed = false;
  for (let i = orderIdx - 1; i >= 0 && !placed; i--) {
    const prev = model.sections.find((s) => s.kind === ORDER[i]);
    if (prev) {
      insertAt = prev.endLine + 1;
      placed = true;
    }
  }
  if (!placed) insertAt = model.titleLineIndex !== null ? model.titleLineIndex + 1 : 0;

  const out = lines.slice();
  out.splice(insertAt, 0, '', TITLES[kind], '');
  model = scanDocument(out);
  return { lines: out, section: model.sections.find((s) => s.kind === kind)! };
}

/** Index just past the enclosing heading's content (before the next same/shallower heading, or EOF). */
export function endOfEnclosingSection(lines: string[], cursorLine: number, level: number): number {
  const H = /^(#{1,6})\s/;
  let start = cursorLine;
  while (start >= 0) {
    const m = H.exec(lines[start] ?? '');
    if (m && m[1].length === level) break;
    start--;
  }
  if (start < 0) start = cursorLine;
  for (let i = start + 1; i < lines.length; i++) {
    const m = H.exec(lines[i] ?? '');
    if (m && m[1].length <= level) return i;
  }
  return lines.length;
}

export type AppEffect =
  | { type: 'goto'; date: string }
  | { type: 'today' }
  | { type: 'tab'; date: string }
  | { type: 'close' }
  | { type: 'save' }
  | { type: 'theme'; theme: string }
  | { type: 'prevDay' }
  | { type: 'nextDay' }
  | { type: 'tabNext' }
  | { type: 'tabPrev' };

export interface CommandCtx {
  nowHHMM: string;
}
export interface CommandResult {
  state: EditorState;
  effect?: AppEffect;
}

function addBlock(state: EditorState, kind: 'meetings' | 'notes', name: string): EditorState {
  const ns = pushUndo(state);
  const ensured = ensureSection(ns.lines, kind);
  const { lines, headingIndex } = appendBlock(ensured.lines, ensured.section, `### ${name}`);
  return clampCursor({ ...ns, lines, cursor: { line: headingIndex, col: 0 }, message: '' });
}

function addTodo(state: EditorState, text: string): EditorState {
  const ns = pushUndo(state);
  const ctx = resolveContext(scanDocument(ns.lines), ns.cursor.line);
  const suffix = ctx.kind === 'meeting' ? ` _(${ctx.block.name})_` : '';
  const ensured = ensureSection(ns.lines, 'todo');
  const { lines } = appendLineToSection(ensured.lines, ensured.section, `- [ ] ${text}${suffix}`);
  return { ...ns, lines, message: '' }; // cursor stays
}

function addSubsection(state: EditorState, name: string): EditorState {
  const level = nearestHeadingLevel(state.lines, state.cursor.line);
  if (level === null) return { ...state, message: 'No enclosing heading' };
  if (level >= 6) return { ...state, message: 'Max heading depth' };
  const ns = pushUndo(state);
  const heading = `${'#'.repeat(level + 1)} ${name}`;
  const insertAt = endOfEnclosingSection(ns.lines, ns.cursor.line, level);
  const lines = ns.lines.slice();
  lines.splice(insertAt, 0, heading, '');
  return clampCursor({ ...ns, lines, cursor: { line: insertAt, col: 0 }, message: '' });
}

function setMeta(
  state: EditorState,
  required: 'meeting' | 'note',
  key: string,
  value: string,
): EditorState {
  const ctx = resolveContext(scanDocument(state.lines), state.cursor.line);
  if (ctx.kind !== required) {
    return { ...state, message: required === 'meeting' ? 'Not in a meeting' : 'Not in a note' };
  }
  const ns = pushUndo(state);
  const { lines } = upsertMeta(ns.lines, ctx.block, key, value);
  return { ...ns, lines, message: '' };
}

export function runCommand(state: EditorState, ctx: CommandCtx): CommandResult {
  const base: EditorState = { ...state, command: null };
  const v = validateCommand(state.command ?? '');
  if (!v.ok) return { state: { ...base, message: v.error } };
  const { command, arg } = v;
  switch (command) {
    case 'goto':
      return { state: { ...base, message: '' }, effect: { type: 'goto', date: arg } };
    case 'today':
      return { state: { ...base, message: '' }, effect: { type: 'today' } };
    case 'tab':
      return { state: { ...base, message: '' }, effect: { type: 'tab', date: arg } };
    case 'close':
      return { state: { ...base, message: '' }, effect: { type: 'close' } };
    case 'w':
      return { state: { ...base, message: 'Written' }, effect: { type: 'save' } };
    case 'theme':
      return { state: { ...base, message: '' }, effect: { type: 'theme', theme: arg } };
    case 'meeting':
      return { state: addBlock(base, 'meetings', arg) };
    case 'note':
      return { state: addBlock(base, 'notes', arg) };
    case 'todo':
      return { state: addTodo(base, arg) };
    case 'section':
      return { state: addSubsection(base, arg) };
    case 'scheduled':
      return { state: setMeta(base, 'meeting', 'scheduled', arg) };
    case 'purpose':
      return { state: setMeta(base, 'meeting', 'purpose', arg) };
    case 'start':
      return { state: setMeta(base, 'meeting', 'started', ctx.nowHHMM) };
    case 'end':
      return { state: setMeta(base, 'meeting', 'ended', ctx.nowHHMM) };
    case 'topic':
      return { state: setMeta(base, 'note', 'topic', arg) };
  }
}
