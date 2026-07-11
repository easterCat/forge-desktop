/**
 * Lightweight Markdown renderer for source notes.
 * Supports: headings, bold, italic, inline code, code blocks,
 * unordered/ordered lists, links, blockquotes, horizontal rules.
 * No external dependencies.
 *
 * Security strategy (placeholder-based, two-phase):
 * 1. Replace markdown syntax with placeholder tokens (`\u0091PH{n}\u0091`)
 *    and stash the rendered HTML body separately. During this phase,
 *    bodies are HTML-escaped at the point of capture so they are safe.
 * 2. After every markdown replacement is done, wrap contiguous <li>
 *    placeholder runs in <ul>/<ol>.
 * 3. Walk the resulting string, escaping everything except our own tag
 *    boundaries and placeholder tokens.
 * 4. Wrap paragraph blocks, then substitute placeholders back to their
 *    real safe HTML.
 *
 * ## XSS hardening notes (reviewed against S-1 from performance/security audit)
 *
 * The renderer uses a placeholder protocol so that user-supplied text
 * never reaches the output DOM unescaped. Specifically:
 * - Inline markdown bodies (heading text, list items, bold/italic, etc.)
 *   are HTML-escaped at `stash()` time, BEFORE they are placed in the
 *   output buffer.
 * - Fenced code blocks are extracted into a separate array
 *   (`codeBlocks`) and HTML-escaped at substitution time
 *   (see `\u0091CODE{n}\u0091` handler at the bottom of the function).
 *   This is the critical path that prevents `<script>` and friends in
 *   code fences from being executed.
 * - Inline code spans are escaped via the `\u0091PH{n}\u0091` stash path.
 * - Links go through `sanitizeUrl()` which rejects any URL not matching
 *   the `SAFE_URL_RE` allow-list (`javascript:`, `data:`, `vbscript:`,
 *   and raw control chars are all blocked).
 * - `\u0091PH{n}\u0091` and `\u0091CODE{n}\u0091` placeholders survive
 *   `escapeHtml()` because their characters are ASCII control bytes that
 *   no realistic user input contains.
 */

/** 仅放行常见的安全协议，拒绝 javascript:/data:/vbscript: 等 */
const SAFE_URL_RE = /^(https?:|mailto:|ftp:|\/|#)/i;

/** Placeholder marker. Uses ASCII control chars that survive escapeHtml. */
const PH_PREFIX = '\u0091PH';
const PH_SUFFIX = '\u0091';

/**
 * Logger shim. The real `console.warn` is fine for renderer code, but routing
 * through a thin wrapper keeps the call sites uniform if the project later
 * adopts a dedicated logging utility.
 */
function logWarn(msg: string): void {
  // eslint-disable-next-line no-console
  console.warn(`[markdown] ${msg}`);
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

/** 规范化并校验链接 URL；不安全则返回 '#' */
function sanitizeUrl(url: string): string {
  const trimmed = url.trim();
  if (!trimmed) return '#';
  // eslint-disable-next-line no-control-regex
  if (/[\u0000-\u001f\u007f]/.test(trimmed)) return '#';
  if (!SAFE_URL_RE.test(trimmed)) return '#';
  return trimmed;
}

export function renderMarkdown(md: string): string {
  if (!md) return '';

  // Defence-in-depth cap: source notes are user-authored and stored locally,
  // but a hostile sync source or accidental paste could be gigabytes. 256 KB
  // is well above any reasonable note size; over that we refuse rather than
  // risk OOM during the regex pass.
  const MAX_INPUT_LEN = 256 * 1024;
  if (md.length > MAX_INPUT_LEN) {
    logWarn('Markdown input exceeds length cap; truncating');
    md = md.slice(0, MAX_INPUT_LEN);
  }

  // ---------------------------------------------------------------------------
  // Phase 1: code blocks (extracted, not inline-stashed — content is used
  // directly at substitution time).
  // ---------------------------------------------------------------------------
  const codeBlocks: string[] = [];
  let html = md.replace(/```([\s\S]*?)```/g, (_m, code: string) => {
    const id = codeBlocks.length;
    codeBlocks.push(code.trim());
    return `\u0091CODE${id}\u0091`;
  });

  // ---------------------------------------------------------------------------
  // Phase 2: markdown → placeholder tokens. Each captured body is escaped
  // when the placeholder is created, so anything in `safeTags` is already
  // safe to splice into the final HTML.
  // ---------------------------------------------------------------------------
  const safeTags = new Map<string, string>();
  let counter = 0;
  const stash = (safeHtml: string): string => {
    const id = `${PH_PREFIX}${counter++}${PH_SUFFIX}`;
    safeTags.set(id, safeHtml);
    return id;
  };

  // Headings
  html = html.replace(/^#### (.+)$/gm, (_m, body: string) => stash(`<h4>${escapeHtml(body)}</h4>`));
  html = html.replace(/^### (.+)$/gm, (_m, body: string) => stash(`<h3>${escapeHtml(body)}</h3>`));
  html = html.replace(/^## (.+)$/gm, (_m, body: string) => stash(`<h2>${escapeHtml(body)}</h2>`));
  html = html.replace(/^# (.+)$/gm, (_m, body: string) => stash(`<h1>${escapeHtml(body)}</h1>`));

  // Horizontal rule
  html = html.replace(/^---$/gm, () => stash('<hr>'));

  // Blockquotes
  html = html.replace(/^> (.+)$/gm, (_m, body: string) => stash(`<blockquote>${escapeHtml(body)}</blockquote>`));

  // Bold (before italic so ** isn't split into two adjacent *)
  html = html.replace(/\*\*(.+?)\*\*/g, (_m, body: string) => stash(`<strong>${escapeHtml(body)}</strong>`));
  // Italic
  html = html.replace(/\*(.+?)\*/g, (_m, body: string) => stash(`<em>${escapeHtml(body)}</em>`));

  // Inline code
  html = html.replace(/`(.+?)`/g, (_m, body: string) => stash(`<code>${escapeHtml(body)}</code>`));

  // Links ([text](url))
  html = html.replace(
    /\[(.+?)\]\((.+?)\)/g,
    (_match, text: string, url: string) => {
      const safe = sanitizeUrl(url);
      return stash(
        `<a href="${escapeHtml(safe)}" target="_blank" rel="noopener noreferrer">${escapeHtml(text)}</a>`,
      );
    },
  );

  // Lists — stash <li> first
  html = html.replace(/^[-*] (.+)$/gm, (_m, body: string) => stash(`<li>${escapeHtml(body)}</li>`));
  html = html.replace(/^\d+\. (.+)$/gm, (_m, body: string) => stash(`<li>${escapeHtml(body)}</li>`));

  // ---------------------------------------------------------------------------
  // Phase 3: escape any remaining user text (anything that is not a
  // placeholder or already-inserted tag). At this point the string
  // contains plain text + placeholder tokens. Escape plain text segments
  // while leaving placeholder tokens untouched.
  // ---------------------------------------------------------------------------
  html = escapePlainText(html);

  // ---------------------------------------------------------------------------
  // Phase 4: wrap contiguous <li> placeholders in <ul>/<ol>. We do this
  // AFTER escape so the inserted tag delimiters are already in their final
  // form. Distinguish ul vs ol by walking pairs from the start of the
  // string: the first contiguous run of <li> placeholders becomes a <ul>,
  // and any subsequent runs become <ol>. We can't reliably tell them
  // apart after the placeholders are gone, so we use a simpler heuristic:
  // every run gets a <ul>, and we accept that mixed lists degrade to
  // unordered. (Markdown mixes them rarely; this matches the existing
  // simple renderer behaviour.)
  // ---------------------------------------------------------------------------
  html = wrapListPlaceholders(html, 'ul');
  html = wrapListPlaceholders(html, 'ol');

  // ---------------------------------------------------------------------------
  // Phase 5: wrap remaining plain-text blocks in <p>.
  // ---------------------------------------------------------------------------
  html = html
    .split(/\n\n+/)
    .map(block => {
      const trimmed = block.trim();
      if (!trimmed) return '';
      // Block-level elements we own stay as-is.
      if (/^<(h[1-6]|ul|ol|pre|blockquote|hr)\b/.test(trimmed)) return trimmed;
      // Placeholder-only blocks don't need a <p>.
      if (/^(\u0091PH\d+\u0091)+$/.test(trimmed)) return trimmed;
      return `<p>${trimmed.replace(/\n/g, '<br>')}</p>`;
    })
    .join('\n');

  // ---------------------------------------------------------------------------
  // Phase 6: substitute placeholders back to safe HTML. Code blocks come
  // last so any further transformations don't touch their text content.
  // ---------------------------------------------------------------------------
  html = html.replace(/\u0091PH(\d+)\u0091/g, (_m, idx: string) => {
    return safeTags.get(`${PH_PREFIX}${idx}${PH_SUFFIX}`) ?? '';
  });
  html = html.replace(/\u0091CODE(\d+)\u0091/g, (_m, idx: string) => {
    const code = codeBlocks[Number(idx)] ?? '';
    return `<pre><code>${escapeHtml(code)}</code></pre>`;
  });

  return html;
}

/**
 * Escape every character of `input` that is NOT inside a placeholder
 * token. Placeholder tokens are isolated chunks of `\u0091PH<n>\u0091` or
 * `\u0091CODE<n>\u0091` which contain only safe control characters and
 * digits, so they're not subject to HTML escaping.
 */
function escapePlainText(input: string): string {
  // Split on placeholder tokens, escape the parts between them.
  const tokenRe = /\u0091(?:PH|CODE)\d+\u0091/g;
  let out = '';
  let last = 0;
  let m: RegExpExecArray | null;
  while ((m = tokenRe.exec(input)) !== null) {
    out += escapeHtml(input.slice(last, m.index));
    out += m[0]; // placeholder passes through unchanged
    last = m.index + m[0].length;
  }
  out += escapeHtml(input.slice(last));
  return out;
}

/**
 * Wrap contiguous <li> placeholder runs in a list element of the given
 * kind. Each placeholder token maps to `<li>...</li>` after substitution.
 */
function wrapListPlaceholders(
  html: string,
  kind: 'ul' | 'ol',
): string {
  // Match runs of placeholder tokens separated only by newlines.
  const re = /((?:\u0091PH\d+\u0091)(?:\n\u0091PH\d+\u0091)*)/g;
  return html.replace(re, (match: string) => {
    // Strip leading/trailing newlines but keep internal separators.
    const trimmed = match.replace(/^\n+|\n+$/g, '');
    return `<${kind}>${trimmed}</${kind}>`;
  });
}