const SAFE_SCHEME = /^(https?:|mailto:)/i;

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export function renderInline(text: string): string {
  // 1. Pull out code spans first so their contents are not further processed.
  const codes: string[] = [];
  let s = text.replace(/`([^`]+)`/g, (_m, code: string) => {
    const i = codes.push(`<code>${escapeHtml(code)}</code>`) - 1;
    return `\u0000${i}\u0000`;
  });

  // 2. Escape everything else.
  s = escapeHtml(s);

  // 3. Links: [label](url). label/url are already escaped.
  s = s.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (m, label: string, url: string) => {
    const raw = url.replace(/&amp;/g, '&');
    if (!SAFE_SCHEME.test(raw)) return m;
    return `<a href="${url}" rel="noopener noreferrer">${label}</a>`;
  });

  // 4. Bold before italic.
  s = s.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  s = s.replace(/\*([^*]+)\*/g, '<em>$1</em>');
  s = s.replace(/_([^_]+)_/g, '<em>$1</em>');

  // 5a. Strikethrough.
  s = s.replace(/~~([^~]+)~~/g, '<del>$1</del>');

  // 5b. Highlight.
  s = s.replace(/==([^=]+)==/g, '<mark>$1</mark>');

  // 6. Restore code spans.
  s = s.replace(/\u0000(\d+)\u0000/g, (_m, i: string) => codes[Number(i)]);
  return s;
}
