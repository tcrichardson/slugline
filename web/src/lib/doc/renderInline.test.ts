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

  it('renders strikethrough', () => {
    expect(renderInline('~~deleted~~')).toBe('<del>deleted</del>');
  });

  it('renders highlight', () => {
    expect(renderInline('==marked==')).toBe('<mark>marked</mark>');
  });

  it('renders strikethrough alongside bold without conflict', () => {
    expect(renderInline('**bold** and ~~strike~~')).toBe(
      '<strong>bold</strong> and <del>strike</del>',
    );
  });

  it('does not process strikethrough or highlight inside code spans', () => {
    expect(renderInline('`~~raw~~`')).toBe('<code>~~raw~~</code>');
    expect(renderInline('`==raw==`')).toBe('<code>==raw==</code>');
  });
});
