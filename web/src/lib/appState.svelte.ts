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
import { getConfig, listNotes, getNote } from './api';
import type { UiConfig } from './types';

class AppStore {
  tabsState = $state<TabsState>(initTabs(todayISO()));
  noteContent = $state<string>('');
  notesWithFiles = $state<string[]>([]);
  config = $state<UiConfig | null>(null);
  now = $state<Date>(new Date());
  calendar = $state<{ year: number; month: number }>(yearMonth(todayISO()));

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
    await this.refreshNotesList();
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
      this.noteContent = await getNote(date);
      this.calendar = yearMonth(date);
      await this.refreshNotesList(); // a freshly materialized date gets its dot
    } catch (e) {
      console.error(e);
    }
  }

  async goToDate(date: string): Promise<void> {
    this.tabsState = retarget(this.tabsState, date);
    await this.loadActive();
  }

  async openInNewTab(date: string): Promise<void> {
    this.tabsState = openNewTab(this.tabsState, date);
    await this.loadActive();
  }

  async goToday(): Promise<void> {
    await this.goToDate(todayISO());
  }
  async prevDay(): Promise<void> {
    await this.goToDate(addDays(this.activeDate, -1));
  }
  async nextDay(): Promise<void> {
    await this.goToDate(addDays(this.activeDate, 1));
  }

  async switchTab(index: number): Promise<void> {
    this.tabsState = { tabs: this.tabsState.tabs, activeIndex: index };
    await this.loadActive();
  }
  async cycleNext(): Promise<void> {
    this.tabsState = nextTab(this.tabsState);
    await this.loadActive();
  }
  async cyclePrev(): Promise<void> {
    this.tabsState = prevTab(this.tabsState);
    await this.loadActive();
  }
  async closeAt(index: number): Promise<void> {
    this.tabsState = closeTab(this.tabsState, index, todayISO());
    await this.loadActive();
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
