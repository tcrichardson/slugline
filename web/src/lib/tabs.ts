export interface TabsState {
  tabs: string[]; // date strings, de-duplicated
  activeIndex: number;
}

export function initTabs(today: string): TabsState {
  return { tabs: [today], activeIndex: 0 };
}

export function activeDate(state: TabsState): string {
  return state.tabs[state.activeIndex];
}

/** Retarget the active tab to `date` in place; if `date` is already open, focus that tab. */
export function retarget(state: TabsState, date: string): TabsState {
  const existing = state.tabs.indexOf(date);
  if (existing !== -1) {
    return { tabs: state.tabs, activeIndex: existing };
  }
  const tabs = state.tabs.slice();
  tabs[state.activeIndex] = date;
  return { tabs, activeIndex: state.activeIndex };
}

/** Open `date` in a new tab (appended right) and focus it; focus an existing tab if present. */
export function openNewTab(state: TabsState, date: string): TabsState {
  const existing = state.tabs.indexOf(date);
  if (existing !== -1) {
    return { tabs: state.tabs, activeIndex: existing };
  }
  const tabs = [...state.tabs, date];
  return { tabs, activeIndex: tabs.length - 1 };
}

/** Close the tab at `index`; guarantees >=1 tab by falling back to `today`. */
export function closeTab(state: TabsState, index: number, today: string): TabsState {
  if (index < 0 || index >= state.tabs.length) return state;
  const tabs = state.tabs.slice();
  tabs.splice(index, 1);
  if (tabs.length === 0) {
    return { tabs: [today], activeIndex: 0 };
  }
  let activeIndex = state.activeIndex;
  if (index < activeIndex) activeIndex -= 1;
  else if (index === activeIndex) activeIndex = Math.min(activeIndex, tabs.length - 1);
  return { tabs, activeIndex };
}

export function nextTab(state: TabsState): TabsState {
  return { tabs: state.tabs, activeIndex: (state.activeIndex + 1) % state.tabs.length };
}

export function prevTab(state: TabsState): TabsState {
  const n = state.tabs.length;
  return { tabs: state.tabs, activeIndex: (state.activeIndex - 1 + n) % n };
}
