import { describe, it, expect } from 'vitest';
import { classifyLine } from './classify';

describe('classifyLine', () => {
  it('classifies blank lines', () => {
    expect(classifyLine('').kind).toBe('blank');
    expect(classifyLine('   ').kind).toBe('blank');
  });

  it('classifies headings with level and text', () => {
    const h = classifyLine('### Weekly Sync');
    expect(h.kind).toBe('heading');
    expect(h.level).toBe(3);
    expect(h.text).toBe('Weekly Sync');
  });

  it('classifies tasks with done state', () => {
    const open = classifyLine('- [ ] Buy milk');
    expect(open.kind).toBe('task');
    expect(open.done).toBe(false);
    expect(open.text).toBe('Buy milk');

    const done = classifyLine('- [X] Send invoice');
    expect(done.kind).toBe('task');
    expect(done.done).toBe(true);
    expect(done.text).toBe('Send invoice');
  });

  it('classifies meta lines with key and value', () => {
    const m = classifyLine('meta:scheduled 14:30');
    expect(m.kind).toBe('meta');
    expect(m.metaKey).toBe('scheduled');
    expect(m.text).toBe('14:30');
  });

  it('classifies meta lines with empty value', () => {
    const m = classifyLine('meta:purpose');
    expect(m.kind).toBe('meta');
    expect(m.metaKey).toBe('purpose');
    expect(m.text).toBe('');
  });

  it('classifies plain list items', () => {
    const l = classifyLine('- a bullet');
    expect(l.kind).toBe('list');
    expect(l.text).toBe('a bullet');
  });

  it('falls back to paragraph', () => {
    const p = classifyLine('just some prose');
    expect(p.kind).toBe('paragraph');
    expect(p.text).toBe('just some prose');
  });
});
