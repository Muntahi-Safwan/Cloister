#!/usr/bin/env node
// Builds site/changelog.html from CHANGELOG.md, so the landing page's changelog
// always matches the repository's. No dependencies: CHANGELOG.md follows Keep a
// Changelog, so only a small part of Markdown is needed.
//
//   node scripts/build-changelog.mjs           writes site/changelog.html
//   node scripts/build-changelog.mjs --check   fails if the file is out of date

import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const SOURCE = join(root, 'CHANGELOG.md');
const TARGET = join(root, 'site', 'changelog.html');
const REPO = 'https://github.com/Muntahi-Safwan/Cloister';

// Parsing ---------------------------------------------------------------------------------------

/**
 * Splits the changelog into releases:
 * [{ version, date, link, intro: [paragraph], groups: [{ kind, items: [markdown] }] }].
 * Empty sections (such as an empty "Unreleased") are dropped.
 */
export function parseChangelog(markdown) {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n');
  const links = {};
  for (const line of lines) {
    const match = /^\[([^\]]+)\]:\s*(\S+)/.exec(line);
    if (match) links[match[1]] = match[2];
  }

  const releases = [];
  let release = null;
  let group = null;
  let item = null;
  let paragraph = null;

  const endItem = () => {
    if (item !== null) group.items.push(item.trim());
    item = null;
  };
  const endParagraph = () => {
    if (paragraph !== null && release) release.intro.push(paragraph.trim());
    paragraph = null;
  };

  for (const line of lines) {
    const version = /^## \[([^\]]+)\](?:\s*-\s*(\d{4}-\d{2}-\d{2}))?/.exec(line);
    if (version) {
      endItem();
      endParagraph();
      release = {
        version: version[1],
        date: version[2] ?? null,
        link: links[version[1]] ?? null,
        intro: [],
        groups: [],
      };
      releases.push(release);
      group = null;
      continue;
    }
    if (!release || /^\[[^\]]+\]:/.test(line)) continue;

    const heading = /^### (.+)/.exec(line);
    if (heading) {
      endItem();
      endParagraph();
      group = { kind: heading[1].trim(), items: [] };
      release.groups.push(group);
      continue;
    }

    if (group && line.startsWith('- ')) {
      endItem();
      item = line.slice(2);
    } else if (group && item !== null && /^\s+\S/.test(line)) {
      item += ` ${line.trim()}`;
    } else if (line.trim() === '') {
      endItem();
      endParagraph();
    } else if (!group) {
      paragraph = paragraph === null ? line : `${paragraph} ${line.trim()}`;
    }
  }
  endItem();
  endParagraph();

  return releases.filter((r) => r.groups.some((g) => g.items.length) || r.intro.length);
}

const escapeHtml = (text) =>
  text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

/** Inline Markdown: `code`, **bold**, *emphasis* and [links](url). */
export function inline(markdown) {
  const codes = [];
  let html = escapeHtml(markdown).replace(/`([^`]+)`/g, (_, code) => {
    codes.push(code);
    return `\uE000${codes.length - 1}\uE000`;
  });
  html = html
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/(^|[^*])\*([^*\s][^*]*?)\*(?!\*)/g, '$1<em>$2</em>')
    .replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, text, url) => `<a href="${url}">${text}</a>`);
  return html.replace(/\uE000(\d+)\uE000/g, (_, i) => `<code>${codes[Number(i)]}</code>`);
}

// Rendering -------------------------------------------------------------------------------------

const MONTHS = [
  'January',
  'February',
  'March',
  'April',
  'May',
  'June',
  'July',
  'August',
  'September',
  'October',
  'November',
  'December',
];

function formatDate(iso) {
  const [year, month, day] = iso.split('-').map(Number);
  return `${day} ${MONTHS[month - 1]} ${year}`;
}

const idFor = (version) => (/^\d/.test(version) ? `v${version}` : version.toLowerCase());

const MARK = `<svg class="mark" viewBox="0 0 24 24" width="22" height="22" aria-hidden="true"><path d="M5 21V10a7 7 0 0 1 14 0v11" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/><path d="M3 21h18" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/><circle cx="12" cy="15" r="1.7" fill="var(--accent)"/></svg>`;

function renderRelease(release, index) {
  const id = idFor(release.version);
  const isLatest = index === 0 && release.version.toLowerCase() !== 'unreleased';
  const date = release.date
    ? `<time datetime="${release.date}">${formatDate(release.date)}</time>`
    : '<span>Not released yet</span>';
  const tag = release.link
    ? `<a class="release-link" href="${release.link}">${release.link.includes('/releases/') ? 'Release and downloads' : 'Changes on GitHub'}</a>`
    : '';
  const intro = release.intro.map((p) => `<p class="release-intro">${inline(p)}</p>`).join('');
  const groups = release.groups
    .filter((g) => g.items.length)
    .map(
      (g) => `
          <section class="change-group" data-kind="${escapeHtml(g.kind.toLowerCase())}">
            <h3>${escapeHtml(g.kind)}</h3>
            <ul>${g.items
              .map(
                (item) => `
              <li>${inline(item)}</li>`,
              )
              .join('')}
            </ul>
          </section>`,
    )
    .join('');
  return `
      <article class="page split release" id="${id}">
        <header class="margin release-meta">
          <h2 class="release-version"><a href="#${id}">${escapeHtml(release.version)}</a></h2>
          <p class="release-date mono">${date}</p>
          ${isLatest ? '<p class="release-latest mono">Latest</p>' : ''}
          ${tag}
        </header>
        <div class="release-body">${intro}${groups}
        </div>
      </article>`;
}

export function renderPage(releases) {
  const index = releases
    .map((r) => `<li><a href="#${idFor(r.version)}">${escapeHtml(r.version)}</a></li>`)
    .join('');
  return `<!doctype html>
<!-- Generated from CHANGELOG.md by scripts/build-changelog.mjs. Do not edit by hand. -->
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Changelog · Cloister</title>
    <meta name="description" content="Every release of Cloister, newest first: what was added, changed and fixed." />
    <meta name="theme-color" content="#f2eee5" />
    <link rel="icon" href="icon.svg" type="image/svg+xml" />
    <link rel="apple-touch-icon" href="apple-touch-icon.png" />
    <meta property="og:type" content="website" />
    <meta property="og:title" content="Changelog · Cloister" />
    <meta property="og:description" content="Every release of Cloister, newest first." />
    <meta property="og:image" content="https://muntahi-safwan.github.io/Cloister/og.png" />
    <link rel="preload" href="fonts/newsreader-latin-400-normal.woff2" as="font" type="font/woff2" crossorigin />
    <link rel="stylesheet" href="styles.css" />
  </head>
  <body class="changelog-page">
    <a class="skip" href="#top">Skip to content</a>
    <header class="nav is-scrolled">
      <div class="page nav-inner">
        <a class="brand" href="./" aria-label="Cloister home">${MARK}<span>Cloister</span></a>
        <nav class="nav-links" aria-label="Main">
          <a href="./#features">Features</a>
          <a href="./#memory">Memory</a>
          <a href="./#faq">Questions</a>
          <a href="changelog.html" aria-current="page">Changelog</a>
        </nav>
        <div class="nav-end">
          <a class="btn btn-small" href="./#install">Download</a>
        </div>
      </div>
    </header>

    <main id="top">
      <section class="changelog-head">
        <div class="page split">
          <p class="margin margin-note mono">Updated with every release</p>
          <div>
            <h1>Changelog</h1>
            <p class="lede">
              Every release of Cloister, newest first. Installed copies update themselves, so this is mostly here
              for the curious.
            </p>
            <nav class="release-index" aria-label="Releases"><ul>${index}</ul></nav>
          </div>
        </div>
      </section>
${releases.map(renderRelease).join('\n')}
    </main>

    <footer class="colophon">
      <div class="page colophon-inner">
        <p class="brand">${MARK}<span>Cloister</span></p>
        <p>
          Generated from <a href="${REPO}/blob/main/CHANGELOG.md">CHANGELOG.md</a>, which follows
          <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a> and semantic versioning.
        </p>
        <p class="colophon-links">
          <a href="./">Home</a>
          <a href="${REPO}/releases">Releases</a>
          <a href="${REPO}">Source</a>
        </p>
      </div>
    </footer>
  </body>
</html>
`;
}

// Command line ----------------------------------------------------------------------------------

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const html = renderPage(parseChangelog(readFileSync(SOURCE, 'utf8')));
  if (process.argv.includes('--check')) {
    let current = '';
    try {
      current = readFileSync(TARGET, 'utf8');
    } catch {
      // Missing counts as out of date.
    }
    if (current !== html) {
      console.error('site/changelog.html is out of date. Run: pnpm site:changelog');
      process.exit(1);
    }
    console.log('site/changelog.html is up to date.');
  } else {
    writeFileSync(TARGET, html);
    console.log('Wrote site/changelog.html');
  }
}
