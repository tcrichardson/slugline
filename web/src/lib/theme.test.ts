import { describe, it, expect } from 'vitest';
import { resolveTokens, LIGHT, DARK, nextTheme } from './theme';

describe('theme', () => {
  it('returns built-in light tokens by default', () => {
    expect(resolveTokens('light')['--bg']).toBe(LIGHT['--bg']);
  });

  it('returns dark tokens for the dark theme', () => {
    expect(resolveTokens('dark')['--bg']).toBe(DARK['--bg']);
  });

  it('falls back to light for unknown themes', () => {
    expect(resolveTokens('neon')['--bg']).toBe(LIGHT['--bg']);
  });

  it('applies per-theme config overrides over the base', () => {
    const t = resolveTokens('dark', { dark: { '--bg': '#000000' } });
    expect(t['--bg']).toBe('#000000');
    expect(t['--fg']).toBe(DARK['--fg']);
  });
});

describe('nextTheme', () => {
  it('flips dark to light and anything else to dark', () => {
    expect(nextTheme('dark')).toBe('light');
    expect(nextTheme('light')).toBe('dark');
    expect(nextTheme('whatever')).toBe('dark');
  });
});
