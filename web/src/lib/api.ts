import type { UiConfig } from './types';

export async function listNotes(): Promise<string[]> {
  const res = await fetch('/api/notes');
  if (!res.ok) throw new Error(`listNotes failed: ${res.status}`);
  return res.json();
}

export async function getNote(date: string): Promise<string> {
  const res = await fetch(`/api/notes/${date}`);
  if (!res.ok) throw new Error(`getNote failed: ${res.status}`);
  return res.text();
}

export async function putNote(date: string, content: string): Promise<void> {
  const res = await fetch(`/api/notes/${date}`, {
    method: 'PUT',
    headers: { 'content-type': 'text/markdown' },
    body: content,
  });
  if (!res.ok) throw new Error(`putNote failed: ${res.status}`);
}

export async function getConfig(): Promise<UiConfig> {
  const res = await fetch('/api/config');
  if (!res.ok) throw new Error(`getConfig failed: ${res.status}`);
  return res.json();
}

export async function putTheme(theme: string): Promise<void> {
  const res = await fetch('/api/config/theme', {
    method: 'PUT',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ theme }),
  });
  if (!res.ok) throw new Error(`putTheme failed: ${res.status}`);
}
