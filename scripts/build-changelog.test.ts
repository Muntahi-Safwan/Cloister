import { describe, expect, it } from 'vitest';

import { inline, parseChangelog, renderPage } from './build-changelog.mjs';

const SAMPLE = `# Changelog

Intro text that is not part of any release.

## [Unreleased]

## [1.1.0] - 2026-02-03

A short note about this release.

### Added

- **Big thing.** Spans
  two lines with \`code\`.
- Small thing.

- After a blank line, same list.

### Fixed

- A [linked](https://example.com) fix.

## [1.0.0] - 2026-01-01

### Added

- First.

[1.1.0]: https://github.com/o/r/releases/tag/v1.1.0
`;

describe('changelog page', () => {
  const releases = parseChangelog(SAMPLE);

  it('drops empty releases and keeps the order', () => {
    expect(releases.map((r) => r.version)).toEqual(['1.1.0', '1.0.0']);
  });

  it('reads dates, links and intro paragraphs', () => {
    expect(releases[0].date).toBe('2026-02-03');
    expect(releases[0].link).toBe('https://github.com/o/r/releases/tag/v1.1.0');
    expect(releases[1].link).toBeNull();
    expect(releases[0].intro).toEqual(['A short note about this release.']);
  });

  it('joins wrapped list items and keeps items split by blank lines', () => {
    const added = releases[0].groups[0];
    expect(added.kind).toBe('Added');
    expect(added.items).toEqual([
      '**Big thing.** Spans two lines with `code`.',
      'Small thing.',
      'After a blank line, same list.',
    ]);
  });

  it('formats inline Markdown and escapes HTML', () => {
    expect(inline('**A** `x<y>` [b](https://c.d) *e*')).toBe(
      '<strong>A</strong> <code>x&lt;y&gt;</code> <a href="https://c.d">b</a> <em>e</em>',
    );
    expect(inline('<script>')).toBe('&lt;script&gt;');
  });

  it('marks only the newest release as latest and anchors every version', () => {
    const html = renderPage(releases);
    expect(html.match(/release-latest/g)).toHaveLength(1);
    expect(html).toContain('id="v1.1.0"');
    expect(html).toContain('id="v1.0.0"');
    expect(html).toContain('<time datetime="2026-02-03">3 February 2026</time>');
  });
});
