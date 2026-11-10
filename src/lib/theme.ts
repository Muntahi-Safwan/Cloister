/**
 * Themes: metadata, resolution and application.
 *
 * Colours are defined in `src/styles/themes.css`; this module only decides
 * which theme is active and puts its id on `<html data-theme>`.
 */

import { getCurrentWindow } from '@tauri-apps/api/window';

/** Concrete themes, each with a block in `themes.css`. */
export type ThemeId = 'quiet-room' | 'midnight' | 'forest' | 'mono' | 'paper' | 'daylight';

/** What the user picks: a concrete theme, or follow Windows light/dark mode. */
export type ThemeChoice = ThemeId | 'system';

export interface ThemeInfo {
  id: ThemeId;
  name: string;
  description: string;
  scheme: 'dark' | 'light';
  /** Window background shown before the UI paints. Matches `--bg-app`. */
  background: string;
}

export const THEMES: readonly ThemeInfo[] = [
  {
    id: 'quiet-room',
    name: 'Quiet Room',
    description: 'Warm graphite, ember accent',
    scheme: 'dark',
    background: '#121110',
  },
  {
    id: 'midnight',
    name: 'Midnight',
    description: 'Deep blue, sky accent',
    scheme: 'dark',
    background: '#0e131b',
  },
  {
    id: 'forest',
    name: 'Forest',
    description: 'Dark moss, sage accent',
    scheme: 'dark',
    background: '#0f1411',
  },
  {
    id: 'mono',
    name: 'Mono',
    description: 'Neutral greys, white accent',
    scheme: 'dark',
    background: '#101010',
  },
  {
    id: 'paper',
    name: 'Paper',
    description: 'Warm white, terracotta',
    scheme: 'light',
    background: '#f6f2ec',
  },
  {
    id: 'daylight',
    name: 'Daylight',
    description: 'Cool white, blue accent',
    scheme: 'light',
    background: '#f7f8fa',
  },
];

const DARK_DEFAULT: ThemeId = 'quiet-room';
const LIGHT_DEFAULT: ThemeId = 'paper';
const STORAGE_KEY = 'cloister.theme';

/** Turns a choice into a concrete theme, following Windows for `system`. */
export function resolveTheme(choice: ThemeChoice, prefersDark: boolean): ThemeId {
  if (choice !== 'system') return choice;
  return prefersDark ? DARK_DEFAULT : LIGHT_DEFAULT;
}

/** Metadata for a theme id (falls back to the default theme). */
export function themeInfo(id: ThemeId): ThemeInfo {
  return THEMES.find((t) => t.id === id) ?? THEMES[0]!;
}

/** Applies a theme to the document and window, and remembers it for the next launch. */
export function applyTheme(id: ThemeId): void {
  document.documentElement.dataset.theme = id;
  try {
    localStorage.setItem(STORAGE_KEY, id);
  } catch {
    // Storage is a convenience; the backend setting is the source of truth.
  }
  getCurrentWindow()
    .setBackgroundColor(themeInfo(id).background)
    .catch(() => undefined);
}

/** Applies the theme used last time, before the backend has answered. */
export function applyStoredTheme(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored && THEMES.some((t) => t.id === stored)) document.documentElement.dataset.theme = stored;
  } catch {
    // Fall back to the default theme in CSS.
  }
}

/** Calls `onChange` whenever Windows switches between light and dark. Returns a cleanup function. */
export function watchSystemScheme(onChange: (prefersDark: boolean) => void): () => void {
  const query = window.matchMedia('(prefers-color-scheme: dark)');
  const handler = () => onChange(query.matches);
  query.addEventListener('change', handler);
  return () => query.removeEventListener('change', handler);
}

export function systemPrefersDark(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}
