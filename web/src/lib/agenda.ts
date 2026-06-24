import { scanDocument } from './doc/scan';

export interface AgendaItem {
  time: string;
  name: string;
  headingLineIndex: number;
  started?: string;
  ended?: string;
}

/** Scheduled meetings for a note, sorted ascending by HH:MM. Meetings without a scheduled time are omitted. */
export function deriveAgenda(lines: string[]): AgendaItem[] {
  const meetings = scanDocument(lines).sections.find((s) => s.kind === 'meetings');
  if (!meetings) return [];

  const items: AgendaItem[] = [];
  for (const block of meetings.blocks) {
    const scheduled = block.meta.find((m) => m.key === 'scheduled');
    if (!scheduled || scheduled.value.trim() === '') continue;
    items.push({
      time: scheduled.value.trim(),
      name: block.name,
      headingLineIndex: block.headingLineIndex,
      started: block.meta.find((m) => m.key === 'started')?.value.trim(),
      ended: block.meta.find((m) => m.key === 'ended')?.value.trim(),
    });
  }
  items.sort((a, b) => a.time.localeCompare(b.time));
  return items;
}
