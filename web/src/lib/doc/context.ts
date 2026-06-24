import type { DocModel, Block, Section } from './types';

export type Context =
  | { kind: 'none' }
  | { kind: 'todo'; section: Section }
  | { kind: 'meeting'; block: Block; section: Section }
  | { kind: 'note'; block: Block; section: Section }
  | { kind: 'other'; section: Section };

export function resolveContext(model: DocModel, lineIndex: number): Context {
  const section = model.sections.find((s) => lineIndex >= s.startLine && lineIndex <= s.endLine);
  if (!section) return { kind: 'none' };

  if (section.kind === 'todo') return { kind: 'todo', section };

  if (section.kind === 'meetings' || section.kind === 'notes') {
    const block = section.blocks.find((b) => lineIndex >= b.startLine && lineIndex <= b.endLine);
    if (block) {
      return section.kind === 'meetings'
        ? { kind: 'meeting', block, section }
        : { kind: 'note', block, section };
    }
    return { kind: 'other', section };
  }

  return { kind: 'other', section };
}

const HEADING = /^(#{1,6})\s+/;

export function nearestHeadingLevel(lines: string[], lineIndex: number): number | null {
  for (let i = Math.min(lineIndex, lines.length - 1); i >= 0; i--) {
    const m = HEADING.exec(lines[i]);
    if (m) return m[1].length;
  }
  return null;
}
