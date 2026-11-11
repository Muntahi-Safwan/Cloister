/** UI constants shared by several components. */

import type { ServiceOptions } from '$lib/types';

/**
 * Service tile colour slots. Each index maps to `--tile-<n>-bg` and
 * `--tile-<n>-fg` in `styles/themes.css`, so tiles follow the theme.
 * The index is stored per service (`Service.color`): only append.
 */
export const TILE_INDEXES = [0, 1, 2, 3, 4, 5] as const;

/** Default behaviour for a newly added service. Matches `ServiceOptions::default()` in Rust. */
export const DEFAULT_OPTIONS: ServiceOptions = {
  notifications: true,
  hidePreviews: false,
  unreadFromTitle: true,
  veilWholePage: false,
  hideDuringFocus: false,
  background: 'keepNotifying',
  userAgent: 'default',
};

/** Focus session lengths offered in the UI, in minutes. */
export const FOCUS_PRESETS = [25, 50] as const;
