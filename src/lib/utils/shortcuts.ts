/**
 * Keyboard shortcuts for the main UI.
 *
 * Inside a service page the injected bridge (`src-tauri/scripts/bridge.js`)
 * forwards the same chords, except Ctrl+K, which many sites use themselves;
 * there, Ctrl+Shift+K opens the palette. Ctrl+F opens the bridge's own find
 * bar inside the page; from the main UI it asks the active page to open it.
 */

import type { ShortcutAction } from '$lib/types';

/** The subset of `KeyboardEvent` this module needs (makes testing easy). */
export type KeyChord = Pick<KeyboardEvent, 'key' | 'ctrlKey' | 'shiftKey' | 'altKey' | 'metaKey'>;

/** Running on a Mac, where shortcuts use Cmd instead of Ctrl. */
export const IS_MAC = typeof navigator !== 'undefined' && /^Mac/i.test(navigator.platform);

/**
 * Maps a key chord to a Cloister action, or `null` when it is not a Cloister shortcut.
 * The main modifier is Cmd on macOS and Ctrl elsewhere.
 */
export function shortcutFor(event: KeyChord, mac = IS_MAC): ShortcutAction | null {
  const primary = mac ? event.metaKey : event.ctrlKey;
  const other = mac ? event.ctrlKey : event.metaKey;
  if (!primary || event.altKey || other) return null;
  const key = event.key.toLowerCase();

  if (key === '=' || key === '+') return 'zoom:in';
  if (key === '-' || key === '_') return 'zoom:out';
  if (key === '0' && !event.shiftKey) return 'zoom:reset';

  if (key === 'k') return 'palette';
  if (event.shiftKey) {
    if (key === 'p') return 'veil';
    if (key === 'f') return 'focus';
    if (key === 'l') return 'lock';
    return null;
  }
  if (key === ',') return 'settings';
  if (key === 'f') return 'find';
  if (/^[1-9]$/.test(key)) return `switch:${Number(key)}`;
  return null;
}

/** Human-readable labels shown in menus and tooltips. */
export function shortcutLabels(mac: boolean) {
  const cmd = mac ? '⌘' : 'Ctrl';
  const shift = mac ? '⇧' : 'Shift';
  // The system-wide shortcuts are registered as CommandOrControl+Alt (see hotkeys.rs).
  const global = mac ? '⌘ ⌥' : 'Ctrl Alt';
  return {
    palette: `${cmd} K`,
    veil: `${cmd} ${shift} P`,
    focus: `${cmd} ${shift} F`,
    lock: `${cmd} ${shift} L`,
    settings: `${cmd} ,`,
    find: `${cmd} F`,
    switchTo: `${cmd} 1–9`,
    zoom: `${cmd} + / − / 0`,
    showHide: `${global} Space`,
    panic: `${global} H`,
  } as const;
}

export const SHORTCUT_LABELS = shortcutLabels(IS_MAC);
