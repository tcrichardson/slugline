import {
  initTabs,
  retarget,
  openNewTab,
  closeTab,
  nextTab,
  prevTab,
  activeDate,
  type TabsState,
} from './tabs';
import { todayISO, addDays, yearMonth } from './dates';
import { applyTheme } from './theme';
import { getConfig, listNotes, getNote, putNote } from './api';
import type { UiConfig } from './types';
import { createEditorState, clampCursor, type EditorState } from './editor/state';
import { handleKey, type KeyInput } from './editor/keymap';
import type { AppEffect } from './editor/commands';
import { extractTodos, windowDates, type TodoGroup } from './todos';

function nowHHMM(d: Date): string {
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
}

class AppStore {
  tabsState = $state<TabsState>(initTabs(todayISO()));
  editor = $state<EditorState>(createEditorState(['']));
  notesWithFiles = $state<string[]>([]);
  config = $state<UiConfig | null>(null);
  now = $state<Date>(new Date());
  calendar = $state<{ year: number; month: number }>(yearMonth(todayISO()));
  todoGroups = $state<TodoGroup[]>([]);

  private sharedRegister: string[] = [];
  private lastSaved = '';
  private saveTimer: ReturnType<typeof setTimeout> | null = null;

  get activeDate(): string {
    return activeDate(this.tabsState);
  }

  async init(): Promise<void> {
    try {
      this.config = await getConfig();
      applyTheme(this.config.theme, this.config.font, this.config.colors);
    } catch (e) {
      console.error(e);
    }
    await this.loadActive();
    setInterval(() => {
      this.now = new Date();
    }, 30_000);
  }

  async refreshNotesList(): Promise<void> {
    try {
      this.notesWithFiles = await listNotes();
    } catch (e) {
      console.error(e);
    }
  }

  async loadActive(): Promise<void> {
    const date = this.activeDate;
    try {
      const content = await getNote(date);
      this.lastSaved = content;
      this.editor = createEditorState(content.split('\n'), this.sharedRegister);
      this.calendar = yearMonth(date);
      await this.refreshNotesList();
      await this.refreshTodos();
    } catch (e) {
      console.error(e);
    }
  }

  // ---- keyboard ----
  onKey(input: KeyInput): void {
    const ctx = { nowHHMM: nowHHMM(this.now) };
    const before = this.editor.lines;
    const { state, effect } = handleKey(this.editor, input, ctx);
    this.editor = state;
    this.sharedRegister = state.register;
    if (state.lines !== before) this.scheduleSave();
    if (effect) void this.runEffect(effect);
  }

  private content(): string {
    const body = this.editor.lines.join('\n');
    return body.endsWith('\n') ? body : body + '\n';
  }

  private normalized(s: string): string {
    return s.endsWith('\n') ? s : s + '\n';
  }

  private scheduleSave(): void {
    if (this.saveTimer) clearTimeout(this.saveTimer);
    this.saveTimer = setTimeout(() => void this.flush(), 750);
  }

  async flush(): Promise<void> {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
      this.saveTimer = null;
    }
    const content = this.content();
    if (content === this.normalized(this.lastSaved)) return;
    try {
      await putNote(this.activeDate, content);
      this.lastSaved = content;
      await this.refreshTodos();
    } catch (e) {
      this.editor = { ...this.editor, message: 'Save failed' };
      console.error(e);
    }
  }

  private async runEffect(effect: AppEffect): Promise<void> {
    switch (effect.type) {
      case 'goto':
        return this.goToDate(effect.date);
      case 'today':
        return this.goToDate(todayISO());
      case 'tab':
        return this.openInNewTab(effect.date);
      case 'close':
        return this.closeActive();
      case 'save':
        return this.flush();
      case 'prevDay':
        return this.goToDate(addDays(this.activeDate, -1));
      case 'nextDay':
        return this.goToDate(addDays(this.activeDate, 1));
      case 'tabNext':
        await this.flush();
        this.tabsState = nextTab(this.tabsState);
        return this.loadActive();
      case 'tabPrev':
        await this.flush();
        this.tabsState = prevTab(this.tabsState);
        return this.loadActive();
      case 'theme':
        if (this.config) {
          this.config = { ...this.config, theme: effect.theme };
          applyTheme(effect.theme, this.config.font, this.config.colors);
        }
        return;
    }
  }

  // ---- navigation (flush the current buffer first) ----
  async goToDate(date: string): Promise<void> {
    await this.flush();
    this.tabsState = retarget(this.tabsState, date);
    await this.loadActive();
  }
  async openInNewTab(date: string): Promise<void> {
    await this.flush();
    this.tabsState = openNewTab(this.tabsState, date);
    await this.loadActive();
  }
  async switchTab(index: number): Promise<void> {
    await this.flush();
    this.tabsState = { tabs: this.tabsState.tabs, activeIndex: index };
    await this.loadActive();
  }
  async closeActive(): Promise<void> {
    await this.flush();
    this.tabsState = closeTab(this.tabsState, this.tabsState.activeIndex, todayISO());
    await this.loadActive();
  }
  async closeAt(index: number): Promise<void> {
    await this.flush();
    this.tabsState = closeTab(this.tabsState, index, todayISO());
    await this.loadActive();
  }

  async refreshTodos(): Promise<void> {
    const active = this.activeDate;
    const existing = new Set(this.notesWithFiles);
    const groups: TodoGroup[] = [];
    for (const date of windowDates(active)) {
      if (date !== active && !existing.has(date)) continue; // never materialize other days
      try {
        const content = await getNote(date);
        const todos = extractTodos(content.split('\n'));
        if (todos.length > 0) groups.push({ date, todos });
      } catch (e) {
        console.error(e);
      }
    }
    this.todoGroups = groups;
  }

  jumpToLine(line: number): void {
    this.editor = clampCursor({ ...this.editor, cursor: { line, col: 0 } });
  }

  async goToDateAndLine(date: string, line: number): Promise<void> {
    if (date === this.activeDate) {
      this.jumpToLine(line);
      return;
    }
    await this.goToDate(date);
    this.jumpToLine(line);
  }

  prevMonth(): void {
    let { year, month } = this.calendar;
    month -= 1;
    if (month < 1) {
      month = 12;
      year -= 1;
    }
    this.calendar = { year, month };
  }
  nextMonth(): void {
    let { year, month } = this.calendar;
    month += 1;
    if (month > 12) {
      month = 1;
      year += 1;
    }
    this.calendar = { year, month };
  }
}

export const app = new AppStore();
