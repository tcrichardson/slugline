export type Tokens = Record<string, string>;

export const LIGHT: Tokens = {
  '--bg': '#fbfcfe',
  '--fg': '#1b2330',
  '--muted': '#5b6675',
  '--accent': '#2f6df6',
  '--heading-1': '#1d4ed8',
  '--heading-2': '#2563eb',
  '--heading-3': '#3b82f6',
  '--heading-4': '#60a5fa',
  '--heading-5': '#7dabfb',
  '--heading-6': '#9cc2fc',
  '--todo-done': '#8a93a3',
  '--meta': '#6b7686',
  '--status-bar': '#eef2f9',
  '--edit-line-bg': '#eaf1ff',
  '--edit-bar-bg': '#e2ebff',
  '--rule': '#d9e0ec',
  '--cursor': '#1b2330',
  '--blockquote-border': '#93c5fd',
  '--highlight-bg': '#fef08a',
};

export const DARK: Tokens = {
  '--bg': '#161a26',
  '--fg': '#e7ecf5',
  '--muted': '#97a1b3',
  '--accent': '#6f9bff',
  '--heading-1': '#FFBA00',
  '--heading-2': '#FFBA00',
  '--heading-3': '#60a5fa',
  '--heading-4': '#3b82f6',
  '--heading-5': '#2563eb',
  '--heading-6': '#1d4ed8',
  '--todo-done': '#6b7686',
  '--meta': '#97a1b3',
  '--status-bar': '#1f2535',
  '--edit-line-bg': '#222a3d',
  '--edit-bar-bg': '#2a344c',
  '--rule': '#2d3650',
  '--cursor': '#e7ecf5',
  '--blockquote-border': '#3b82f6',
  '--highlight-bg': '#713f12',
};

export function builtinTokens(theme: string): Tokens {
  return theme === 'dark' ? { ...DARK } : { ...LIGHT };
}

/** The opposite of the given theme (anything not 'dark' flips to 'dark'). */
export function nextTheme(theme: string): string {
  return theme === 'dark' ? 'light' : 'dark';
}

/** Merge built-in tokens with per-theme overrides from config. */
export function resolveTokens(
  theme: string,
  overrides: Record<string, Record<string, string>> = {},
): Tokens {
  return { ...builtinTokens(theme), ...(overrides[theme] ?? {}) };
}

/** Apply tokens + font to the document root. DOM side-effect; not unit-tested. */
export function applyTheme(
  theme: string,
  font: string,
  overrides: Record<string, Record<string, string>> = {},
): void {
  const tokens = resolveTokens(theme, overrides);
  const root = document.documentElement;
  for (const [k, v] of Object.entries(tokens)) root.style.setProperty(k, v);
  root.style.setProperty('--font', font);
  root.dataset.theme = theme;
}
