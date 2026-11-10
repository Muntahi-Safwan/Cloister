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

const api = () => (window as unknown as { __CLOISTER__: { setVeil: (on: boolean) => void } }).__CLOISTER__;

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
});
