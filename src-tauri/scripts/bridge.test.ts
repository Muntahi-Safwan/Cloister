// @vitest-environment happy-dom
/**
 * Tests for the service bridge, run in a simulated browser DOM.
 * The bridge is loaded the same way the backend injects it: source + `(config);`.
 */
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import source from './bridge.js?raw';

type Invoke = ReturnType<typeof vi.fn>;

function loadBridge(config: { veil: boolean; veilSelectors: string[]; revealOnHover: boolean }): Invoke {
  const invoke = vi.fn(() => Promise.resolve());
  (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = { invoke };
  // Same substitution the backend performs (src/runtime/inject.rs).
  const script = source.replace('})(__CLOISTER_CONFIG__);', `})(${JSON.stringify(config)});`);
  // Bind `window` explicitly: the test environment's global object is not the DOM window.
  new Function('window', script)(window);
  return invoke;
}

const veilText = () =>
  [
    ...[...document.adoptedStyleSheets].flatMap((s) => [...s.cssRules].map((r) => r.cssText)),
    document.getElementById('cloister-veil')?.textContent ?? '',
  ].join('\n');

const api = () =>
  (window as unknown as { __CLOISTER__: { setVeil: (on: boolean) => void; find: () => void } }).__CLOISTER__;

// The find bar uses a closed shadow root; keep a handle to it for the tests.
let findRoot: ShadowRoot | null = null;
const attachShadow = Element.prototype.attachShadow;
Element.prototype.attachShadow = function (init: ShadowRootInit) {
  const root = attachShadow.call(this, init);
  if (this.localName === 'cloister-find') findRoot = root;
  return root;
};

const findInput = () => findRoot?.querySelector('input') as HTMLInputElement;
const findCount = () => findRoot?.querySelector('.count')?.textContent ?? '';
const findHost = () => document.querySelector('cloister-find');
const press = (key: string, init: KeyboardEventInit = {}) =>
  findInput().dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, ...init }));
const search = (query: string) => {
  findInput().value = query;
  press('Enter');
};

describe('bridge', () => {
  // The bridge installs itself once per window (like a real page), so load it once.
  let invoke: Invoke;

  beforeAll(() => {
    invoke = loadBridge({ veil: false, veilSelectors: ['#pane-side'], revealOnHover: true });
  });

  beforeEach(() => {
    document.body.innerHTML = '<div id="pane-side">chats</div>';
    sessionStorage.clear();
    api().setVeil(false);
    invoke.mockClear();
  });

  it('forwards notifications and reports permission as granted', () => {
    const Shim = (window as unknown as { Notification: typeof Notification }).Notification;
    expect(Shim.permission).toBe('granted');
    new Shim('Ayesha', { body: 'hello' });
    expect(invoke).toHaveBeenCalledWith('bridge_notify', { title: 'Ayesha', body: 'hello' });
  });

  it('blurs a region without list items as a whole, with hover-to-peek', () => {
    api().setVeil(true);
    expect(veilText()).toMatch(/#pane-side\s*\{/);
    expect(veilText()).toContain('#pane-side:hover');
    api().setVeil(false);
    expect(veilText()).not.toContain('blur');
  });

  it('blurs list regions item by item so hovering reveals one row', () => {
    document.body.innerHTML =
      '<div id="pane-side"><div role="listitem">Ayesha</div><div role="listitem">Design team</div></div>';
    api().setVeil(true);
    const css = veilText();
    expect(css).toContain('#pane-side :is(');
    expect(css).toContain(':not(:is(');
    expect(css).toMatch(/\):hover/);
    expect(css).not.toMatch(/#pane-side\s*\{/);

    // happy-dom cannot evaluate `:not(:is(...) *)`; the selector was verified in Chrome
    // (the WebView2 engine) to match each top-level row once and skip nested items.
  });

  it('falls back to blurring the whole page when regions are missing', () => {
    document.body.innerHTML = '<main>a redesigned page</main>';
    api().setVeil(true);
    expect(veilText()).toContain('body');
    expect(veilText()).not.toContain(':hover');
  });

  it('remembers Veil across reloads of the tab', () => {
    api().setVeil(true);
    expect(sessionStorage.getItem('__cloisterVeil')).toBe('1');
  });

  it('forwards only Cloister shortcuts', () => {
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'P', ctrlKey: true, shiftKey: true }));
    window.dispatchEvent(new KeyboardEvent('keydown', { key: '2', ctrlKey: true }));
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true }));
    expect(invoke).toHaveBeenCalledWith('bridge_shortcut', { action: 'veil' });
    expect(invoke).toHaveBeenCalledWith('bridge_shortcut', { action: 'switch:2' });
    window.dispatchEvent(new KeyboardEvent('keydown', { key: '=', ctrlKey: true }));
    expect(invoke).toHaveBeenCalledWith('bridge_shortcut', { action: 'zoom:in' });
    expect(invoke).not.toHaveBeenCalledWith('bridge_shortcut', { action: 'palette' });
  });

  describe('find in page', () => {
    beforeEach(() => {
      document.body.innerHTML =
        '<p>Invoice for Ayesha</p><p>Second invoice</p><script>var invoice = 1;</script><p>Other</p>';
      findHost()?.remove();
    });

    it('opens on Ctrl+F inside the page without asking Cloister', () => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'f', ctrlKey: true }));
      expect(findHost()).not.toBeNull();
      expect(invoke).not.toHaveBeenCalled();
    });

    it('counts matches case-insensitively, skipping scripts', () => {
      api().find();
      search('INVOICE');
      expect(findCount()).toBe('1 of 2');
    });

    it('moves forward and back through matches, wrapping around', () => {
      api().find();
      search('invoice');
      press('Enter');
      expect(findCount()).toBe('2 of 2');
      press('Enter');
      expect(findCount()).toBe('1 of 2');
      press('Enter', { shiftKey: true });
      expect(findCount()).toBe('2 of 2');
    });

    it('says when nothing matches', () => {
      api().find();
      search('nothing like this');
      expect(findCount()).toBe('No results');
    });

    it('searches again after the page changes', () => {
      api().find();
      search('invoice');
      document.body.innerHTML = '<p>invoice one</p><p>invoice two</p><p>invoice three</p>';
      press('Enter');
      expect(findCount()).toBe('1 of 3');
    });

    it('closes with Escape and keeps typing away from the site', () => {
      const siteKeys = vi.fn();
      document.addEventListener('keydown', siteKeys);
      api().find();
      press('a');
      expect(siteKeys).not.toHaveBeenCalled();
      press('Escape');
      expect(findHost()).toBeNull();
      document.removeEventListener('keydown', siteKeys);
    });
  });
});
