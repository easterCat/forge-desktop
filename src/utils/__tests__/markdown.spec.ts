import { describe, it, expect } from 'vitest';
import { renderMarkdown } from '../markdown';

describe('renderMarkdown', () => {
  it('returns empty string for falsy input', () => {
    expect(renderMarkdown('')).toBe('');
    expect(renderMarkdown(null as any)).toBe('');
    expect(renderMarkdown(undefined as any)).toBe('');
  });

  it('escapes HTML special characters', () => {
    // '&copy;' in text is escaped to '&amp;copy;'
    // '&lt;' in text is a string literal that gets escaped to '&amp;lt;'
    expect(renderMarkdown('&copy;')).toContain('&amp;copy;');
    expect(renderMarkdown('&lt;3')).toContain('&amp;lt;3');
    // Plain '&' is escaped
    expect(renderMarkdown('foo & bar')).toContain('&amp;');
  });

  describe('headings', () => {
    it('renders h1', () => {
      expect(renderMarkdown('# Hello')).toContain('<h1>Hello</h1>');
    });

    it('renders h2', () => {
      expect(renderMarkdown('## Hello')).toContain('<h2>Hello</h2>');
    });

    it('renders h3', () => {
      expect(renderMarkdown('### Hello')).toContain('<h3>Hello</h3>');
    });

    it('renders h4', () => {
      expect(renderMarkdown('#### Hello')).toContain('<h4>Hello</h4>');
    });
  });

  describe('horizontal rule', () => {
    it('renders --- as hr', () => {
      expect(renderMarkdown('---')).toContain('<hr>');
    });
  });

  describe('blockquote', () => {
    it('renders > text as blockquote', () => {
      expect(renderMarkdown('> quoted text')).toContain('<blockquote>quoted text</blockquote>');
    });
  });

  describe('bold', () => {
    it('renders **text** as strong', () => {
      expect(renderMarkdown('**bold**')).toContain('<strong>bold</strong>');
    });

    it('renders **text** inside other content', () => {
      expect(renderMarkdown('This is **bold** text')).toContain('<strong>bold</strong>');
    });
  });

  describe('italic', () => {
    it('renders *text* as em', () => {
      expect(renderMarkdown('*italic*')).toContain('<em>italic</em>');
    });
  });

  describe('inline code', () => {
    it('renders `code` as code', () => {
      expect(renderMarkdown('`console.log`')).toContain('<code>console.log</code>');
    });
  });

  describe('code blocks', () => {
    it('renders ``` blocks as pre/code', () => {
      const html = renderMarkdown('```\nconst x = 1;\n```');
      expect(html).toContain('<pre>');
      expect(html).toContain('<code>');
      expect(html).toContain('const x = 1');
    });

    it('renders ```js blocks with language', () => {
      const html = renderMarkdown('```js\nconst x = 1;\n```');
      expect(html).toContain('<pre>');
      expect(html).toContain('<code>');
    });
  });

  describe('links', () => {
    it('renders [text](url) as anchor', () => {
      const html = renderMarkdown('[Click here](https://example.com)');
      expect(html).toContain('<a href="https://example.com"');
      expect(html).toContain('target="_blank"');
    });
  });

  describe('unordered lists', () => {
    it('renders - item as ul/li', () => {
      const html = renderMarkdown('- item one\n- item two');
      expect(html).toContain('<ul>');
      expect(html).toContain('<li>item one</li>');
    });

    it('renders * item as ul/li', () => {
      const html = renderMarkdown('* item');
      expect(html).toContain('<ul>');
      expect(html).toContain('<li>item</li>');
    });
  });

  describe('ordered lists', () => {
    it('renders 1. item as ol/li', () => {
      const html = renderMarkdown('1. first\n2. second');
      expect(html).toContain('<ol>');
      expect(html).toContain('<li>first</li>');
    });
  });

  describe('paragraphs', () => {
    it('wraps plain lines in p tags', () => {
      const html = renderMarkdown('Line one\n\nLine two');
      expect(html).toContain('<p>Line one</p>');
      expect(html).toContain('<p>Line two</p>');
    });
  });

  it('handles mixed content', () => {
    const md = '# Title\n\n**Bold** and *italic*\n\n- list item';
    const html = renderMarkdown(md);
    expect(html).toContain('<h1>Title</h1>');
    expect(html).toContain('<strong>Bold</strong>');
    expect(html).toContain('<em>italic</em>');
    expect(html).toContain('<ul>');
  });

  describe('XSS protection', () => {
    it('blocks javascript: URLs in markdown links', () => {
      const html = renderMarkdown('[click me](javascript:alert(1))');
      expect(html).not.toContain('javascript:');
      expect(html).toContain('href="#"');
    });

    it('blocks data: URLs in markdown links', () => {
      const html = renderMarkdown('[click me](data:text/html,<script>alert(1)</script>)');
      expect(html).not.toContain('data:text/html');
      expect(html).toContain('href="#"');
    });

    it('blocks vbscript: URLs in markdown links', () => {
      const html = renderMarkdown('[click me](vbscript:msgbox(1))');
      expect(html).not.toContain('vbscript:');
    });

    it('allows https: URLs', () => {
      const html = renderMarkdown('[ok](https://example.com)');
      expect(html).toContain('href="https://example.com"');
    });

    it('allows mailto: URLs', () => {
      const html = renderMarkdown('[mail](mailto:a@b.com)');
      expect(html).toContain('mailto:a@b.com');
    });

    it('allows relative URLs starting with /', () => {
      const html = renderMarkdown('[home](/home)');
      expect(html).toContain('href="/home"');
    });

    it('strips control characters from URLs', () => {
      const html = renderMarkdown('[evil](https://example.com\njavascript:alert(1))');
      expect(html).not.toMatch(/href="[^"]*javascript:/);
    });

    it('escapes <script> tags in plain text', () => {
      const html = renderMarkdown('<script>alert(1)</script>');
      expect(html).not.toContain('<script>');
      expect(html).toContain('&lt;script&gt;');
    });

    it('escapes HTML inside code blocks', () => {
      const html = renderMarkdown('```\n<script>alert(1)</script>\n```');
      expect(html).toContain('&lt;script&gt;');
      expect(html).not.toContain('<script>alert(1)</script>');
    });

    it('escapes img tag injection in text', () => {
      const html = renderMarkdown('<img src=x onerror=alert(1)>');
      expect(html).not.toContain('<img');
      expect(html).toContain('&lt;img');
    });

    it('escapes inline event handlers in markdown text', () => {
      const html = renderMarkdown('Hello <span onclick="alert(1)">world</span>');
      expect(html).not.toContain('<span onclick');
      expect(html).toContain('&lt;span');
    });
  });
});
