/**
 * Lightweight Markdown renderer for source notes.
 * Supports: headings, bold, italic, inline code, code blocks,
 * unordered/ordered lists, links, blockquotes, horizontal rules.
 * No external dependencies.
 */

export function renderMarkdown(md: string): string {
  if (!md) return '';

  // Escape HTML to prevent XSS
  let html = md
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  // Code blocks (``` ... ```)
  html = html.replace(/```([\s\S]*?)```/g, (_match, code: string) => {
    return `<pre><code>${code.trim()}</code></pre>`;
  });

  // Headings (# H1 ~ #### H4)
  html = html.replace(/^#### (.+)$/gm, '<h4>$1</h4>');
  html = html.replace(/^### (.+)$/gm, '<h3>$1</h3>');
  html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');
  html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');

  // Horizontal rule (---)
  html = html.replace(/^---$/gm, '<hr>');

  // Blockquotes (> text)
  html = html.replace(/^> (.+)$/gm, '<blockquote>$1</blockquote>');

  // Bold (**text**)
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');

  // Italic (*text*)
  html = html.replace(/\*(.+?)\*/g, '<em>$1</em>');

  // Inline code (`code`)
  html = html.replace(/`(.+?)`/g, '<code>$1</code>');

  // Links ([text](url))
  html = html.replace(
    /\[(.+?)\]\((.+?)\)/g,
    '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>'
  );

  // Unordered lists (- item or * item)
  html = html.replace(/^[\-\*] (.+)$/gm, '<li>$1</li>');
  html = html.replace(/((?:<li>.*<\/li>\n?)+)/g, '<ul>$1</ul>');

  // Ordered lists (1. item)
  html = html.replace(/^\d+\. (.+)$/gm, '<li>$1</li>');
  // Wrap consecutive <li> in <ol> (only if not already in <ul>)
  html = html.replace(/(?<!<\/ul>)((?:<li>.*<\/li>\n?)+)(?!<\/ul>)/g, '<ol>$1</ol>');

  // Paragraphs: wrap remaining lines in <p>
  // Split by double newlines for paragraph boundaries
  html = html
    .split(/\n\n+/)
    .map(block => {
      const trimmed = block.trim();
      if (!trimmed) return '';
      // Don't wrap blocks that already have HTML tags
      if (/^<(h[1-6]|ul|ol|li|pre|blockquote|hr)/.test(trimmed)) {
        return trimmed;
      }
      return `<p>${trimmed.replace(/\n/g, '<br>')}</p>`;
    })
    .join('\n');

  return html;
}
