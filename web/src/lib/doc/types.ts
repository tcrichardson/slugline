export type LineKind = 'heading' | 'task' | 'list' | 'meta' | 'blank' | 'paragraph';

export interface ClassifiedLine {
  kind: LineKind;
  /** The original line, unmodified. */
  raw: string;
  /** Content with any prefix stripped. For `meta` this is the value; for `blank` it is ''. */
  text: string;
  /** Heading level 1–6 when kind === 'heading'. */
  level?: number;
  /** Done state when kind === 'task'. */
  done?: boolean;
  /** Key when kind === 'meta'. */
  metaKey?: string;
}

export interface MetaEntry {
  key: string;
  value: string;
  lineIndex: number;
}

export type SectionKind = 'todo' | 'meetings' | 'notes' | 'other';

export interface Block {
  name: string;
  level: number; // 3
  headingLineIndex: number;
  startLine: number; // inclusive
  endLine: number; // inclusive
  meta: MetaEntry[];
  /** Index of the last meta line, or headingLineIndex when the block has no meta. */
  metaEndLine: number;
}

export interface Section {
  kind: SectionKind;
  title: string;
  level: number; // 2
  headingLineIndex: number;
  startLine: number; // inclusive (the heading line)
  endLine: number; // inclusive (last line before next H2/H1 or EOF)
  blocks: Block[]; // H3 blocks for meetings/notes; empty otherwise
}

export interface DocModel {
  title: string | null;
  titleLineIndex: number | null;
  sections: Section[];
}
