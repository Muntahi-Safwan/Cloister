/**
 * Cloister service bridge.
 *
 * Injected into every service page before the site's own scripts run. The
 * backend replaces the `__CLOISTER_CONFIG__` placeholder on the last line with
 * a per-service JSON configuration object (see `src/runtime/inject.rs`).
 *
 * Responsibilities, deliberately small:
 *   1. Replace the Web Notifications API so site notifications become native
 *      Windows toasts routed through Cloister (focus and Veil apply to them).
 *   2. Implement Veil: blur sensitive regions with a single stylesheet, and
 *      report the site's icon so the sidebar can show it.
 *   3. Forward Cloister's keyboard shortcuts, which a focused page would
 *      otherwise swallow.
 *
 * The bridge never reads message content from the page and never clicks,
 * types or sends anything on the user's behalf.
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
    if (/^[1-9]$/.test(key)) return `switch:${key}`;
    return null;
  };

  window.addEventListener(
    'keydown',
    (event) => {
      const action = shortcutFor(event);
      if (!action) return;
      event.preventDefault();
      event.stopPropagation();
      invoke('bridge_shortcut', { action });
    },
    { capture: true },
  );

  // ---------------------------------------------------------------------------
  // Public surface used by the backend (`window.__CLOISTER__.setVeil(true)`).
  // ---------------------------------------------------------------------------

  Object.defineProperty(window, '__CLOISTER__', {
    value: Object.freeze({ setVeil }),
    configurable: false,
    writable: false,
  });

  if (initialVeil()) setVeil(true);
})(__CLOISTER_CONFIG__);
