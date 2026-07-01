import { classifyLine } from './classify';
import type { ClassifiedLine, DocModel, Section, Block, SectionKind, MetaEntry } from './types';

function sectionKind(title: string): SectionKind {
  const t = title.trim().toLowerCase();
  if (t === 'to do' || t === 'todo') return 'todo';
  if (t === 'meetings') return 'meetings';
  if (t === 'notes') return 'notes';
  return 'other';
}

/**
 * Scans forward from `from` to `to` (inclusive) and returns the index just
 * before the first heading whose level is `<= maxLevel`, or `to` if none is
 * found. Used to find where an H3 block or H2 section ends.
 */
function findBoundaryEnd(classified: ClassifiedLine[], from: number, to: number, maxLevel: number): number {
  for (let j = from; j <= to; j++) {
    const c = classified[j];
    if (c.kind === 'heading' && c.level! <= maxLevel) return j - 1;
  }
  return to;
}

function collectBlocks(classified: ClassifiedLine[], from: number, to: number): Block[] {
  const blocks: Block[] = [];
  for (let i = from; i <= to; i++) {
    const c = classified[i];
    if (c.kind === 'heading' && c.level === 3) {
      const start = i;
      const end = findBoundaryEnd(classified, i + 1, to, 3);

      const meta: MetaEntry[] = [];
      let metaEndLine = start;
      for (let k = start + 1; k <= end && classified[k].kind === 'meta'; k++) {
        meta.push({ key: classified[k].metaKey!, value: classified[k].text, lineIndex: k });
        metaEndLine = k;
      }

      blocks.push({
        name: c.text,
        level: 3,
        headingLineIndex: start,
        startLine: start,
        endLine: end,
        meta,
        metaEndLine,
      });
      i = end;
    }
  }
  return blocks;
}

export function scanDocument(lines: string[]): DocModel {
  const classified = lines.map(classifyLine);

  let title: string | null = null;
  let titleLineIndex: number | null = null;
  for (let i = 0; i < classified.length; i++) {
    const c = classified[i];
    if (c.kind === 'heading' && c.level === 1) {
      title = c.text;
      titleLineIndex = i;
      break;
    }
  }

  const sections: Section[] = [];
  for (let i = 0; i < classified.length; i++) {
    const c = classified[i];
    if (c.kind === 'heading' && c.level === 2) {
      const start = i;
      const end = findBoundaryEnd(classified, i + 1, classified.length - 1, 2);

      const kind = sectionKind(c.text);
      const section: Section = {
        kind,
        title: c.text,
        level: 2,
        headingLineIndex: start,
        startLine: start,
        endLine: end,
        blocks: kind === 'meetings' || kind === 'notes' ? collectBlocks(classified, start + 1, end) : [],
      };
      sections.push(section);
    }
  }

  return { title, titleLineIndex, sections };
}
