/**
 * Theme checks: every theme in the picker has a CSS block, and its text
 * colours meet WCAG AA contrast (4.5:1) on the main surfaces.
 */
import { readFileSync } from 'node:fs';

import { describe, expect, it, vi } from 'vitest';

// Vitest strips CSS imports, so read the stylesheet from disk (tests run from the project root).
const css = readFileSync('src/styles/themes.css', 'utf8');

vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => ({}) }));

const { THEMES, resolveTheme } = await import('./theme');

/** Reads `--name: #hex` declarations from a theme block. */
function themeVars(id: string): Record<string, string> {
  const block = new RegExp(`\\[data-theme='${id}'\\][^{]*\\{([^}]*)\\}`).exec(css)?.[1] ?? '';
  return Object.fromEntries([...block.matchAll(/--([\w-]+):\s*(#[0-9a-f]{6})/gi)].map((m) => [m[1]!, m[2]!]));
}

function luminance(hex: string): number {
  const channels = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const [r, g, b] = channels.map((c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r! + 0.7152 * g! + 0.0722 * b!;
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi! + 0.05) / (lo! + 0.05);
}

describe('themes', () => {
  for (const theme of THEMES) {
    describe(theme.name, () => {
      const vars = themeVars(theme.id);

      it('has a CSS block whose background matches its metadata', () => {
        expect(vars['bg-app']).toBe(theme.background);
      });

      it('keeps text readable on app and raised surfaces', () => {
        for (const surface of ['bg-app', 'bg-raised']) {
          for (const text of ['text', 'text-muted', 'text-faint', 'accent-text']) {
            expect(contrast(vars[text]!, vars[surface]!), `${text} on ${surface}`).toBeGreaterThanOrEqual(
              4.5,
            );
          }
        }
      });

      it('keeps button labels readable on the accent', () => {
        expect(contrast(vars['accent-ink']!, vars['accent']!)).toBeGreaterThanOrEqual(4.5);
      });
    });
  }

  it('follows Windows for the system choice', () => {
    expect(resolveTheme('system', true)).toBe('quiet-room');
    expect(resolveTheme('system', false)).toBe('paper');
    expect(resolveTheme('forest', false)).toBe('forest');
  });
});
