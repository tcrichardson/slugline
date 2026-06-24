import { describe, it, expect } from 'vitest';
import { renderInline } from './renderInline';

describe('renderInline', () => {
  it('escapes HTML', () => {
    expect(renderInline('a < b & c')).toBe('a &lt; b &amp; c');
  });

  it('renders bold and italic', () => {
    expect(renderInline('**bold**')).toBe('<strong>bold</strong>');
    expect(renderInline('*it*')).toBe('<em>it</em>');
    expect(renderInline('_it_')).toBe('<em>it</em>');
  });

  it('renders inline code without processing inner markup', () => {
    expect(renderInline('`a*b*c`')).toBe('<code>a*b*c</code>');
  });

  it('renders safe links', () => {
    expect(renderInline('[rfc](https://example.com)')).toBe(
      '<a href="https://example.com" rel="noopener noreferrer">rfc</a>',
    );
  });

  it('rejects unsafe link schemes (no anchor produced)', () => {
    expect(renderInline('[x](javascript:alert(1))')).not.toContain('<a');
  });
});
