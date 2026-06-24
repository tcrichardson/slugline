import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import * as api from './api';

describe('api client', () => {
  const fetchMock = vi.fn();

  beforeEach(() => {
    vi.stubGlobal('fetch', fetchMock);
    fetchMock.mockReset();
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('getNote returns text from the right URL', async () => {
    fetchMock.mockResolvedValue({ ok: true, text: async () => '# hi\n' });
    expect(await api.getNote('2026-06-23')).toBe('# hi\n');
    expect(fetchMock).toHaveBeenCalledWith('/api/notes/2026-06-23');
  });

  it('listNotes parses a JSON array', async () => {
    fetchMock.mockResolvedValue({ ok: true, json: async () => ['2026-06-23'] });
    expect(await api.listNotes()).toEqual(['2026-06-23']);
  });

  it('putNote sends a PUT with the body', async () => {
    fetchMock.mockResolvedValue({ ok: true });
    await api.putNote('2026-06-23', '# hi\n');
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/notes/2026-06-23',
      expect.objectContaining({ method: 'PUT', body: '# hi\n' }),
    );
  });

  it('throws on a non-ok response', async () => {
    fetchMock.mockResolvedValue({ ok: false, status: 500 });
    await expect(api.getConfig()).rejects.toThrow();
  });
});
