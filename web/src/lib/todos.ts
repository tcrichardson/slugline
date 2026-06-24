import { scanDocument } from './doc/scan';
import { classifyLine } from './doc/classify';
import { addDays } from './dates';

export interface TodoItem {
  text: string;
  done: boolean;
  lineIndex: number;
}

export interface TodoGroup {
  date: string;
  todos: TodoItem[];
}

/** Task items in the `## To Do` section (both states), skipping blanks. */
export function extractTodos(lines: string[]): TodoItem[] {
  const section = scanDocument(lines).sections.find((s) => s.kind === 'todo');
  if (!section) return [];
  const out: TodoItem[] = [];
  for (let i = section.startLine + 1; i <= section.endLine; i++) {
    const c = classifyLine(lines[i] ?? '');
    if (c.kind === 'task' && c.text.trim() !== '') {
      out.push({ text: c.text, done: !!c.done, lineIndex: i });
    }
  }
  return out;
}

/** The `days` dates ending on `activeDate` (inclusive), most-recent first. */
export function windowDates(activeDate: string, days = 7): string[] {
  const out: string[] = [];
  for (let i = 0; i < days; i++) out.push(addDays(activeDate, -i));
  return out;
}
