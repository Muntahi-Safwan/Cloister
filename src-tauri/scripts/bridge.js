/**
 * Cloister service bridge.
 *
 * Injected into every service page before the site's own scripts run. The
 * backend replaces the `__CLOISTER_CONFIG__` placeholder on the last line with
 * a per-service JSON configuration object (see `src/runtime/inject.rs`).
 *
 * Responsibilities, deliberately small:
 *   1. Replace the Web Notifications API so site notifications become native
 *      system notifications routed through Cloister (focus and Veil apply to them).
 *   2. Implement Veil: blur sensitive regions with a single stylesheet, and
 *      report the site's icon so the sidebar can show it.
 *   3. Forward Cloister's keyboard shortcuts, which a focused page would
 *      otherwise swallow.
 *   4. Find in page (Ctrl+F): a small search bar that highlights matches in
 *      the text the page has loaded. The search stays inside the page.
 *
 * The bridge never sends page content anywhere and never clicks, types or
 * sends anything on the user's behalf. Find reads the page's text only to
 * search it, inside the page, when the user opens the find bar.
 *
 * @param {object}   config
 * @param {boolean}  config.veil            Initial Veil state.
 * @param {string[]} config.veilSelectors   Regions to blur; empty blurs the page.
 * @param {boolean}  config.revealOnHover   Un-blur a region while hovered.
 */
/* global __CLOISTER_CONFIG__ */
(function cloisterBridge(config) {
  'use strict';

  // Only run once, and only in the top-level document.
  if (window.top !== window || window.__CLOISTER__) return;

  /** Sends a command to the Cloister backend. Errors are ignored on purpose. */
  const invoke = (command, args) => {
    const ipc = window.__TAURI_INTERNALS__;
    if (!ipc || typeof ipc.invoke !== 'function') return Promise.resolve();
    return ipc.invoke(command, args).catch(() => undefined);
  };

  // ---------------------------------------------------------------------------
  // 1. Notifications
  // ---------------------------------------------------------------------------

  const forwardNotification = (title, options) =>
    invoke('bridge_notify', {
      title: String(title ?? ''),
      body: String((options && options.body) ?? ''),
    });

  /** Drop-in replacement for `window.Notification`. */
  class CloisterNotification extends EventTarget {
    constructor(title, options = {}) {
      super();
      this.title = String(title ?? '');
      this.body = String(options.body ?? '');
      this.tag = String(options.tag ?? '');
      this.icon = String(options.icon ?? '');
      this.data = options.data ?? null;
      this.onclick = null;
      this.onclose = null;
      this.onerror = null;
      this.onshow = null;
      forwardNotification(title, options);
      queueMicrotask(() => this.#emit('show'));
    }

    static get permission() {
      return 'granted';
    }

    static requestPermission(callback) {
      if (typeof callback === 'function') callback('granted');
      return Promise.resolve('granted');
    }

    close() {
      this.#emit('close');
    }

    #emit(type) {
      const event = new Event(type);
      this.dispatchEvent(event);
      const handler = this[`on${type}`];
      if (typeof handler === 'function') handler.call(this, event);
    }
  }

  Object.defineProperty(window, 'Notification', {
    value: CloisterNotification,
    configurable: true,
    writable: true,
  });

  if (window.ServiceWorkerRegistration) {
    window.ServiceWorkerRegistration.prototype.showNotification = function showNotification(title, options) {
      forwardNotification(title, options);
      return Promise.resolve();
    };
  }

  // Some sites ask the Permissions API before showing notifications.
  if (navigator.permissions && typeof navigator.permissions.query === 'function') {
    const originalQuery = navigator.permissions.query.bind(navigator.permissions);
    navigator.permissions.query = (descriptor) =>
      descriptor && descriptor.name === 'notifications'
        ? Promise.resolve({ name: 'notifications', state: 'granted', onchange: null })
        : originalQuery(descriptor);
  }

  // ---------------------------------------------------------------------------
  // 2. Veil
  // ---------------------------------------------------------------------------
  //
  // Uses a constructable stylesheet, which a site's Content Security Policy
  // cannot block (a <style> tag is the fallback). If the recipe's selectors
  // stop matching because the site changed its markup, the whole page is
  // blurred instead, so Veil never silently fails open.
  //
  // Regions that contain list items (chats, messages, channels, emails) are
  // blurred item by item, so hovering reveals only the one row under the
  // pointer. Regions without items are blurred as a whole. Reveals wait a
  // moment, so sweeping the mouse across a list does not flash its contents.

  const VEIL_STORAGE_KEY = '__cloisterVeil';
  const VEIL_STYLE_ID = 'cloister-veil';
  const VEIL_RECHECK_MS = 2000;
  const BLUR = 'filter: blur(9px) grayscale(0.6) !important; transition: filter 120ms ease-out !important;';
  const REVEAL = 'filter: none !important; transition-delay: 150ms !important;';
  /** Elements that usually hold one chat, message, channel or email. */
  const ITEMS = '[role="listitem"], [role="row"], [role="option"], [role="article"], [role="treeitem"], li';
  /** Only the outermost item, so a row and its nested parts are revealed together. */
  const OUTERMOST_ITEM = `:is(${ITEMS}):not(:is(${ITEMS}) *)`;

  const sheet = (() => {
    try {
      return new CSSStyleSheet();
    } catch {
      return null;
    }
  })();
  let veilTimer = null;
  let renderedCss = '';

  /** Blur targets for one region: its list items if it has any, else the region itself. */
  const targetsFor = (region) => {
    const hasItems = [...document.querySelectorAll(region)].some((el) => el.querySelector(ITEMS) !== null);
    return hasItems ? `${region} ${OUTERMOST_ITEM}` : region;
  };

  const veilCss = () => {
    const regions = config.veilSelectors.filter((r) => document.querySelector(r) !== null);
    if (regions.length === 0) return `body { ${BLUR} }`;

    const targets = regions.map(targetsFor);
    const blur = `${targets.join(',\n')} { ${BLUR} }`;
    const reveal = config.revealOnHover
      ? `\n${targets.map((t) => `${t}:hover`).join(',\n')} { ${REVEAL} }`
      : '';
    return blur + reveal;
  };

  const renderVeil = () => {
    const css = veilCss();
    if (css === renderedCss) return;
    renderedCss = css;
    if (sheet) {
      sheet.replaceSync(css);
      if (!document.adoptedStyleSheets.includes(sheet)) {
        document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
      }
      return;
    }
    let style = document.getElementById(VEIL_STYLE_ID);
    if (!style) {
      style = document.createElement('style');
      style.id = VEIL_STYLE_ID;
      (document.head || document.documentElement).appendChild(style);
    }
    style.textContent = css;
  };

  const clearVeil = () => {
    renderedCss = '';
    if (sheet) document.adoptedStyleSheets = document.adoptedStyleSheets.filter((s) => s !== sheet);
    document.getElementById(VEIL_STYLE_ID)?.remove();
  };

  /** Remembers Veil across reloads of this tab, falling back to the creation-time value. */
  const initialVeil = () => {
    try {
      const stored = sessionStorage.getItem(VEIL_STORAGE_KEY);
      if (stored !== null) return stored === '1';
    } catch {
      // Storage can be unavailable; use the configured value.
    }
    return config.veil;
  };

  const setVeil = (on) => {
    try {
      sessionStorage.setItem(VEIL_STORAGE_KEY, on ? '1' : '0');
    } catch {
      // The backend re-applies Veil on every page load anyway.
    }
    clearInterval(veilTimer);
    veilTimer = null;
    if (!on) {
      clearVeil();
      return;
    }
    renderVeil();
    // Single-page apps build their UI after load; re-check which regions exist.
    veilTimer = setInterval(renderVeil, VEIL_RECHECK_MS);
  };

  // Pick up the real layout as soon as it exists, not only on the next re-check.
  const rerenderIfVeiled = () => {
    if (veilTimer !== null) renderVeil();
  };
  document.addEventListener('DOMContentLoaded', rerenderIfVeiled);
  window.addEventListener('load', rerenderIfVeiled);

  // ---------------------------------------------------------------------------
  // 2b. Site icon
  // ---------------------------------------------------------------------------

  /** Picks the page's best icon: a touch icon if present, else the largest declared icon. */
  const bestIcon = () => {
    const touch = document.querySelector('link[rel~="apple-touch-icon"][href]');
    if (touch) return touch.href;
    const size = (link) =>
      Math.max(0, ...(link.sizes?.value || '').split(/\s+/).map((s) => parseInt(s, 10) || 0));
    const icons = [...document.querySelectorAll('link[rel~="icon"][href]')].sort((a, b) => size(b) - size(a));
    return icons[0]?.href || new URL('/favicon.ico', location.origin).href;
  };

  // Report once per page, after the site settles (some sites swap icons to show unread counts).
  window.addEventListener('load', () => {
    setTimeout(() => invoke('bridge_icon', { href: bestIcon() }), 1500);
  });

  // ---------------------------------------------------------------------------
  // 3. Shortcuts
  // ---------------------------------------------------------------------------

  /** Maps a keyboard event to a Cloister action, or `null` if it is not ours. */
  const shortcutFor = (event) => {
    // Cmd on macOS, Ctrl elsewhere; the other modifier means "not ours".
    const primary = config.mac ? event.metaKey : event.ctrlKey;
    const other = config.mac ? event.ctrlKey : event.metaKey;
    if (!primary || event.altKey || other) return null;
    const key = event.key.toLowerCase();
    if (key === '=' || key === '+') return 'zoom:in';
    if (key === '-' || key === '_') return 'zoom:out';
    if (key === '0' && !event.shiftKey) return 'zoom:reset';
    if (event.shiftKey) {
      if (key === 'k') return 'palette';
      if (key === 'p') return 'veil';
      if (key === 'f') return 'focus';
      if (key === 'l') return 'lock';
      return null;
    }
    if (key === 'f') return 'find';
    if (/^[1-9]$/.test(key)) return `switch:${key}`;
    return null;
  };

  window.addEventListener(
    'keydown',
    (event) => {
      if (event.key === 'F3' && findIsOpen()) {
        event.preventDefault();
        event.stopPropagation();
        stepFind(event.shiftKey ? -1 : 1);
        return;
      }
      const action = shortcutFor(event);
      if (!action) return;
      event.preventDefault();
      event.stopPropagation();
      // Find runs inside the page; everything else is handled by Cloister.
      if (action === 'find') openFind();
      else invoke('bridge_shortcut', { action });
    },
    { capture: true },
  );

  // ---------------------------------------------------------------------------
  // 4. Find in page
  // ---------------------------------------------------------------------------
  //
  // Matches are found in the page's text nodes and painted with the CSS Custom
  // Highlight API, so the page's own DOM is never changed. Where that API is
  // missing, the current match is selected instead. Only text the page has
  // loaded can be found: chat apps load older messages as you scroll.
  //
  // The bar lives in a closed shadow root so the site's styles cannot reach it,
  // and is built with DOM calls only, because some sites (Gmail) forbid
  // `innerHTML` through Trusted Types.

  const FIND_LIMIT = 1000;
  const FIND_DEBOUNCE_MS = 120;
  const FIND_SKIP = new Set(['SCRIPT', 'STYLE', 'NOSCRIPT', 'TEMPLATE', 'TEXTAREA', 'HEAD', 'TITLE']);
  const HIGHLIGHT_ALL = 'cloister-find';
  const HIGHLIGHT_CURRENT = 'cloister-find-current';
  const canHighlight = typeof CSS !== 'undefined' && 'highlights' in CSS && typeof Highlight === 'function';

  const FIND_PAGE_CSS = `
    ::highlight(${HIGHLIGHT_ALL}) { background-color: rgba(255, 200, 60, 0.45); }
    ::highlight(${HIGHLIGHT_CURRENT}) { background-color: rgb(255, 140, 40); color: #111; }`;

  // Styles live in the shadow root's own sheet: strict sites block inline styles.
  const FIND_BAR_CSS = `
    :host { all: initial; position: fixed; top: 12px; right: 16px; z-index: 2147483647; }
    .bar {
      display: flex; align-items: center; gap: 2px; padding: 5px 5px 5px 12px;
      border-radius: 10px; border: 1px solid rgba(255, 255, 255, 0.12);
      background: #1f2224; color: #e8eaed; box-shadow: 0 8px 28px rgba(0, 0, 0, 0.35);
      font: 13px/1.3 system-ui, -apple-system, "Segoe UI", sans-serif;
    }
    input {
      width: 210px; padding: 4px 0; border: 0; outline: 0;
      background: transparent; color: inherit; font: inherit;
    }
    input::placeholder { color: #9aa0a6; }
    .count {
      min-width: 70px; padding: 0 6px; text-align: right; white-space: nowrap;
      color: #9aa0a6; font-variant-numeric: tabular-nums;
    }
    button {
      all: unset; box-sizing: border-box; display: grid; place-items: center;
      width: 26px; height: 26px; border-radius: 6px; cursor: pointer; color: inherit;
    }
    button:hover { background: rgba(255, 255, 255, 0.1); }
    button:focus-visible { outline: 2px solid #7fc8a9; outline-offset: -2px; }
    svg { width: 15px; height: 15px; fill: none; stroke: currentColor; stroke-width: 2;
          stroke-linecap: round; stroke-linejoin: round; }
    @media (prefers-color-scheme: light) {
      .bar { background: #fff; color: #1f1f1f; border-color: rgba(0, 0, 0, 0.12);
             box-shadow: 0 8px 28px rgba(0, 0, 0, 0.16); }
      input::placeholder, .count { color: #5f6368; }
      button:hover { background: rgba(0, 0, 0, 0.06); }
    }`;

  const find = { host: null, input: null, count: null, matches: [], index: -1, query: '', timer: 0 };
  let findPageSheet = null;

  const findIsOpen = () => Boolean(find.host && find.host.isConnected);

  /** Every case-insensitive match of `query` in visible text, as ranges, in page order. */
  const findRanges = (query) => {
    const needle = query.toLowerCase();
    const ranges = [];
    if (!needle || !document.body) return ranges;
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, {
      acceptNode(node) {
        const parent = node.parentElement;
        if (!parent || FIND_SKIP.has(parent.nodeName.toUpperCase())) return NodeFilter.FILTER_REJECT;
        if (typeof parent.checkVisibility === 'function' && !parent.checkVisibility()) {
          return NodeFilter.FILTER_REJECT;
        }
        return NodeFilter.FILTER_ACCEPT;
      },
    });
    for (let node = walker.nextNode(); node && ranges.length < FIND_LIMIT; node = walker.nextNode()) {
      const text = node.data.toLowerCase();
      let at = text.indexOf(needle);
      while (at !== -1 && ranges.length < FIND_LIMIT) {
        const range = document.createRange();
        range.setStart(node, Math.min(at, node.length));
        range.setEnd(node, Math.min(at + needle.length, node.length));
        ranges.push(range);
        at = text.indexOf(needle, at + needle.length);
      }
    }
    return ranges;
  };

  const clearHighlights = () => {
    if (canHighlight) {
      CSS.highlights.delete(HIGHLIGHT_ALL);
      CSS.highlights.delete(HIGHLIGHT_CURRENT);
    }
  };

  /** Shows the matches, scrolls to the current one and updates the counter. */
  const paintFind = () => {
    const current = find.matches[find.index];
    if (canHighlight) {
      CSS.highlights.set(HIGHLIGHT_ALL, new Highlight(...find.matches));
      if (current) CSS.highlights.set(HIGHLIGHT_CURRENT, new Highlight(current));
      else CSS.highlights.delete(HIGHLIGHT_CURRENT);
    } else if (current) {
      const selection = window.getSelection();
      selection.removeAllRanges();
      selection.addRange(current);
    }
    const target = current && current.startContainer.parentElement;
    if (target && typeof target.scrollIntoView === 'function') {
      target.scrollIntoView({ block: 'center', inline: 'nearest' });
    }

    const total = find.matches.length;
    if (!find.query) find.count.textContent = '';
    else if (!total) find.count.textContent = 'No results';
    else find.count.textContent = `${find.index + 1} of ${total}${total >= FIND_LIMIT ? '+' : ''}`;
  };

  const runFind = () => {
    window.clearTimeout(find.timer);
    find.query = find.input.value;
    find.matches = findRanges(find.query);
    find.index = find.matches.length ? 0 : -1;
    paintFind();
  };

  /** Moves to the next (`1`) or previous (`-1`) match, searching again if the page changed. */
  const stepFind = (delta) => {
    const current = find.matches[find.index];
    const stale = find.query !== find.input.value || (current && !current.startContainer.isConnected);
    if (stale) {
      runFind();
      if (delta < 0 && find.matches.length) {
        find.index = find.matches.length - 1;
        paintFind();
      }
      return;
    }
    if (!find.matches.length) return;
    find.index = (find.index + delta + find.matches.length) % find.matches.length;
    paintFind();
  };

  const closeFind = () => {
    window.clearTimeout(find.timer);
    clearHighlights();
    if (find.host) find.host.remove();
    find.matches = [];
    find.index = -1;
  };

  const svgIcon = (path) => {
    const ns = 'http://www.w3.org/2000/svg';
    const svg = document.createElementNS(ns, 'svg');
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('aria-hidden', 'true');
    const line = document.createElementNS(ns, 'path');
    line.setAttribute('d', path);
    svg.append(line);
    return svg;
  };

  const findButton = (label, path, onClick) => {
    const button = document.createElement('button');
    button.type = 'button';
    button.title = label;
    button.setAttribute('aria-label', label);
    button.append(svgIcon(path));
    button.addEventListener('click', onClick);
    return button;
  };

  const buildFindBar = () => {
    const host = document.createElement('cloister-find');
    const root = host.attachShadow({ mode: 'closed' });
    try {
      const style = new CSSStyleSheet();
      style.replaceSync(FIND_BAR_CSS);
      root.adoptedStyleSheets = [style];
    } catch {
      const style = document.createElement('style');
      style.textContent = FIND_BAR_CSS;
      root.append(style);
    }

    const bar = document.createElement('div');
    bar.className = 'bar';
    bar.setAttribute('role', 'search');
    const input = document.createElement('input');
    input.type = 'text';
    input.placeholder = 'Find in page';
    input.spellcheck = false;
    input.autocomplete = 'off';
    input.setAttribute('aria-label', 'Find in page');
    const count = document.createElement('span');
    count.className = 'count';
    count.setAttribute('aria-live', 'polite');
    bar.append(
      input,
      count,
      findButton('Previous match (Shift+Enter)', 'M18 15l-6-6-6 6', () => stepFind(-1)),
      findButton('Next match (Enter)', 'M6 9l6 6 6-6', () => stepFind(1)),
      findButton('Close (Esc)', 'M18 6L6 18M6 6l12 12', closeFind),
    );
    root.append(bar);

    input.addEventListener('input', () => {
      window.clearTimeout(find.timer);
      find.timer = window.setTimeout(runFind, FIND_DEBOUNCE_MS);
    });
    input.addEventListener('keydown', (event) => {
      if (event.isComposing) return;
      if (event.key === 'Enter') {
        event.preventDefault();
        stepFind(event.shiftKey ? -1 : 1);
      } else if (event.key === 'Escape') {
        event.preventDefault();
        closeFind();
      }
    });
    // Keep typing in the bar away from the site's own keyboard shortcuts.
    for (const type of ['keydown', 'keyup', 'keypress']) {
      bar.addEventListener(type, (event) => event.stopPropagation());
    }

    find.host = host;
    find.input = input;
    find.count = count;
  };

  /** Opens the find bar (or focuses it), starting from the selected text if any. */
  const openFind = () => {
    if (canHighlight && !findPageSheet) {
      try {
        findPageSheet = new CSSStyleSheet();
        findPageSheet.replaceSync(FIND_PAGE_CSS);
        document.adoptedStyleSheets = [...document.adoptedStyleSheets, findPageSheet];
      } catch {
        findPageSheet = null;
      }
    }
    if (!find.host) buildFindBar();
    if (!find.host.isConnected) document.documentElement.append(find.host);

    const selected = String(window.getSelection() || '').trim();
    if (selected && selected.length <= 100 && !selected.includes('\n')) find.input.value = selected;
    find.input.focus();
    find.input.select();
    if (find.input.value) runFind();
  };

  // ---------------------------------------------------------------------------
  // Public surface used by the backend (`window.__CLOISTER__.setVeil(true)`, `.find()`).
  // ---------------------------------------------------------------------------

  Object.defineProperty(window, '__CLOISTER__', {
    value: Object.freeze({ setVeil, find: openFind }),
    configurable: false,
    writable: false,
  });

  if (initialVeil()) setVeil(true);
})(__CLOISTER_CONFIG__);
