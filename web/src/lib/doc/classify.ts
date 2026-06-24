import type { ClassifiedLine } from './types';

const HEADING = /^(#{1,6})\s+(.*)$/;
const TASK = /^- \[([ xX])\]\s?(.*)$/;
const META = /^meta:(\S+)(?: (.*))?$/;
const UL = /^\s*[-*+]\s+(.*)$/;
const OL = /^\s*\d+\.\s+(.*)$/;

export function classifyLine(raw: string): ClassifiedLine {
  if (raw.trim() === '') return { kind: 'blank', raw, text: '' };

  const h = HEADING.exec(raw);
  if (h) return { kind: 'heading', raw, level: h[1].length, text: h[2].trim() };

  const t = TASK.exec(raw);
  if (t) return { kind: 'task', raw, done: t[1].toLowerCase() === 'x', text: t[2] };

  const m = META.exec(raw);
  if (m) return { kind: 'meta', raw, metaKey: m[1], text: (m[2] ?? '').trim() };

  const ul = UL.exec(raw);
  if (ul) return { kind: 'list', raw, text: ul[1] };
  const ol = OL.exec(raw);
  if (ol) return { kind: 'list', raw, text: ol[1] };

  return { kind: 'paragraph', raw, text: raw };
}
